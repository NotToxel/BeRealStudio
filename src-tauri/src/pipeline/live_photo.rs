use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crate::pipeline::{
    exif_writer,
    types::Location,
    video_ops,
};

/// Create an Apple Photos compatible Live Photo pair (.jpg + .mov)
/// with matching Apple Content Identifier UUIDs in the MakerNote and QuickTime container.
pub fn create_apple_live_photo_pair(
    composite_image_path: &Path,
    bts_video_path: &Path,
    output_dir: &Path,
    base_name: &str,
    datetime: &DateTime<Utc>,
    location: Option<&Location>,
    caption: Option<&str>,
) -> Result<(PathBuf, PathBuf)> {
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir)
            .with_context(|| format!("Failed to create live photo directory: {}", output_dir.display()))?;
    }

    // 1. Generate unique Apple Asset Identifier (UUID v4 in uppercase)
    let asset_id = uuid::Uuid::new_v4().to_string().to_uppercase();

    // 2. Prepare destination paths (.jpg and .mov with matching stem)
    let dest_jpg = output_dir.join(format!("{}.jpg", base_name));
    let dest_mov = output_dir.join(format!("{}.mov", base_name));

    // 3. Copy image to live_photos destination
    std::fs::copy(composite_image_path, &dest_jpg)
        .with_context(|| format!("Failed to copy composite image to {}", dest_jpg.display()))?;

    // 4. Inject Apple MakerNote Tag 17 (ContentIdentifier) into the JPEG
    let still_result = exif_writer::write_metadata_with_apple_id(
        &dest_jpg,
        datetime,
        location,
        caption,
        Some(&asset_id),
    ).with_context(|| format!("Failed to write Apple Live Photo metadata to {}", dest_jpg.display()));
    if let Err(error) = still_result {
        let _ = std::fs::remove_file(&dest_jpg);
        return Err(error);
    }

    // 5. Encode a broadly supported QuickTime movie with the matching identifier.
    // Copying the source codec can leave a valid MOV that Photos cannot import.
    let movie_result = (|| -> Result<()> {
        let ffmpeg = video_ops::detect_ffmpeg().context("FFmpeg is required for Apple Live Photos")?;
        let output = video_ops::silent_command(&ffmpeg)
            .arg("-i").arg(bts_video_path)
            .args(["-map", "0:v:0", "-map", "0:a?"])
            .args(["-c:v", "libx264", "-pix_fmt", "yuv420p", "-crf", "18", "-preset", "medium"])
            .args(["-vf", "scale=trunc(iw/2)*2:trunc(ih/2)*2"])
            .args(["-c:a", "aac", "-b:a", "128k"])
            .args(["-brand", "qt  "])
            .args(["-movflags", "use_metadata_tags"])
            // MP4 source tags otherwise override the MOV muxer's QuickTime brand.
            .args(["-metadata:g", "major_brand=", "-metadata:g", "compatible_brands=", "-metadata:g", "minor_version="])
            .arg("-metadata:g")
            .arg(format!("com.apple.quicktime.content.identifier={asset_id}"))
            .args(["-f", "mov", "-y"])
            .arg(&dest_mov)
            .output()
            .with_context(|| format!("Failed to run FFmpeg for {}", bts_video_path.display()))?;
        if !output.status.success() {
            anyhow::bail!("FFmpeg could not create the Live Photo MOV: {}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(())
    })();
    if let Err(error) = movie_result {
        let _ = std::fs::remove_file(&dest_jpg);
        let _ = std::fs::remove_file(&dest_mov);
        return Err(error);
    }

    // FFmpeg puts mdta under moov/udta/meta. AVFoundation's asset.metadata()
    // did not find that identifier on macOS 26. The native makelive result also
    // has a moov/meta atom. Write that atom in the same format and position.
    if let Err(error) = add_movie_level_content_identifier(&dest_mov, &asset_id) {
        let _ = std::fs::remove_file(&dest_jpg);
        let _ = std::fs::remove_file(&dest_mov);
        return Err(error);
    }

    // 6. Synchronize filesystem timestamps on both files
    let ft = filetime::FileTime::from_unix_time(datetime.timestamp(), 0);
    let _ = filetime::set_file_times(&dest_jpg, ft, ft);
    let _ = filetime::set_file_times(&dest_mov, ft, ft);

    Ok((dest_jpg, dest_mov))
}

fn qt_atom(kind: &[u8; 4], payload: &[u8]) -> Result<Vec<u8>> {
    let size = u32::try_from(payload.len() + 8).context("QuickTime metadata atom is too large")?;
    let mut atom = Vec::with_capacity(size as usize);
    atom.extend_from_slice(&size.to_be_bytes());
    atom.extend_from_slice(kind);
    atom.extend_from_slice(payload);
    Ok(atom)
}

fn content_identifier_atom(asset_id: &str) -> Result<Vec<u8>> {
    anyhow::ensure!(asset_id.is_ascii() && asset_id.len() == 36, "Invalid Apple asset identifier");
    let mut handler = vec![0; 8];
    handler.extend_from_slice(b"mdta");
    handler.extend_from_slice(&[0; 14]);
    let handler = qt_atom(b"hdlr", &handler)?;

    let key_entry = qt_atom(b"mdta", b"com.apple.quicktime.content.identifier")?;
    let mut keys_data = vec![0, 0, 0, 0, 0, 0, 0, 1];
    keys_data.extend_from_slice(&key_entry);
    let key = qt_atom(b"keys", &keys_data)?;

    let mut data = vec![0, 0, 0, 1, 0, 0, 0, 0];
    data.extend_from_slice(asset_id.as_bytes());
    let data = qt_atom(b"data", &data)?;
    let item = qt_atom(&[0, 0, 0, 1], &data)?;
    let items = qt_atom(b"ilst", &item)?;

    let mut metadata = Vec::with_capacity(handler.len() + key.len() + items.len());
    metadata.extend_from_slice(&handler);
    metadata.extend_from_slice(&key);
    metadata.extend_from_slice(&items);
    qt_atom(b"meta", &metadata)
}

fn add_movie_level_content_identifier(path: &Path, asset_id: &str) -> Result<()> {
    let mut movie = std::fs::OpenOptions::new().read(true).write(true).open(path)?;
    let file_len = movie.metadata()?.len();
    let mut cursor = 0u64;
    let mut moov = None;
    while cursor + 8 <= file_len {
        movie.seek(SeekFrom::Start(cursor))?;
        let mut header = [0u8; 8];
        movie.read_exact(&mut header)?;
        let size = u32::from_be_bytes(header[..4].try_into()?) as u64;
        anyhow::ensure!(size >= 8 && cursor + size <= file_len, "Invalid QuickTime atom size");
        if &header[4..] == b"moov" {
            moov = Some((cursor, size));
            break;
        }
        cursor += size;
    }
    let (moov_start, moov_size) = moov.context("QuickTime movie header is missing")?;
    anyhow::ensure!(moov_start + moov_size == file_len, "QuickTime movie header must be at end of file");
    let body_len = usize::try_from(moov_size - 8)?;
    let mut body = vec![0; body_len];
    movie.seek(SeekFrom::Start(moov_start + 8))?;
    movie.read_exact(&mut body)?;

    let mut insert_at = body.len();
    let mut position = 0usize;
    while position + 8 <= body.len() {
        let size = u32::from_be_bytes(body[position..position + 4].try_into()?) as usize;
        anyhow::ensure!(size >= 8 && position + size <= body.len(), "Invalid movie child atom size");
        if &body[position + 4..position + 8] == b"udta" {
            insert_at = position;
            break;
        }
        position += size;
    }
    anyhow::ensure!(position == body.len() || insert_at < body.len(), "Invalid movie child atoms");
    let metadata = content_identifier_atom(asset_id)?;
    body.splice(insert_at..insert_at, metadata);
    let new_moov = qt_atom(b"moov", &body)?;
    movie.seek(SeekFrom::Start(moov_start))?;
    movie.write_all(&new_moov)?;
    movie.set_len(moov_start + new_moov.len() as u64)?;
    Ok(())
}

/// Create a .pvt directory using the package layout
/// emitted by makelive: paired media files and metadata.plist.
pub fn create_apple_live_photo_package(
    composite_image_path: &Path,
    bts_video_path: &Path,
    package_path: &Path,
    datetime: &DateTime<Utc>,
    location: Option<&Location>,
    caption: Option<&str>,
) -> Result<PathBuf> {
    anyhow::ensure!(
        package_path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("pvt")),
        "Live Photo package path must end in .pvt"
    );
    let base_name = package_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .context("Live Photo package needs a valid filename")?;
    std::fs::create_dir(package_path)
        .with_context(|| format!("Cannot create Live Photo package {}", package_path.display()))?;

    let result = (|| -> Result<()> {
        create_apple_live_photo_pair(
            composite_image_path,
            bts_video_path,
            package_path,
            base_name,
            datetime,
            location,
            caption,
        )?;
        std::fs::write(
            package_path.join("metadata.plist"),
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
             <plist version=\"1.0\"><dict><key>PFVideoComplementMetadataVersionKey</key><string>1</string></dict></plist>\n",
        )
        .with_context(|| format!("Cannot write metadata.plist in {}", package_path.display()))?;
        Ok(())
    })();

    if let Err(error) = result {
        let _ = std::fs::remove_dir_all(package_path);
        return Err(error);
    }
    Ok(package_path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn pvt_package_contains_parseable_pair_and_plist() {
        let Ok(ffmpeg) = video_ops::detect_ffmpeg() else { return };
        let Some(exiftool) = exif_writer::detect_exiftool() else { return };
        let root = std::env::temp_dir().join(format!("bereal-pvt-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&root).unwrap();
        let image_path = root.join("source.jpg");
        let video_path = root.join("source.mp4");
        image::RgbImage::new(2, 2).save(&image_path).unwrap();
        let video = video_ops::silent_command(ffmpeg)
            .args(["-hide_banner", "-loglevel", "error", "-f", "lavfi", "-i", "color=c=black:s=64x64:r=10"])
            .args(["-t", "1", "-c:v", "mpeg4", "-y"])
            .arg(&video_path)
            .output().unwrap();
        assert!(video.status.success(), "{}", String::from_utf8_lossy(&video.stderr));

        let package = root.join("memory.pvt");
        let dt = Utc.with_ymd_and_hms(2024, 3, 15, 12, 0, 0).unwrap();
        let location = Location { latitude: 51.5074, longitude: -0.1278 };
        create_apple_live_photo_package(&image_path, &video_path, &package, &dt, Some(&location), Some("A memory with caption")).unwrap();
        assert!(package.join("memory.jpg").is_file());
        assert!(package.join("memory.mov").is_file());
        let plist = std::fs::read_to_string(package.join("metadata.plist")).unwrap();
        assert!(plist.contains("PFVideoComplementMetadataVersionKey"));

        let read_tag = |file: &Path, tag: &str| -> String {
            let output = video_ops::silent_command(&exiftool)
                .args(["-s3", tag]).arg(file).output().unwrap();
            assert!(output.status.success());
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        };
        let still_id = read_tag(&package.join("memory.jpg"), "-MediaGroupUUID");
        let movie_id = read_tag(&package.join("memory.mov"), "-ContentIdentifier");
        assert!(!still_id.is_empty());
        assert_eq!(still_id, movie_id);
        let movie_bytes = std::fs::read(package.join("memory.mov")).unwrap();
        let native_metadata = content_identifier_atom(&movie_id).unwrap();
        assert!(
            movie_bytes.windows(native_metadata.len()).any(|window| window == native_metadata),
            "MOV must contain the movie-level identifier atom read by AVFoundation"
        );
        assert_eq!(read_tag(&package.join("memory.mov"), "-MajorBrand"), "Apple QuickTime (.MOV/QT)");
        assert_eq!(read_tag(&package.join("memory.mov"), "-CompressorID"), "avc1");
        let validation = video_ops::silent_command(&exiftool)
            .args(["-a", "-Validate", "-Warning", "-Error"])
            .arg(package.join("memory.jpg"))
            .output().unwrap();
        assert!(validation.status.success());
        let report = String::from_utf8_lossy(&validation.stdout);
        assert!(!report.contains("Warning") && !report.contains("Error"), "{report}");
        std::fs::remove_dir_all(root).unwrap();
    }
}
