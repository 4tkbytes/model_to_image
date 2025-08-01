use std::path::PathBuf;

use model_to_image;

fn main() -> anyhow::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    let mut model_path = PathBuf::new();
    if args.len() >= 2 {
        model_path = PathBuf::from(args[1].clone());
    } else {
        println!("No model specified, using default");
        model_path = PathBuf::from("C:/Users/thrib/model_to_image/src/low_poly_horse.glb");
    }
    let mut model = model_to_image::ModelToImageBuilder::new(PathBuf::from(
        model_path,
    ))
    .with_size((800, 800))
    .build()?;

    model.render()?;
    model.write_to(None)?;
    Ok(())
}
