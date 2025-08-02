//! # ModelToImage
//! 
//! This library aims to convert a 3d model and spit out an image. This library aims to limit
//! the amount of dependencies used, therefore it uses a software renderer instead of something
//! like OpenGL or DirectX (which would be overkill). 
//! 
//! Here is a sample render:
//! 
//! ![fish](output.png)
//! 
//! ## Performance
//! 
//! With a software renderer, you may have some concerns about performance. Well, do not fret
//! as under testing of an `Intel i7-1165G7` with an SSD and `32GB` RAM, here are my results:
//! 
//! ![Performance chart](doc/image.png)
//! 
//! This was tested under the release profile using the command `cargo run --release` and the
//! `time` command on Git Bash. 
//! 
//! ## Example
//! 
//! ```rust
//! use std::path::PathBuf;
//! 
//! fn main() {
//!     let fish = PathBuf::from("C:\\Users\\thrib\\model_to_image\\src\\fish.glb");
//!     let mut image = model_to_image::ModelToImageBuilder::new(&fish)
//!         .with_size((800, 600))
//!         .with_light_direction([0.0, 0.0, -1.0])
//!         .with_margin(0.1)
//!         .build()
//!         .unwrap();
//!     
//!     image.render();
//!     let image_buffer = image.output();
//!     image.write_to(Some(&PathBuf::from(output.png")));
//!     // Writes the image to the path
//! }
//! ```

pub(crate) mod utils;

use std::path::PathBuf;

use image::{DynamicImage, GenericImageView, Rgb, RgbImage};
use nalgebra::Vector3;
use russimp::scene::{PostProcess, Scene};

use crate::utils::Colour;

#[derive(Debug, Clone)]
pub struct ModelToImageBuilder {
    pub model_path: PathBuf,
    pub size: (u32, u32),
    pub light_dir: [f32; 3],
    pub margin: f32,
}

impl ModelToImageBuilder {
    /// Creates a new instance of an model_image builder.
    /// 
    /// ## Parameters
    /// - model_path: A PathBuf to the model itself 
    pub fn new(model_path: &PathBuf) -> Self {
        Self {
            model_path: model_path.clone(),
            size: (256, 256),
            light_dir: Vector3::new(0.0, 0.0 ,-1.0).into(),
            margin: 0.1,
        }
    }

    /// Provides an size for the image. 
    ///
    /// Default: (256, 256) if function not used
    pub fn with_size(mut self, size: (u32, u32)) -> Self {
        self.size = (size.0.max(10), size.1.max(10));
        self
    }

    /// Provides a light direction to be shining onto the model. 
    /// 
    /// Default: (0.0, 0.0, -1.0) if function not used
    pub fn with_light_direction<T: Into<[f32; 3]>>(mut self, light_dir: T) -> Self {
        self.light_dir = light_dir.into();
        self
    }

    /// Adds a margin from the border when rendering the image
    /// 
    /// Default: 0.1_f32
    pub fn with_margin(mut self, margin: f32) -> Self {
        self.margin = margin;
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


#[derive(Debug)]
pub struct ModelToImage {
    #[allow(dead_code)]
    config: ModelToImageBuilder,
    size: Size,
    margin: f32,
    img_buf: RgbImage,
    scene: Scene,
    light_dir: [f32; 3],
    textures: Vec<Option<DynamicImage>>,
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
        let light_dir = builder.light_dir;

        let mut textures = Vec::new();
        for material in &scene.materials {
            if let Some(texture_path) = material.textures.get(&russimp::material::TextureType::Diffuse) {
                match &texture_path.borrow().data {
                    russimp::material::DataContent::Bytes(data) => {
                        match image::load_from_memory(&data) {
                            Ok(img) => textures.push(Some(img)),
                            Err(e) => {
                                eprintln!("Failed to load embedded texture: {}", e);
                                textures.push(None);
                            }
                        }
                    },
                    _ => textures.push(None)
                }
            } else {
                textures.push(None)
            }
        }

        let margin = builder.margin;
        Ok(Self {
            config: builder,
            size,
            img_buf: RgbImage::new(size.width, size.height),
            scene,
            light_dir,
            textures,
            margin,
        })
    }

    /// Starts the rendering, and provides a populated image buffer in the [`ModelToImage`] struct
    pub fn render(&mut self) -> anyhow::Result<&mut Self> {
        self.gen_bkg();

        let mut z_buffer = vec![f32::NEG_INFINITY; (self.size.width * self.size.height) as usize];

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
        let margin = self.margin;
        let scale_x = (self.size.width as f32 * (1.0 - 2.0 * margin)) / model_width;
        let scale_y = (self.size.height as f32 * (1.0 - 2.0 * margin)) / model_height;
        let scale = scale_x.min(scale_y);

        let center_x = (min_x + max_x) / 2.0;
        let center_y = (min_y + max_y) / 2.0;
        let viewport_center_x = self.size.width as f32 / 2.0;
        let viewport_center_y = self.size.height as f32 / 2.0;

        let mesh_draw_data: Vec<(Vec<(i32, i32)>, Vec<Vec<usize>>, Vec<nalgebra::Vector3<f32>>, Vec<Vec<(f32, f32)>>, usize)> = self
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
                let world_coords: Vec<nalgebra::Vector3<f32>> = mesh
                    .vertices
                    .iter()
                    .map(|v| nalgebra::Vector3::new(v.x, v.y, v.z))
                    .collect();

                let texture_coords: Vec<Vec<(f32, f32)>> = faces
                    .iter()
                    .map(|face_indices| {
                        face_indices.iter().map(|&vertex_index| {
                            if let Some(Some(tex_coords)) = mesh.texture_coords.get(0) {
                                if vertex_index < tex_coords.len() {
                                    let tc = &tex_coords[vertex_index];
                                    (tc.x, tc.y)
                                } else {
                                    (0.0, 0.0)
                                }
                            } else {
                                (0.0, 0.0)
                            }
                        }).collect()
                    })
                    .collect();

                let idx = mesh.material_index as usize; 

                (projected, faces, world_coords, texture_coords, idx)
            })
            .collect();

        let light = Vector3::from(self.light_dir).normalize();

        for (projected, faces, world_coords, texture_coords, idx) in mesh_draw_data {
            let texture = if idx < self.textures.len() {
                self.textures[idx].clone()
            } else {
                None
            };

            for (face_idx, indices) in faces.iter().enumerate() {
                let (i0, i1, i2) = (indices[0], indices[1], indices[2]);

                let edge1 = world_coords[i2] - world_coords[i0];
                let edge2 = world_coords[i1] - world_coords[i0];
                let normal = edge1.cross(&edge2).normalize();

                let intensity = normal.dot(&light);

                if intensity > 0.0 {
                    let pts = [
                        (projected[i0].0 as f32, projected[i0].1 as f32, world_coords[i0].z),
                        (projected[i1].0 as f32, projected[i1].1 as f32, world_coords[i1].z),
                        (projected[i2].0 as f32, projected[i2].1 as f32, world_coords[i2].z),
                    ];

                    let tex_coords = if face_idx < texture_coords.len() && texture_coords[face_idx].len() == 3 {
                        Some([
                            texture_coords[face_idx][0],
                            texture_coords[face_idx][1],
                            texture_coords[face_idx][2],
                        ])
                    } else {
                        None
                    };
                    
                    self.draw_triangle(&pts, &mut z_buffer, texture.as_ref(), tex_coords, intensity);
                }
            }
        }

        // at the end, ensure the image is flipped. 
        image::imageops::flip_vertical_in_place(&mut self.img_buf);
        Ok(self)
    }

    fn barycentric(a: (f32, f32), b: (f32, f32), c: (f32, f32), p: (f32, f32)) -> Option<(f32, f32, f32)> {
        let s0 = (c.0 - a.0, b.0 - a.0, a.0 - p.0);
        let s1 = (c.1 - a.1, b.1 - a.1, a.1 - p.1);
        
        let u = (
            s0.1 * s1.2 - s0.2 * s1.1,
            s0.2 * s1.0 - s0.0 * s1.2,
            s0.0 * s1.1 - s0.1 * s1.0
        );
        
        if u.2.abs() > 1e-2 {
            let w0 = 1.0 - (u.0 + u.1) / u.2;
            let w1 = u.1 / u.2;
            let w2 = u.0 / u.2;
            Some((w0, w1, w2))
        } else {
            None
        }
    }

    fn draw_triangle(
        &mut self,
        pts: &[(f32, f32, f32); 3],
        z_buffer: &mut [f32],
        texture: Option<&DynamicImage>,
        tex_coords: Option<[(f32, f32); 3]>,
        light_intensity: f32,
    ) {
        let mut bbox_min = (f32::MAX, f32::MAX);
        let mut bbox_max = (f32::NEG_INFINITY, f32::NEG_INFINITY);
        
        for &(x, y, _) in pts {
            bbox_min.0 = bbox_min.0.min(x);
            bbox_min.1 = bbox_min.1.min(y);
            bbox_max.0 = bbox_max.0.max(x);
            bbox_max.1 = bbox_max.1.max(y);
        }
        
        let min_x = (bbox_min.0.max(0.0) as i32).max(0);
        let max_x = (bbox_max.0.min(self.size.width as f32 - 1.0) as i32).min(self.size.width as i32 - 1);
        let min_y = (bbox_min.1.max(0.0) as i32).max(0);
        let max_y = (bbox_max.1.min(self.size.height as f32 - 1.0) as i32).min(self.size.height as i32 - 1);
        
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let p = (x as f32, y as f32);
                
                if let Some((w0, w1, w2)) = Self::barycentric(
                    (pts[0].0, pts[0].1),
                    (pts[1].0, pts[1].1),
                    (pts[2].0, pts[2].1),
                    p
                ) {
                    if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                        let z = pts[0].2 * w0 + pts[1].2 * w1 + pts[2].2 * w2;
                        
                        let buffer_index = (x + y * self.size.width as i32) as usize;
                        
                        if z > z_buffer[buffer_index] {
                            z_buffer[buffer_index] = z;
                            
                            let color = if let (Some(texture), Some(tex_coords)) = (texture, tex_coords) {
                                let u = tex_coords[0].0 * w0 + tex_coords[1].0 * w1 + tex_coords[2].0 * w2;
                                let v = tex_coords[0].1 * w0 + tex_coords[1].1 * w1 + tex_coords[2].1 * w2;
                                
                                let tex_x = ((u.fract().abs() * texture.width() as f32) as u32).min(texture.width() - 1);
                                let tex_y = (((1.0 - v).fract().abs() * texture.height() as f32) as u32).min(texture.height() - 1);
                                
                                let pixel = texture.get_pixel(tex_x, tex_y);
                                let rgb = pixel.0;
                                
                                let r = ((rgb[0] as f32 * light_intensity).min(255.0)) as u8;
                                let g = ((rgb[1] as f32 * light_intensity).min(255.0)) as u8;
                                let b = ((rgb[2] as f32 * light_intensity).min(255.0)) as u8;
                                
                                Rgb([r, g, b])
                            } else {
                                let color_value = (light_intensity * 255.0) as u8;
                                Rgb([color_value, color_value, color_value])
                            };
                            
                            self.img_buf.put_pixel(x as u32, y as u32, color);
                        }
                    }
                }
            }
        }
    }

    /// Generates a solid white-grayish background as a backdrop
    fn gen_bkg(&mut self) {
        for (_, _, pixel) in self.img_buf.enumerate_pixels_mut() {
            let bkg = Colour::from((211, 211, 211));
            // let bkg = DefinedColours::Black.colour(); // remove after
            *pixel = Rgb(bkg.into());
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
