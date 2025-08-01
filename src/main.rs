use std::path::PathBuf;

use model_to_image;

fn main() -> anyhow::Result<()> {
    let mut model = model_to_image::ModelToImageBuilder::new(PathBuf::from(
        "C:/Users/thrib/model_to_image/src/low_poly_horse.glb",
    ))
    .with_size((800, 800))
    .build()?;

    model.render()?;
    model.write_to(None)?;
    Ok(())
}
