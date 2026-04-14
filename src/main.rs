use image::GenericImageView;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_path = "/Users/han/Desktop/wallp-1.jpg";

    let image = image::open(input_path)?;
    let (width, height) = image.dimensions();

    println!("loaded {width}x{height}");

    Ok(())
}
