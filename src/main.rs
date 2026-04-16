use image::{GenericImageView, RgbaImage};
use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};

fn rgba_to_minifb_buffer(image: &RgbaImage) -> Vec<u32> {
    image
        .pixels()
        .map(|pixel| {
            let [r, g, b, _a] = pixel.0;
            ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
        })
        .collect()
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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_path = "/Users/han/Desktop/wallp-1.jpg";

    let image = image::open(input_path)?.to_rgba8();
    let (width, height) = image.dimensions();

    let original_buffer = rgba_to_minifb_buffer(&image);
    let mut display_buffer = original_buffer.clone();

    let mut window = Window::new(
        "Circular Crop - click and drag",
        width as usize,
        height as usize,
        WindowOptions::default(),
    )?;

    let mut center: Option<(f32, f32)> = None;
    let mut current_radius = 0.0f32;
    let mut dragging = false;
    let mut mouse_was_down = false;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mouse_position = window.get_mouse_pos(MouseMode::Clamp).unwrap_or((0.0, 0.0));
        let mouse_down = window.get_mouse_down(MouseButton::Left);

        // remember where the drag started
        if mouse_down && !mouse_was_down {
            center = Some(mouse_position);
            current_radius = 0.0;
            dragging = true;
        }

        // while dragging, grow the radius
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

        if !mouse_down && mouse_was_down {
            dragging = false;
        }

        mouse_was_down = mouse_down;

        window.update_with_buffer(&display_buffer, width as usize, height as usize)?;
    }

    Ok(())
}
