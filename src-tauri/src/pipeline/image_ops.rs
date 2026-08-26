use anyhow::{Context, Result};
use image::{DynamicImage, GenericImageView, ImageFormat, Rgba, RgbaImage};
use std::path::Path;

use crate::pipeline::types::OutputFormat;

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
/// Matches official BeReal aspect ratio, corner continuous radius, and positioning.
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

    // 100% exact pixel parity ratios matching official BeReal layout
    let outer_w = ((pw as f32 * 0.304688).round() as u32).max(1);
    let outer_h = ((outer_w as f32 * (4.0 / 3.0)).round() as u32).max(1);
    let outline_size = ((pw as f32 * 0.005208).round() as u32).max(4);
    let outer_radius = ((outer_w as f32 * 0.1624).round() as u32).max(8);
    let pos_x = ((pw as f32 * 0.037760).round() as u32).max(8);
    let pos_y = ((pw as f32 * 0.037760).round() as u32).max(8);

    let inner_w = outer_w.saturating_sub(outline_size * 2).max(1);
    let inner_h = outer_h.saturating_sub(outline_size * 2).max(1);
    let inner_radius = outer_radius.saturating_sub(outline_size).max(4);
    let inner_x = pos_x + outline_size;
    let inner_y = pos_y + outline_size;

    let secondary_resized = secondary
        .resize_exact(inner_w, inner_h, image::imageops::FilterType::Lanczos3)
        .to_rgba8();

    // Create anti-aliased rounded mask for secondary inner content
    let mask = create_rounded_mask(inner_w, inner_h, inner_radius);

    // Start with primary as base canvas
    let mut canvas = primary.to_rgba8();

    // Draw smooth black outline (rounded rect) containing the secondary
    draw_rounded_rect_filled(
        &mut canvas,
        pos_x,
        pos_y,
        outer_w,
        outer_h,
        outer_radius,
        Rgba([0, 0, 0, 255]),
    );

    // Alpha-blend composite secondary onto canvas
    for y in 0..inner_h {
        for x in 0..inner_w {
            let mask_alpha = mask.get_pixel(x, y)[0] as f32 / 255.0;
            if mask_alpha <= 0.001 {
                continue;
            }
            let src = secondary_resized.get_pixel(x, y);
            let cx = inner_x + x;
            let cy = inner_y + y;
            if cx < pw && cy < ph {
                if mask_alpha >= 0.999 {
                    canvas.put_pixel(cx, cy, *src);
                } else {
                    let dst = canvas.get_pixel(cx, cy);
                    let inv = 1.0 - mask_alpha;
                    let r = (src[0] as f32 * mask_alpha + dst[0] as f32 * inv).round() as u8;
                    let g = (src[1] as f32 * mask_alpha + dst[1] as f32 * inv).round() as u8;
                    let b = (src[2] as f32 * mask_alpha + dst[2] as f32 * inv).round() as u8;
                    canvas.put_pixel(cx, cy, Rgba([r, g, b, 255]));
                }
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
        .resize_exact(new_sw, target_h, image::imageops::FilterType::Lanczos3)
        .to_rgb8();
    let primary_rgb = primary.to_rgb8();

    // Create wide canvas
    let total_w = pw.saturating_add(new_sw);
    let mut canvas = image::RgbImage::new(total_w, target_h);
    image::imageops::overlay(&mut canvas, &primary_rgb, 0, 0);
    image::imageops::overlay(&mut canvas, &secondary_resized, pw as i64, 0);

    Ok(DynamicImage::ImageRgb8(canvas))
}

/// Create an anti-aliased L-channel alpha mask with continuous rounded corners.
fn create_rounded_mask(w: u32, h: u32, radius: u32) -> image::GrayImage {
    let mut mask = image::GrayImage::new(w, h);
    let r = (radius.min(w / 2).min(h / 2)) as f64;
    let wf = w as f64;
    let hf = h as f64;

    for y in 0..h {
        let yf = y as f64 + 0.5;
        for x in 0..w {
            let xf = x as f64 + 0.5;

            let dist = if xf < r && yf < r {
                let dx = r - xf;
                let dy = r - yf;
                (dx * dx + dy * dy).sqrt()
            } else if xf > wf - r && yf < r {
                let dx = xf - (wf - r);
                let dy = r - yf;
                (dx * dx + dy * dy).sqrt()
            } else if xf < r && yf > hf - r {
                let dx = r - xf;
                let dy = yf - (hf - r);
                (dx * dx + dy * dy).sqrt()
            } else if xf > wf - r && yf > hf - r {
                let dx = xf - (wf - r);
                let dy = yf - (hf - r);
                (dx * dx + dy * dy).sqrt()
            } else {
                0.0
            };

            let alpha = if dist <= 0.0 {
                255u8
            } else if dist <= r - 0.5 {
                255u8
            } else if dist >= r + 0.5 {
                0u8
            } else {
                ((r + 0.5 - dist) * 255.0).clamp(0.0, 255.0).round() as u8
            };

            mask.put_pixel(x, y, image::Luma([alpha]));
        }
    }
    mask
}

/// Draw a filled anti-aliased rounded rectangle on an RGBA canvas.
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
            let alpha = mask.get_pixel(mx, my)[0] as f32 / 255.0;
            if alpha > 0.0 {
                let target_x = x + mx;
                let target_y = y + my;
                if target_x < img_w && target_y < img_h {
                    if alpha >= 0.999 {
                        img.put_pixel(target_x, target_y, color);
                    } else {
                        let dst = img.get_pixel(target_x, target_y);
                        let inv = 1.0 - alpha;
                        let r = (color[0] as f32 * alpha + dst[0] as f32 * inv).round() as u8;
                        let g = (color[1] as f32 * alpha + dst[1] as f32 * inv).round() as u8;
                        let b = (color[2] as f32 * alpha + dst[2] as f32 * inv).round() as u8;
                        img.put_pixel(target_x, target_y, Rgba([r, g, b, 255]));
                    }
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
