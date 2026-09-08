use image::{Rgba, RgbaImage};
use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Cut a circular region out of `source` and paste it into a transparent,
/// square canvas so the circle sits exactly in the middle.
///
/// The edge is anti-aliased: each output pixel gets a coverage value based on
/// how far it lies from the circle boundary, which removes the hard, jagged
/// staircase you get from a plain inside/outside alpha cutoff.
fn crop_to_circle(source: &RgbaImage, cx: f32, cy: f32, radius: f32) -> RgbaImage {
    let (source_width, source_height) = source.dimensions();

    // Square canvas that fits the circle, plus a transparent margin so the
    // circle visibly sits in the middle of an empty square.
    let diameter = radius * 2.0;
    let padding = (diameter * 0.1).ceil();
    let size = (diameter + padding * 2.0).ceil() as u32 + 2;

    let mut output = RgbaImage::new(size, size);

    let half = size as f32 / 2.0;

    for y in 0..size {
        for x in 0..size {
            // Map this output pixel back into source coordinates, keeping the
            // selected center at the center of the new square.
            let source_x = cx + (x as f32 + 0.5 - half);
            let source_y = cy + (y as f32 + 0.5 - half);

            let dx = source_x - cx;
            let dy = source_y - cy;

            let distance = (dx * dx + dy * dy).sqrt();

            // Coverage ramps from 1 fully inside to 0 fully outside across a
            // single pixel, giving a smooth edge.
            let coverage = (radius - distance + 0.5).clamp(0.0, 1.0);

            if coverage <= 0.0 {
                continue;
            }

            let source_ix = source_x.floor() as i32;
            let source_iy = source_y.floor() as i32;

            if source_ix < 0
                || source_iy < 0
                || source_ix >= source_width as i32
                || source_iy >= source_height as i32
            {
                continue;
            }

            let pixel = source.get_pixel(source_ix as u32, source_iy as u32);

            let alpha = (pixel.0[3] as f32 * coverage).round() as u8;

            output.put_pixel(x, y, Rgba([pixel.0[0], pixel.0[1], pixel.0[2], alpha]));
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

        draw_pixel(buffer, width, height, x, center_y, 0x00FFCC00);
    }

    // Draw a crosshair at the fixed center.
    for offset in -8..=8 {
        draw_pixel(buffer, width, height, center_x + offset, center_y, 0x0000FF00);

        draw_pixel(buffer, width, height, center_x, center_y + offset, 0x0000FF00);
    }

    // Draw a small marker at the radius endpoint.
    draw_pixel(buffer, width, height, end_x, center_y, 0x00FFFFFF);
}

/// Non-interactive mode used for benchmarking:
///   circular-img <input> <cx> <cy> <radius>
fn run_headless(input_path: &Path, cx: f32, cy: f32, radius: f32) -> Result<(), Box<dyn std::error::Error>> {
    let timing = std::env::var_os("CIRCULAR_TIMING").is_some();

    let t0 = Instant::now();
    let image = image::open(input_path)?.to_rgba8();
    let t1 = Instant::now();

    let cropped = crop_to_circle(&image, cx, cy, radius);
    let t2 = Instant::now();

    let save_path = output_path(input_path);
    cropped.save(&save_path)?;
    let t3 = Instant::now();

    if timing {
        eprintln!(
            "[rust] decode {:.2} ms | crop {:.2} ms | save {:.2} ms | total {:.2} ms",
            (t1 - t0).as_secs_f64() * 1000.0,
            (t2 - t1).as_secs_f64() * 1000.0,
            (t3 - t2).as_secs_f64() * 1000.0,
            (t3 - t0).as_secs_f64() * 1000.0,
        );
    }

    println!("Saved circular image to:");
    println!("{}", save_path.display());

    Ok(())
}

fn run_interactive(input_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
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

    let mut center: Option<(f32, f32)> = None;
    let mut radius = 0.0f32;
    let mut dragging = false;
    let mut mouse_was_down = false;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mouse_position = window.get_mouse_pos(MouseMode::Clamp).unwrap_or((0.0, 0.0));

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

        window.update_with_buffer(&display_buffer, width as usize, height as usize)?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);

    let input_path = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/han/Desktop/wallp-1.jpg"));

    // If center and radius are supplied, run without opening a window. This is
    // the path used when comparing runtimes between implementations.
    if let (Some(cx), Some(cy), Some(radius)) = (args.next(), args.next(), args.next()) {
        let cx: f32 = cx.parse()?;
        let cy: f32 = cy.parse()?;
        let radius: f32 = radius.parse()?;

        return run_headless(&input_path, cx, cy, radius);
    }

    run_interactive(&input_path)
}
