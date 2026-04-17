use image::{GenericImageView, RgbaImage};
use minifb::{
    Key, MouseButton, MouseMode, Window, WindowOptions,
};
use std::path::{Path, PathBuf};

fn crop_to_circle(
    source: &RgbaImage,
    cx: f32,
    cy: f32,
    radius: f32,
) -> RgbaImage {
    let (width, height) = source.dimensions();
    let mut output = source.clone();

    let radius_squared = radius * radius;

    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;

            if dx * dx + dy * dy > radius_squared {
                // Fully transparent, while preserving the RGB values.
                let pixel = output.get_pixel_mut(x, y);
                pixel.0[3] = 0;
            }
        }
    }

    output
}

fn rgba_to_minifb_buffer(image: &RgbaImage) -> Vec<u32> {
    image
        .pixels()
        .map(|pixel| {
            let [r, g, b, a] = pixel.0;

            // Composite transparent pixels over a checkerboard.
            let x = pixel as *const _ as usize; // Not used; see replacement below.
            let _ = x;

            let alpha = a as u32;
            let background = 40u32;

            let r = (r as u32 * alpha + background * (255 - alpha)) / 255;
            let g = (g as u32 * alpha + background * (255 - alpha)) / 255;
            let b = (b as u32 * alpha + background * (255 - alpha)) / 255;

            (r << 16) | (g << 8) | b
        })
        .collect()
}

fn output_path(input_path: &Path) -> PathBuf {
    let directory = input_path.parent().unwrap_or_else(|| Path::new("."));
    let stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image");

    directory.join(format!("{stem}_circular.png"))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_path = Path::new("/Users/han/Desktop/wallp-1.jpg");

    let image = image::open(input_path)?.to_rgba8();
    let (width, height) = image.dimensions();

    let original_buffer = rgba_to_minifb_buffer(&image);
    let mut display_buffer = original_buffer.clone();

    let mut window = Window::new(
        "Circular Crop - click and drag, Esc to exit",
        width as usize,
        height as usize,
        WindowOptions::default(),
    )?;

    let mut dragging = false;
    let mut mouse_was_down = false;
    let mut center: Option<(f32, f32)> = None;
    let mut current_radius = 0.0f32;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mouse_position = window
            .get_mouse_pos(MouseMode::Clamp)
            .unwrap_or((0.0, 0.0));

        let mouse_down = window.get_mouse_down(MouseButton::Left);

        // Mouse button was just pressed: store the circle center.
        if mouse_down && !mouse_was_down {
            center = Some(mouse_position);
            current_radius = 0.0;
            dragging = true;
        }

        // While dragging, update the radius.
        if dragging && mouse_down {
            if let Some((cx, cy)) = center {
                let dx = mouse_position.0 - cx;
                let dy = mouse_position.1 - cy;
                current_radius = (dx * dx + dy * dy).sqrt();

                display_buffer = original_buffer.clone();
                draw_circle_outline(
                    &mut display_buffer,
                    width as usize,
                    height as usize,
                    cx,
                    cy,
                    current_radius,
                );
            }
        }

        // Mouse button was just released: save the final circle.
        if !mouse_down && mouse_was_down && dragging {
            if let Some((cx, cy)) = center {
                let cropped = crop_to_circle(&image, cx, cy, current_radius);
                let path = output_path(input_path);

                cropped.save(&path)?;
                println!("Saved circular image to {}", path.display());
            }

            dragging = false;
        }

        mouse_was_down = mouse_down;

        window.update_with_buffer(
            &display_buffer,
            width as usize,
            height as usize,
        )?;
    }

    Ok(())
}

fn draw_circle_outline(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    cx: f32,
    cy: f32,
    radius: f32,
) {
    let steps = 720;

    for i in 0..steps {
        let angle = i as f32 / steps as f32 * std::f32::consts::TAU;

        let x = (cx + radius * angle.cos()).round() as i32;
        let y = (cy + radius * angle.sin()).round() as i32;

        if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
            buffer[y as usize * width + x as usize] = 0x00FF0000;
        }
    }

    // Draw a small marker at the center.
    let center_x = cx.round() as i32;
    let center_y = cy.round() as i32;

    for dy in -2..=2 {
        for dx in -2..=2 {
            let x = center_x + dx;
            let y = center_y + dy;

            if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                buffer[y as usize * width + x as usize] = 0x0000FF00;
            }
        }
    }
}
