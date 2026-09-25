use anyhow::{Context, Result};
use image::{DynamicImage, GenericImageView, ImageFormat, Rgba};
use std::path::Path;

use crate::pipeline::types::OutputFormat;

/// Convert an image file from its source format to the target format.
pub fn convert_image(
    input: &Path,
    output: &Path,
    format: &OutputFormat,
    quality: u8,
) -> Result<()> {
    let img =
        image::open(input).with_context(|| format!("Failed to open image: {}", input.display()))?;
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
                std::fs::File::create(output)
                    .with_context(|| format!("Cannot create {}", output.display()))?,
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
/// Match the inset geometry measured from 1080x1440 native BeReal exports.
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

    let scale = pw as f64 / 1080.0;
    // The quarter-pixel offset reproduces the partially covered straight edges.
    let x = 29.75 * scale;
    let y = 29.75 * scale;
    let w = 328.0 * scale;
    let h = 436.0 * scale;
    let border = 4.0 * scale;
    let radius = 33.0 * scale;
    let inner_x = x + border;
    let inner_y = y + border;
    let inner_w = (w - 2.0 * border).max(1.0);
    let inner_h = (h - 2.0 * border).max(1.0);
    let inner_radius = (radius - border).max(0.0);

    let secondary_resized = secondary
        .resize_exact(
            inner_w.round() as u32,
            inner_h.round() as u32,
            image::imageops::FilterType::Lanczos3,
        )
        .to_rgba8();
    let mut canvas = primary.to_rgba8();
    let x_start = ((x - 1.0).floor().max(0.0) as u32).min(pw);
    let y_start = ((y - 1.0).floor().max(0.0) as u32).min(ph);
    let x_end = ((x + w + 1.0).ceil().max(0.0) as u32).min(pw);
    let y_end = ((y + h + 1.0).ceil().max(0.0) as u32).min(ph);

    for cy in y_start..y_end {
        for cx in x_start..x_end {
            let px = cx as f64 + 0.5;
            let py = cy as f64 + 0.5;
            let outer_alpha = rounded_rect_coverage(px, py, x, y, w, h, radius);
            if outer_alpha <= 0.0 {
                continue;
            }
            let inner_alpha =
                rounded_rect_coverage(px, py, inner_x, inner_y, inner_w, inner_h, inner_radius);
            let dst = canvas.get_pixel(cx, cy);
            let sx = (((px - inner_x) / inner_w * secondary_resized.width() as f64).floor() as i64)
                .clamp(0, secondary_resized.width() as i64 - 1) as u32;
            let sy = (((py - inner_y) / inner_h * secondary_resized.height() as f64).floor() as i64)
                .clamp(0, secondary_resized.height() as i64 - 1) as u32;
            let src = secondary_resized.get_pixel(sx, sy);
            let mut out = [0u8; 4];
            for channel in 0..3 {
                let black_with_primary = dst[channel] as f64 * (1.0 - outer_alpha);
                out[channel] = (black_with_primary * (1.0 - inner_alpha)
                    + src[channel] as f64 * inner_alpha)
                    .round() as u8;
            }
            out[3] = 255;
            canvas.put_pixel(cx, cy, Rgba(out));
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

/// One-pixel anti-aliasing around a rounded rectangle's signed-distance edge.
fn rounded_rect_coverage(px: f64, py: f64, x: f64, y: f64, w: f64, h: f64, r: f64) -> f64 {
    let r = r.min(w / 2.0).min(h / 2.0);
    let qx = (px - (x + w / 2.0)).abs() - (w / 2.0 - r);
    let qy = (py - (y + h / 2.0)).abs() - (h / 2.0 - r);
    let distance = qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - r;
    (0.5 - distance).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounded_border_matches_native_reference_coordinates() {
        // Native 1080x1440 references have black at these samples, and background
        // immediately outside them. The corner also follows a 33 px circle.
        let alpha = |x: f64, y: f64| {
            rounded_rect_coverage(x + 0.5, y + 0.5, 29.75, 29.75, 328.0, 436.0, 33.0)
        };
        assert_eq!(alpha(30.0, 70.0), 1.0);
        assert!(alpha(29.0, 70.0) > 0.0 && alpha(29.0, 70.0) < 1.0);
        assert_eq!(alpha(28.0, 70.0), 0.0);
        assert!(alpha(56.0, 30.0) > 0.5);
        assert_eq!(alpha(45.0, 30.0), 0.0);
        assert_eq!(alpha(200.0, 466.0), 0.0);

        // First and last black pixels sampled from two of the supplied native
        // JPEGs. JPEG compression moves a threshold crossing by up to one pixel.
        let native_edges: [(i32, i32, i32); 11] = [
            (30, 56, 330),
            (33, 48, 339),
            (36, 43, 343),
            (39, 40, 346),
            (42, 37, 349),
            (45, 35, 351),
            (48, 33, 353),
            (51, 32, 354),
            (54, 31, 355),
            (57, 30, 356),
            (60, 30, 356),
        ];
        for (y, native_left, native_right) in native_edges {
            let left = (25..80)
                .find(|&x| alpha(x as f64, y as f64) >= 0.5)
                .unwrap();
            let right = (300..365)
                .rev()
                .find(|&x| alpha(x as f64, y as f64) >= 0.5)
                .unwrap();
            assert!((left - native_left).abs() <= 1, "left edge at y={y}");
            assert!((right - native_right).abs() <= 1, "right edge at y={y}");
        }
    }

    #[test]
    fn composite_places_the_native_sized_inset() {
        let directory = std::env::temp_dir().join(format!(
            "bereal_pip_geometry_{}_{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let primary_path = directory.join("primary.png");
        let secondary_path = directory.join("secondary.png");
        image::RgbImage::from_pixel(1080, 1440, image::Rgb([255, 255, 255]))
            .save(&primary_path)
            .unwrap();
        image::RgbImage::from_pixel(320, 428, image::Rgb([200, 100, 50]))
            .save(&secondary_path)
            .unwrap();

        let result = combine_pip(&primary_path, &secondary_path)
            .unwrap()
            .to_rgb8();
        assert_eq!(result.dimensions(), (1080, 1440));
        assert_eq!(result.get_pixel(28, 100).0, [255, 255, 255]);
        assert_eq!(result.get_pixel(31, 100).0, [0, 0, 0]);
        assert_eq!(result.get_pixel(100, 100).0, [200, 100, 50]);
        assert_eq!(result.get_pixel(45, 30).0, [255, 255, 255]);
        assert_eq!(result.get_pixel(200, 466).0, [255, 255, 255]);
        std::fs::remove_file(primary_path).unwrap();
        std::fs::remove_file(secondary_path).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }
}
