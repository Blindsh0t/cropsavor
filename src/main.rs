use image::RgbaImage;
use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};
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
                output.get_pixel_mut(x, y).0[3] = 0;
            }
        }
    }

    output
}

fn output_path(input_path: &Path) -> PathBuf {
    let directory = input_path.parent().unwrap_or_else(|| Path::new("."));

    let filename = input_path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("image");

    directory.join(format!("{filename}_circular.png"))
}

fn rgba_to_minifb_buffer(image: &RgbaImage) -> Vec<u32> {
    let (width, _) = image.dimensions();

    image
        .pixels()
        .enumerate()
        .map(|(index, pixel)| {
            let [r, g, b, a] = pixel.0;

            let x = index as u32 % width;
            let y = index as u32 / width;

            let checkerboard = if ((x / 16) + (y / 16)) % 2 == 0 {
                220u32
            } else {
                170u32
            };

            let alpha = a as u32;

            let r = (r as u32 * alpha + checkerboard * (255 - alpha)) / 255;
            let g = (g as u32 * alpha + checkerboard * (255 - alpha)) / 255;
            let b = (b as u32 * alpha + checkerboard * (255 - alpha)) / 255;

            (r << 16) | (g << 8) | b
        })
        .collect()
}

fn draw_pixel(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    x: i32,
    y: i32,
    color: u32,
) {
    if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
        buffer[y as usize * width + x as usize] = color;
    }
}

fn draw_circle_preview(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    cx: f32,
    cy: f32,
    radius: f32,
) {
    let center_x = cx.round() as i32;
    let center_y = cy.round() as i32;

    // Draw the circle outline.
    let steps = 720;

    for i in 0..steps {
        let angle = i as f32 / steps as f32 * std::f32::consts::TAU;

        let x = (cx + radius * angle.cos()).round() as i32;
        let y = (cy + radius * angle.sin()).round() as i32;

        draw_pixel(buffer, width, height, x, y, 0x00FF3300);
    }

    // Draw a line from the center to the mouse cursor.
    let end_x = (cx + radius).round() as i32;

    let line_length = (end_x - center_x).abs().max(1);

    for i in 0..=line_length {
        let x = if end_x >= center_x {
            center_x + i
        } else {
            center_x - i
        };

        draw_pixel(
            buffer,
            width,
            height,
            x,
            center_y,
            0x00FFCC00,
        );
    }

    // Draw a crosshair at the fixed center.
    for offset in -8..=8 {
        draw_pixel(
            buffer,
            width,
            height,
            center_x + offset,
            center_y,
            0x0000FF00,
        );

        draw_pixel(
            buffer,
            width,
            height,
            center_x,
            center_y + offset,
            0x0000FF00,
        );
    }

    // Draw a small marker at the radius endpoint.
    draw_pixel(
        buffer,
        width,
        height,
        end_x,
        center_y,
        0x00FFFFFF,
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_path = Path::new("/Users/han/Desktop/wallp-1.jpg");

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
    let mut radius = 0.0f32;
    let mut dragging = false;
    let mut mouse_was_down = false;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mouse_position = window
            .get_mouse_pos(MouseMode::Clamp)
            .unwrap_or((0.0, 0.0));

        let mouse_down = window.get_mouse_down(MouseButton::Left);

        // Mouse button was just pressed.
        if mouse_down && !mouse_was_down {
            center = Some(mouse_position);
            radius = 0.0;
            dragging = true;
        }

        // Mouse is being dragged.
        if dragging && mouse_down {
            if let Some((cx, cy)) = center {
                let dx = mouse_position.0 - cx;
                let dy = mouse_position.1 - cy;

                radius = (dx * dx + dy * dy).sqrt();

                display_buffer = original_buffer.clone();

                draw_circle_preview(
                    &mut display_buffer,
                    width as usize,
                    height as usize,
                    cx,
                    cy,
                    radius,
                );
            }
        }

        // Mouse button was just released.
        if !mouse_down && mouse_was_down && dragging {
            if let Some((cx, cy)) = center {
                if radius > 1.0 {
                    let cropped = crop_to_circle(&image, cx, cy, radius);
                    let save_path = output_path(input_path);

                    cropped.save(&save_path)?;

                    println!("Saved circular image to:");
                    println!("{}", save_path.display());
                }
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
