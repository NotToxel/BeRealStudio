use anyhow::{Context, Result};
use image::{DynamicImage, GenericImageView, ImageFormat, Rgba, RgbaImage};
use std::path::Path;

use crate::pipeline::types::OutputFormat;

// PIP constants (matching the Python script values)
const PIP_SCALE: f32 = 1.0 / 3.333_333;
const PIP_CORNER_RADIUS: u32 = 60;
const PIP_OUTLINE_SIZE: u32 = 7;
const PIP_POSITION_X: u32 = 55;
const PIP_POSITION_Y: u32 = 55;

/// Convert an image file from its source format to the target format.
pub fn convert_image(
    input: &Path,
    output: &Path,
    format: &OutputFormat,
    quality: u8,
) -> Result<()> {
    let img = image::open(input)
        .with_context(|| format!("Failed to open image: {}", input.display()))?;
    let rgb = img.to_rgb8();
    save_rgb_image(&rgb, output, format, quality)?;
    Ok(())
}

/// Save an RGB8 image buffer to disk with the given format and quality.
pub fn save_rgb_image(
    img: &image::RgbImage,
    output: &Path,
    format: &OutputFormat,
    quality: u8,
) -> Result<()> {
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    match format {
        OutputFormat::Jpeg => {
            let mut out = std::io::BufWriter::new(
                std::fs::File::create(output).with_context(|| format!("Cannot create {}", output.display()))?,
            );
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, quality);
            encoder.encode_image(img)?;
        }
        OutputFormat::WebP => {
            image::DynamicImage::ImageRgb8(img.clone())
                .save_with_format(output, ImageFormat::WebP)?;
        }
        OutputFormat::Png => {
            image::DynamicImage::ImageRgb8(img.clone())
                .save_with_format(output, ImageFormat::Png)?;
        }
    }
    Ok(())
}

/// Open an image and copy it (format-converting as needed) to the output path.
pub fn copy_image(input: &Path, output: &Path, format: &OutputFormat, quality: u8) -> Result<()> {
    convert_image(input, output, format, quality)
}

/// Create a Picture-in-Picture composite: secondary camera overlaid on primary.
/// Matches the Python script's combine_images() function exactly.
pub fn combine_pip(primary_path: &Path, secondary_path: &Path) -> Result<DynamicImage> {
    let primary = image::open(primary_path)
        .with_context(|| format!("Failed to open primary: {}", primary_path.display()))?;
    let secondary = image::open(secondary_path)
        .with_context(|| format!("Failed to open secondary: {}", secondary_path.display()))?;

    let (pw, ph) = primary.dimensions();
    let (sw, sh) = secondary.dimensions();

    if pw == 0 || ph == 0 {
        anyhow::bail!("Primary image has invalid zero dimensions: {}x{}", pw, ph);
    }
    if sw == 0 || sh == 0 {
        anyhow::bail!("Secondary image has invalid zero dimensions: {}x{}", sw, sh);
    }

    // Scale secondary down to 1/3.333
    let new_sw = ((sw as f32 * PIP_SCALE) as u32).max(1);
    let new_sh = ((sh as f32 * PIP_SCALE) as u32).max(1);
    let secondary_resized = secondary
        .resize_exact(new_sw, new_sh, image::imageops::FilterType::Triangle)
        .to_rgba8();

    // Create rounded mask for secondary
    let mask = create_rounded_mask(new_sw, new_sh, PIP_CORNER_RADIUS);

    // Start with primary as base canvas
    let mut canvas = primary.to_rgba8();

    // Draw black outline (rounded rect) behind the secondary
    let ol = PIP_OUTLINE_SIZE;
    let ox = PIP_POSITION_X.saturating_sub(ol);
    let oy = PIP_POSITION_Y.saturating_sub(ol);
    draw_rounded_rect_filled(
        &mut canvas,
        ox,
        oy,
        new_sw + ol * 2,
        new_sh + ol * 2,
        PIP_CORNER_RADIUS + ol,
        Rgba([0, 0, 0, 255]),
    );

    // Composite secondary (with rounded mask) onto canvas at position
    for y in 0..new_sh {
        for x in 0..new_sw {
            let mask_px = mask.get_pixel(x, y)[0];
            if mask_px == 0 {
                continue;
            }
            let src = secondary_resized.get_pixel(x, y);
            let cx = PIP_POSITION_X + x;
            let cy = PIP_POSITION_Y + y;
            if cx < pw && cy < ph {
                canvas.put_pixel(cx, cy, *src);
            }
        }
    }

    Ok(DynamicImage::ImageRgba8(canvas))
}

/// Create a side-by-side composite: both images placed horizontally at equal height.
pub fn combine_side_by_side(primary_path: &Path, secondary_path: &Path) -> Result<DynamicImage> {
    let primary = image::open(primary_path)
        .with_context(|| format!("Failed to open primary: {}", primary_path.display()))?;
    let secondary = image::open(secondary_path)
        .with_context(|| format!("Failed to open secondary: {}", secondary_path.display()))?;

    let (pw, ph) = primary.dimensions();
    let target_h = ph;
    let (sw, sh) = secondary.dimensions();

    if pw == 0 || ph == 0 {
        anyhow::bail!("Primary image has invalid zero dimensions: {}x{}", pw, ph);
    }
    if sw == 0 || sh == 0 {
        anyhow::bail!("Secondary image has invalid zero dimensions: {}x{}", sw, sh);
    }

    // Scale secondary to same height as primary
    let new_sw = (sw.saturating_mul(target_h) / sh.max(1)).max(1);
    let secondary_resized = secondary
        .resize_exact(new_sw, target_h, image::imageops::FilterType::Triangle)
        .to_rgb8();
    let primary_rgb = primary.to_rgb8();

    // Create wide canvas
    let total_w = pw.saturating_add(new_sw);
    let mut canvas = image::RgbImage::new(total_w, target_h);
    image::imageops::overlay(&mut canvas, &primary_rgb, 0, 0);
    image::imageops::overlay(&mut canvas, &secondary_resized, pw as i64, 0);

    Ok(DynamicImage::ImageRgb8(canvas))
}

/// Create an L-channel (grayscale) alpha mask with rounded corners.
fn create_rounded_mask(w: u32, h: u32, radius: u32) -> image::GrayImage {
    let mut mask = image::GrayImage::new(w, h);
    let r = (radius.min(w / 2).min(h / 2)) as f64;
    let r_sq = r * r;
    let wf = w as f64;
    let hf = h as f64;

    for y in 0..h {
        let yf = y as f64;
        for x in 0..w {
            let xf = x as f64;
            // Check top-left corner
            let outside = if xf < r && yf < r {
                let dx = xf - r;
                let dy = yf - r;
                dx * dx + dy * dy > r_sq
            } else if xf >= wf - r && yf < r {
                // Top-right corner
                let dx = xf - (wf - 1.0 - r);
                let dy = yf - r;
                dx * dx + dy * dy > r_sq
            } else if xf < r && yf >= hf - r {
                // Bottom-left corner
                let dx = xf - r;
                let dy = yf - (hf - 1.0 - r);
                dx * dx + dy * dy > r_sq
            } else if xf >= wf - r && yf >= hf - r {
                // Bottom-right corner
                let dx = xf - (wf - 1.0 - r);
                let dy = yf - (hf - 1.0 - r);
                dx * dx + dy * dy > r_sq
            } else {
                false
            };

            let val = if outside { 0u8 } else { 255u8 };
            mask.put_pixel(x, y, image::Luma([val]));
        }
    }
    mask
}

/// Draw a filled rounded rectangle on an RGBA canvas.
fn draw_rounded_rect_filled(
    img: &mut RgbaImage,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    radius: u32,
    color: Rgba<u8>,
) {
    let mask = create_rounded_mask(w, h, radius);
    let img_w = img.width();
    let img_h = img.height();

    for my in 0..h {
        for mx in 0..w {
            if mask.get_pixel(mx, my)[0] > 0 {
                let target_x = x + mx;
                let target_y = y + my;
                if target_x < img_w && target_y < img_h {
                    img.put_pixel(target_x, target_y, color);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rounded_mask_corners_are_zero() {
        let mask = create_rounded_mask(300, 300, 60);
        // Corner pixel at (0,0) should be black (masked)
        assert_eq!(mask.get_pixel(0, 0)[0], 0);
        // Center should be white
        assert_eq!(mask.get_pixel(150, 150)[0], 255);
    }
}
