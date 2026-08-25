use ab_glyph::{Font, FontArc, PxScale, ScaleFont};
use anyhow::{Context, Result};
use image::Rgba;
use imageproc::drawing::draw_text_mut;

use crate::pipeline::types::{RecapperConfig, TextPosition};
use crate::recapper::font_resolver::load_font;

/// Render a Recapper frame: load the image, resize to target, overlay date/location text.
/// Ported from BeReal-Recapper.py process_image().
pub fn render_frame(
    image_path: &std::path::Path,
    config: &RecapperConfig,
    date_str: &str,
    location_str: &str,
) -> Result<image::RgbImage> {
    let img = image::open(image_path)
        .with_context(|| format!("Cannot open {}", image_path.display()))?;

    let (target_w, target_h) = config.resolution;
    let resized = img
        .resize_to_fill(target_w, target_h, image::imageops::FilterType::Lanczos3)
        .to_rgba8();

    let mut canvas = resized;

    // Load font from built-in fonts or custom path
    let font = load_font(&config.font_path).unwrap_or_else(|_| {
        load_font("inter").expect("Default Inter font must be available")
    });

    let scale = PxScale::from(config.font_size as f32);
    let shadow = config.shadow_strength as i32;

    // Render date text
    let date_rect = if config.date_enabled && !date_str.is_empty() {
        let (tw, th) = measure_text(&font, scale, date_str);
        let (x, y) = calc_position(
            &config.date_position,
            target_w as i32,
            target_h as i32,
            tw as i32,
            th as i32,
            config.date_offset,
        );
        // Shadow
        draw_text_mut(
            &mut canvas,
            Rgba([0, 0, 0, 180]),
            x + shadow,
            y + shadow,
            scale,
            &font,
            date_str,
        );
        // Text
        draw_text_mut(&mut canvas, Rgba([255, 255, 255, 255]), x, y, scale, &font, date_str);
        Some((x, y, tw, th))
    } else {
        None
    };

    // Render location text
    if config.location_enabled && !location_str.is_empty() {
        let (tw, th) = measure_text(&font, scale, location_str);
        let (x, y) = if config.location_position == TextPosition::BelowDate {
            if let Some((dx, dy, dw, dh)) = date_rect {
                let center_x = dx + dw as i32 / 2;
                let lx = center_x - tw as i32 / 2 + config.location_offset.0;
                let ly = dy + dh as i32 + 10 + config.location_offset.1;
                (lx, ly)
            } else {
                calc_position(
                    &config.location_position,
                    target_w as i32,
                    target_h as i32,
                    tw as i32,
                    th as i32,
                    config.location_offset,
                )
            }
        } else if config.location_position == TextPosition::AboveDate {
            if let Some((dx, dy, dw, _dh)) = date_rect {
                let center_x = dx + dw as i32 / 2;
                let lx = center_x - tw as i32 / 2 + config.location_offset.0;
                let ly = dy - th as i32 - 10 + config.location_offset.1;
                (lx, ly)
            } else {
                calc_position(
                    &config.location_position,
                    target_w as i32,
                    target_h as i32,
                    tw as i32,
                    th as i32,
                    config.location_offset,
                )
            }
        } else {
            calc_position(
                &config.location_position,
                target_w as i32,
                target_h as i32,
                tw as i32,
                th as i32,
                config.location_offset,
            )
        };

        draw_text_mut(
            &mut canvas,
            Rgba([0, 0, 0, 180]),
            x + shadow,
            y + shadow,
            scale,
            &font,
            location_str,
        );
        draw_text_mut(&mut canvas, Rgba([255, 255, 255, 255]), x, y, scale, &font, location_str);
    }

    Ok(image::DynamicImage::ImageRgba8(canvas).to_rgb8())
}

/// Calculate text position for a named position anchor.
/// Ported from BeReal-Recapper.py calc_pos().
fn calc_position(
    pos: &TextPosition,
    w: i32,
    h: i32,
    tw: i32,
    th: i32,
    offset: (i32, i32),
) -> (i32, i32) {
    let (ox, oy) = offset;
    let x = match pos {
        TextPosition::TopCenter | TextPosition::MiddleCenter | TextPosition::BottomCenter => {
            (w - tw) / 2
        }
        TextPosition::TopLeft | TextPosition::MiddleLeft | TextPosition::BottomLeft => 50,
        TextPosition::TopRight | TextPosition::MiddleRight | TextPosition::BottomRight => {
            w - tw - 50
        }
        _ => (w - tw) / 2,
    };
    let y = match pos {
        TextPosition::TopLeft | TextPosition::TopCenter | TextPosition::TopRight => 50,
        TextPosition::MiddleLeft | TextPosition::MiddleCenter | TextPosition::MiddleRight => {
            (h - th) / 2
        }
        TextPosition::BottomLeft | TextPosition::BottomCenter | TextPosition::BottomRight => {
            h - th - 50
        }
        _ => h - th - 50,
    };
    (x + ox, y + oy)
}

/// Measure rendered text width and height in pixels.
fn measure_text(font: &FontArc, scale: PxScale, text: &str) -> (u32, u32) {
    let scaled = font.as_scaled(scale);
    let width: f32 = text
        .chars()
        .map(|c| scaled.h_advance(scaled.scaled_glyph(c).id))
        .sum();
    let height = scaled.ascent() - scaled.descent();
    (width.ceil() as u32, height.ceil() as u32)
}
