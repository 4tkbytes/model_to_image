pub(crate) mod utils;

use std::path::PathBuf;

use image::{Rgb, RgbImage};
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
        ModelToImage::new(self)
    }
}

pub struct ModelToImage {
    #[allow(dead_code)]
    config: ModelToImageBuilder,
    size: Size,
    img_buf: RgbImage,
}

#[derive(Debug, Clone, Copy)]
pub struct Size {
    width: u32,
    height: u32,
}

impl ModelToImage {
    pub(crate) fn new(builder: ModelToImageBuilder) -> anyhow::Result<Self> {
        let size = builder.size;
        let size = Size {
            width: size.0,
            height: size.1,
        };
        Ok(Self {
            config: builder,
            size,
            img_buf: RgbImage::new(size.width, size.height),
        })
    }

    /// Starts the rendering, and provides a populated image buffer in the [`ModelToImage`] struct
    pub fn render(&mut self) -> anyhow::Result<&mut Self> {
        self.gen_bkg();
        let verts: Vec<(i32, i32)> = vec![(2, 3), (12, 37), (62, 53)];
        // let indices = [0, 1, 2, 1, 2, 0, 0, 2];

        self.create_vertices(&verts);
        use rand::Rng;
        let mut rng = rand::rng();
        let num_lines = 2i32.pow(24) as usize;
        for _ in 0..num_lines {
            let ax = rng.random_range(0..self.size.width as i32);
            let ay = rng.random_range(0..self.size.height as i32);
            let bx = rng.random_range(0..self.size.width as i32);
            let by = rng.random_range(0..self.size.height as i32);

            let r = rng.random_range(0..=255);
            let g = rng.random_range(0..=255);
            let b = rng.random_range(0..=255);

            let color = Rgb([r, g, b]);
            self.draw_line(ax, ay, bx, by, color);
        }

        Ok(self)
    }

    /// Generates a solid white-grayish background as a backdrop
    fn gen_bkg(&mut self) {
        for (_, _, pixel) in self.img_buf.enumerate_pixels_mut() {
            // let bkg = Colour::new_u8(211, 211, 211);
            let bkg = DefinedColours::Black.colour(); // remove after
            *pixel = Rgb(bkg.to_array());
        }
    }

    fn create_vertices(&mut self, vertices: &Vec<(i32, i32)>) {
        for (x, y) in vertices {
            if x < &(self.size.width as i32) && y < &(self.size.height as i32) {
                self.img_buf.put_pixel(
                    x.clone() as u32,
                    y.clone() as u32,
                    Rgb(DefinedColours::White.colour().to_array()),
                );
            } else {
                eprintln!(
                    "Warning: Point ({}, {}) is not able to put placed onto the image for not being within the image bounds of ({}, {})",
                    x, y, self.size.width, self.size.height
                );
            }
        }
    }

    /// Draws a line between the vertices
    fn draw_line(&mut self, mut ax: i32, mut ay: i32, mut bx: i32, mut by: i32, color: Rgb<u8>) {
        let steep = (ax - bx).abs() < (ay - by).abs();
        if steep {
            std::mem::swap(&mut ax, &mut ay);
            std::mem::swap(&mut bx, &mut by);
        }
        if ax > bx {
            std::mem::swap(&mut ax, &mut bx);
            std::mem::swap(&mut ay, &mut by);
        }

        let mut y = ay;
        let mut ierror = 0;
        for x in ax..=bx {
            if steep {
                self.img_buf.put_pixel(y as u32, x as u32, color);
            } else {
                self.img_buf.put_pixel(x as u32, y as u32, color);
            }
            ierror += 2 * (by - ay).abs();
            let step = if by > ay { 1 } else { -1 };
            let dx = bx - ax;
            if ierror > dx {
                y += step;
                ierror -= 2 * (dx);
            }
        }
    }

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
