use image::{GenericImageView, RgbaImage};
use minifb::{Key, Window, WindowOptions};

fn rgba_to_minifb_buffer(image: &RgbaImage) -> Vec<u32> {
    image
        .pixels()
        .map(|pixel| {
            let [r, g, b, _a] = pixel.0;
            ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
        })
        .collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_path = "/Users/han/Desktop/wallp-1.jpg";

    let image = image::open(input_path)?.to_rgba8();
    let (width, height) = image.dimensions();

    let buffer = rgba_to_minifb_buffer(&image);

    let mut window = Window::new(
        "Circular Crop",
        width as usize,
        height as usize,
        WindowOptions::default(),
    )?;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&buffer, width as usize, height as usize)?;
    }

    Ok(())
}
