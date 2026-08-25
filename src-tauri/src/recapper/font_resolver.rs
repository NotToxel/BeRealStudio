use ab_glyph::FontArc;
use anyhow::{Context, Result};
use std::path::Path;

use crate::pipeline::types::FontInfo;

pub struct BuiltinFont {
    pub id: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    pub bytes: &'static [u8],
}

// Curated modern built-in fonts
pub const BUILTIN_FONTS: &[BuiltinFont] = &[
    BuiltinFont {
        id: "inter",
        name: "Inter",
        category: "Modern Sans",
        bytes: include_bytes!("../assets/fonts/inter.ttf"),
    },
    BuiltinFont {
        id: "roboto",
        name: "Roboto (BeReal Classic)",
        category: "Punchy Sans",
        bytes: include_bytes!("../assets/fonts/roboto.ttf"),
    },
    BuiltinFont {
        id: "outfit",
        name: "Outfit",
        category: "Geometric Display",
        bytes: include_bytes!("../assets/fonts/outfit.ttf"),
    },
    BuiltinFont {
        id: "bebas",
        name: "Bebas Neue",
        category: "Bold Poster / TikTok",
        bytes: include_bytes!("../assets/fonts/bebas.ttf"),
    },
    BuiltinFont {
        id: "playfair",
        name: "Playfair Display",
        category: "Editorial Serif",
        bytes: include_bytes!("../assets/fonts/playfair.ttf"),
    },
    BuiltinFont {
        id: "jetbrains",
        name: "JetBrains Mono",
        category: "Tech Monospace",
        bytes: include_bytes!("../assets/fonts/jetbrains.ttf"),
    },
    BuiltinFont {
        id: "caveat",
        name: "Caveat",
        category: "Handwritten Journal",
        bytes: include_bytes!("../assets/fonts/caveat.ttf"),
    },
];

/// Return the curated list of in-built modern fonts.
pub fn list_system_fonts() -> Vec<FontInfo> {
    BUILTIN_FONTS
        .iter()
        .map(|f| FontInfo {
            family: f.name.to_string(),
            style: f.category.to_string(),
            path: f.id.to_string(),
        })
        .collect()
}

/// Load a font by its built-in ID ("inter", "outfit", etc.) or from a file path on disk.
pub fn load_font(font_id_or_path: &str) -> Result<FontArc> {
    let lower = font_id_or_path.to_lowercase();

    // Check built-in fonts by ID or name
    for f in BUILTIN_FONTS {
        if f.id == lower || f.name.to_lowercase() == lower {
            return FontArc::try_from_slice(f.bytes)
                .with_context(|| format!("Failed to parse embedded font: {}", f.name));
        }
    }

    // Try loading from file path if it exists on disk
    if Path::new(font_id_or_path).exists() {
        let bytes = std::fs::read(font_id_or_path)
            .with_context(|| format!("Cannot read font file: {}", font_id_or_path))?;
        if let Ok(font) = FontArc::try_from_vec(bytes) {
            return Ok(font);
        }
    }

    // Default fallback: Inter
    FontArc::try_from_slice(BUILTIN_FONTS[0].bytes).context("Failed to load default Inter font")
}
