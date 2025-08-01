pub(crate) mod utils;

use std::path::PathBuf;

use image::{Rgb, RgbImage};
use russimp::scene::{PostProcess, Scene};
use utils::DefinedColours;

#[derive(Clone)]
pub struct ModelToImageBuilder {
    pub model_path: PathBuf,
    pub size: (u32, u32),
}

impl ModelToImageBuilder {
    pub fn new(model_path: PathBuf) -> Self {
        Self {
            model_path,
            size: (256, 256),
        }
    }

    /// Provides a size in the case you wish to provide a custom one.
    ///
    /// Default: (256, 256) if function not used
    pub fn with_size(mut self, size: (u32, u32)) -> Self {
        self.size = (size.0.max(10), size.1.max(10));
        self
    }

    pub fn build(self) -> anyhow::Result<ModelToImage> {
        if !self.model_path.exists() {
            return Err(anyhow::anyhow!(format!(
                "The model path [{}] does not exist on disk. Please ensure it exists or the path provided is correct.",
                self.model_path.to_str().unwrap()
            )));
        }
        let scene = Scene::from_file(
            self.model_path.to_str().unwrap(),
            vec![
                PostProcess::CalculateTangentSpace,
                PostProcess::Triangulate,
                PostProcess::JoinIdenticalVertices,
                PostProcess::SortByPrimitiveType,
            ],
        )?;
        ModelToImage::new(self, scene)
    }
}

pub struct ModelToImage {
    #[allow(dead_code)]
    config: ModelToImageBuilder,
    size: Size,
    img_buf: RgbImage,
    scene: Scene,
}

#[derive(Debug, Clone, Copy)]
pub struct Size {
    width: u32,
    height: u32,
}

impl ModelToImage {
    pub(crate) fn new(builder: ModelToImageBuilder, scene: Scene) -> anyhow::Result<Self> {
        let size = builder.size;
        let size = Size {
            width: size.0,
            height: size.1,
        };
        Ok(Self {
            config: builder,
            size,
            img_buf: RgbImage::new(size.width, size.height),
            scene,
        })
    }

    /// Starts the rendering, and provides a populated image buffer in the [`ModelToImage`] struct
    pub fn render(&mut self) -> anyhow::Result<&mut Self> {
        self.gen_bkg();

        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for mesh in &self.scene.meshes {
            for vertex in &mesh.vertices {
                min_x = min_x.min(vertex.x);
                max_x = max_x.max(vertex.x);
                min_y = min_y.min(vertex.y);
                max_y = max_y.max(vertex.y);
            }
        }

        let model_width = max_x - min_x;
        let model_height = max_y - min_y;
        let margin = 0.1; // 10 percent margin
        let scale_x = (self.size.width as f32 * (1.0 - 2.0 * margin)) / model_width;
        let scale_y = (self.size.height as f32 * (1.0 - 2.0 * margin)) / model_height;
        let scale = scale_x.min(scale_y);

        let center_x = (min_x + max_x) / 2.0;
        let center_y = (min_y + max_y) / 2.0;
        let viewport_center_x = self.size.width as f32 / 2.0;
        let viewport_center_y = self.size.height as f32 / 2.0;

        let mesh_draw_data: Vec<(Vec<(i32, i32)>, Vec<Vec<usize>>)> = self
            .scene
            .meshes
            .iter()
            .map(|mesh| {
                let projected: Vec<(i32, i32)> = mesh
                    .vertices
                    .iter()
                    .map(|v| {
                        let x = ((v.x - center_x) * scale + viewport_center_x) as i32;
                        let y = ((v.y - center_y) * scale + viewport_center_y) as i32;
                        (x, y)
                    })
                    .collect();
                let faces: Vec<Vec<usize>> = mesh
                    .faces
                    .iter()
                    .filter(|face| face.0.len() == 3)
                    .map(|face| face.0.iter().map(|&idx| idx as usize).collect())
                    .collect();
                (projected, faces)
            })
            .collect();
        for (projected, faces) in mesh_draw_data {
            for indices in faces {
                let (i0, i1, i2) = (indices[0], indices[1], indices[2]);
                let color = Rgb([rand::random_range(0..=255), rand::random_range(0..=255), rand::random_range(0..=255)]);
                
                self.triangle(
                    projected[i0].0, projected[i0].1,
                    projected[i1].0, projected[i1].1,
                    projected[i2].0, projected[i2].1,
                    color,
                );
            }
        }

        Ok(self)
    }

    /// Draws a triangle using barycentric coordinates and fills it with a colour
    fn triangle(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, x2: i32, y2: i32, color: Rgb<u8>) {
        let min_x = (x0.min(x1).min(x2)).max(0);
        let max_x = (x0.max(x1).max(x2)).min(self.size.width as i32 - 1);
        let min_y = (y0.min(y1).min(y2)).max(0);
        let max_y = (y0.max(y1).max(y2)).min(self.size.height as i32 - 1);

        let area = ((x1 - x0) * (y2 - y0) - (x2 - x0) * (y1 - y0)) as f32;
        
        if area.abs() < 0.5 {
            return;
        }

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let w0 = ((x1 - x) * (y2 - y) - (x2 - x) * (y1 - y)) as f32 / area;
                let w1 = ((x2 - x) * (y0 - y) - (x0 - x) * (y2 - y)) as f32 / area;
                let w2 = ((x0 - x) * (y1 - y) - (x1 - x) * (y0 - y)) as f32 / area;

                // Check if point is inside triangle
                if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                    self.img_buf.put_pixel(x as u32, y as u32, color);
                }
            }
        }
    }

    // /// Draws an image by clipping to the image size bounds
    // fn draw_line_safe(&mut self, ax: i32, ay: i32, bx: i32, by: i32, color: Rgb<u8>) {
    //     let width = self.size.width as i32;
    //     let height = self.size.height as i32;

    //     if (ax >= 0 && ax < width && ay >= 0 && ay < height)
    //         || (bx >= 0 && bx < width && by >= 0 && by < height)
    //     {
    //         self.draw_line(ax, ay, bx, by, color);
    //     }
    // }

    /// Generates a solid white-grayish background as a backdrop
    fn gen_bkg(&mut self) {
        for (_, _, pixel) in self.img_buf.enumerate_pixels_mut() {
            // let bkg = Colour::new_u8(211, 211, 211);
            let bkg = DefinedColours::Black.colour(); // remove after
            *pixel = Rgb(bkg.to_array());
        }
    }

    // fn create_vertices(&mut self, vertices: &Vec<(i32, i32)>) {
    //     for (x, y) in vertices {
    //         if x < &(self.size.width as i32) && y < &(self.size.height as i32) {
    //             self.img_buf.put_pixel(
    //                 x.clone() as u32,
    //                 y.clone() as u32,
    //                 Rgb(DefinedColours::White.colour().to_array()),
    //             );
    //         } else {
    //             eprintln!(
    //                 "Warning: Point ({}, {}) is not able to put placed onto the image for not being within the image bounds of ({}, {})",
    //                 x, y, self.size.width, self.size.height
    //             );
    //         }
    //     }
    // }

    // /// Draws a line between the vertices
    // fn draw_line(&mut self, mut ax: i32, mut ay: i32, mut bx: i32, mut by: i32, color: Rgb<u8>) {
    //     let steep = (ax - bx).abs() < (ay - by).abs();
    //     if steep {
    //         std::mem::swap(&mut ax, &mut ay);
    //         std::mem::swap(&mut bx, &mut by);
    //     }
    //     if ax > bx {
    //         std::mem::swap(&mut ax, &mut bx);
    //         std::mem::swap(&mut ay, &mut by);
    //     }
    //     let mut y = ay;
    //     let mut ierror = 0;
    //     for x in ax..=bx {
    //         if steep {
    //             self.img_buf.put_pixel(y as u32, x as u32, color);
    //         } else {
    //             self.img_buf.put_pixel(x as u32, y as u32, color);
    //         }
    //         ierror += 2 * (by - ay).abs();
    //         let step = if by > ay { 1 } else { -1 };
    //         let dx = bx - ax;
    //         if ierror > dx {
    //             y += step;
    //             ierror -= 2 * (dx);
    //         }
    //     }
    // }

    /// Provides the image buffer as an output for your own manipulation
    /// of the image
    pub fn output(&self) -> &RgbImage {
        &self.img_buf
    }

    /// Writes to a location as a file. By default, it is optional. If no path is provided, it is saved
    /// as `output.png`.
    pub fn write_to(&self, location: Option<&PathBuf>) -> anyhow::Result<()> {
        if let Some(path) = location {
            self.img_buf.save(path)?;
            Ok(())
        } else {
            self.img_buf.save("output.png")?;
            Ok(())
        }
    }
}

#[allow(dead_code)]
pub fn render() {
    let image_width = 256;
    let image_height = 256;

    let mut image_data: RgbImage = RgbImage::new(image_width, image_height);

    for (x, y, pixel) in image_data.enumerate_pixels_mut() {
        let r = x as f64 / (image_width - 1) as f64;
        let g = y as f64 / (image_height - 1) as f64;
        let b = 0.0;

        let ir = (255.999 * r) as u8;
        let ig = (255.999 * g) as u8;
        let ib = (255.999 * b) as u8;

        *pixel = Rgb([ir, ig, ib]);
    }
    image_data.save("output.png").unwrap();
}
