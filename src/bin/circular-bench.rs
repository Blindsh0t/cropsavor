// Head-less Rust benchmark implementation.
//
// Uses the same shared C codec as the C and Nim ports (via FFI) so that
// decode/encode cost is identical and the only language-specific work is the
// crop loop. Mirrors the C `run_headless` path:
//
//   circular-bench <input> <cx> <cy> <radius>
//
// Env:
//   CIRCULAR_TIMING=1   print decode / crop / save breakdown
//   CIRCULAR_REPEAT=N   run the crop N times (default 1)

use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_uchar};
use std::path::{Path, PathBuf};
use std::time::Instant;

#[repr(C)]
struct CImage {
    width: c_int,
    height: c_int,
    rgba: *mut c_uchar,
}

impl CImage {
    fn pixels(&self) -> &[u8] {
        let len = (self.width as usize) * (self.height as usize) * 4;
        unsafe { std::slice::from_raw_parts(self.rgba, len) }
    }
}

unsafe extern "C" {
    fn cimg_load(path: *const c_char) -> CImage;
    fn cimg_save_png(path: *const c_char, image: *const CImage) -> c_int;
    fn cimg_free(image: *mut CImage);
}

struct CodecImage(CImage);

impl Drop for CodecImage {
    fn drop(&mut self) {
        unsafe { cimg_free(&mut self.0) };
    }
}

// Same crop as the other ports: anti-aliased full-frame mask.
fn crop_to_circle(source: &CImage, cx: f32, cy: f32, radius: f32) -> (Vec<u8>, usize) {
    let source_width = source.width as usize;
    let source_height = source.height as usize;
    let source_pixels = source.pixels();

    let size = source_width;
    let mut output = vec![0u8; size * size * 4];

    for y in 0..source_height {
        for x in 0..source_width {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;

            let distance = (dx * dx + dy * dy).sqrt();

            let coverage = (radius - distance + 0.5).clamp(0.0, 1.0);

            let dest_offset = (y * size + x) * 4;
            output[dest_offset] = source_pixels[dest_offset];
            output[dest_offset + 1] = source_pixels[dest_offset + 1];
            output[dest_offset + 2] = source_pixels[dest_offset + 2];
            output[dest_offset + 3] =
                (source_pixels[dest_offset + 3] as f32 * coverage).round() as u8;
        }
    }

    (output, size)
}

fn output_path(input_path: &Path) -> PathBuf {
    let directory = input_path.parent().unwrap_or_else(|| Path::new("."));

    let filename = input_path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("image");

    directory.join(format!("{filename}_circular.png"))
}

fn env_int(name: &str, fallback: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(fallback)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);

    let input_path = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/han/Desktop/wallp-1.jpg"));

    let cx: f32 = args.next().unwrap_or_else(|| "640".into()).parse()?;
    let cy: f32 = args.next().unwrap_or_else(|| "280".into()).parse()?;
    let radius: f32 = args.next().unwrap_or_else(|| "220".into()).parse()?;

    let timing = std::env::var_os("CIRCULAR_TIMING").is_some();
    let repeat = env_int("CIRCULAR_REPEAT", 1);

    let c_path = CString::new(input_path.to_string_lossy().as_bytes())?;

    let t0 = Instant::now();
    let image = CodecImage(unsafe { cimg_load(c_path.as_ptr()) });
    let t1 = Instant::now();

    if image.0.rgba.is_null() {
        return Err(format!("failed to load image: {}", input_path.display()).into());
    }

    let mut size = 0usize;
    let mut pixels = Vec::new();

    for _ in 0..repeat {
        let (cropped, cropped_size) = crop_to_circle(&image.0, cx, cy, radius);
        pixels = cropped;
        size = cropped_size;
    }
    let t2 = Instant::now();

    let save_path = output_path(&input_path);
    let c_save = CString::new(save_path.to_string_lossy().as_bytes())?;

    let out_image = CImage {
        width: size as c_int,
        height: size as c_int,
        rgba: pixels.as_mut_ptr(),
    };

    unsafe { cimg_save_png(c_save.as_ptr(), &out_image) };
    let t3 = Instant::now();

    if timing {
        let decode = (t1 - t0).as_secs_f64() * 1000.0;
        let crop = (t2 - t1).as_secs_f64() * 1000.0;
        let save = (t3 - t2).as_secs_f64() * 1000.0;
        eprintln!(
            "[rust] decode {:.2} ms | crop {:.2} ms ({:.4} ms x{}) | save {:.2} ms | total {:.2} ms",
            decode,
            crop,
            crop / repeat as f64,
            repeat,
            save,
            decode + crop + save
        );
    }

    println!("Saved circular image to:");
    println!("{}", save_path.display());

    Ok(())
}
