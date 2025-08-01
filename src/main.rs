use std::path::PathBuf;

use model_to_image;

fn main() -> anyhow::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    let model_path = if args.len() >= 2 {
        PathBuf::from(args[1].clone())
    } else {
        #[cfg(debug_assertions)]
        {
            println!("No model specified, using default");
            println!("All args: {:?}", args)
        }
        PathBuf::from("C:/Users/thrib/model_to_image/src/fish.glb")
    };
    let mut model = model_to_image::ModelToImageBuilder::new(PathBuf::from(
        model_path,
    ))
    .with_size((800, 800))
    .build()?;

    model.render()?;
    model.write_to(None)?;
    Ok(())
}
