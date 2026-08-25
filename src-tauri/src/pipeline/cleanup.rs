use anyhow::Result;
use std::path::Path;

use crate::pipeline::types::OutputFormat;

/// Remove intermediate WebP files from combined output folders
/// when the user has chosen JPEG or PNG as the output format.
pub fn cleanup_intermediates(output_dir: &Path, format: &OutputFormat) -> Result<()> {
    if *format == OutputFormat::WebP {
        return Ok(()); // Nothing to clean when output is WebP
    }
    if !output_dir.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(output_dir)?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("webp") {
            if let Err(e) = std::fs::remove_file(&path) {
                log::warn!("Could not delete {}: {}", path.display(), e);
            }
        }
    }
    Ok(())
}

/// Remove IPTC backup files (files ending with '~') from a directory.
pub fn remove_backup_files(dir: &Path) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)?.flatten() {
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.ends_with('~') {
                let _ = std::fs::remove_file(&path);
            }
        }
    }
    Ok(())
}
