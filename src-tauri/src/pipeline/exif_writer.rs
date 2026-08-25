use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use img_parts::{jpeg::Jpeg, ImageEXIF};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::pipeline::types::Location;

/// Detect ExifTool executable in PATH, application bundle, or common system paths.
pub fn detect_exiftool() -> Option<PathBuf> {
    // 1. Query system PATH for absolute binary location
    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = Command::new("where.exe").arg("exiftool").output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(first_line) = stdout.lines().next() {
                    let p = PathBuf::from(first_line.trim());
                    if p.exists() {
                        return Some(p);
                    }
                }
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(output) = Command::new("which").arg("exiftool").output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(first_line) = stdout.lines().next() {
                    let p = PathBuf::from(first_line.trim());
                    if p.exists() {
                        return Some(p);
                    }
                }
            }
        }
    }

    // 2. Check known standard Windows installation directories
    #[cfg(target_os = "windows")]
    {
        let candidates = [
            "E:\\Scripts\\exiftool.exe",
            "C:\\Program Files\\exiftool\\exiftool.exe",
            "C:\\Program Files (x86)\\exiftool\\exiftool.exe",
            "C:\\exiftool\\exiftool.exe",
            "C:\\Windows\\exiftool.exe",
        ];
        for c in candidates {
            let p = PathBuf::from(c);
            if p.exists() {
                return Some(p);
            }
        }
    }
    // 3. Check known Unix / macOS paths
    #[cfg(not(target_os = "windows"))]
    {
        let candidates = [
            "/usr/local/bin/exiftool",
            "/opt/homebrew/bin/exiftool",
            "/usr/bin/exiftool",
        ];
        for c in candidates {
            let p = PathBuf::from(c);
            if p.exists() {
                return Some(p);
            }
        }
    }

    // 4. Fallback check if directly runnable via PATH
    if let Ok(output) = Command::new("exiftool").arg("-ver").output() {
        if output.status.success() {
            return Some(PathBuf::from("exiftool"));
        }
    }

    None
}

/// Batch extract GPS coordinates (lat, lon) for a list of image files using ExifTool in a single process.
pub fn extract_gps_batch(
    exiftool_path: &Path,
    paths: &[PathBuf],
) -> std::collections::HashMap<PathBuf, (f64, f64)> {
    let mut map = std::collections::HashMap::new();
    if paths.is_empty() {
        return map;
    }

    let temp_file = std::env::temp_dir().join(format!("bereal_gps_{}.args", uuid::Uuid::new_v4()));
    let file_content = paths
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("\n");

    if std::fs::write(&temp_file, file_content).is_err() {
        return map;
    }

    let output = Command::new(exiftool_path)
        .arg("-@")
        .arg(&temp_file)
        .arg("-GPSLatitude")
        .arg("-GPSLongitude")
        .arg("-n")
        .arg("-j")
        .arg("-q")
        .output();

    let _ = std::fs::remove_file(&temp_file);

    if let Ok(out) = output {
        if out.status.success() {
            if let Ok(json_arr) = serde_json::from_slice::<Vec<serde_json::Value>>(&out.stdout) {
                for item in json_arr {
                    if let (Some(src), Some(lat), Some(lon)) = (
                        item["SourceFile"].as_str(),
                        item["GPSLatitude"].as_f64(),
                        item["GPSLongitude"].as_f64(),
                    ) {
                        map.insert(PathBuf::from(src), (lat, lon));
                    }
                }
            }
        }
    }

    map
}

/// Write comprehensive metadata using ExifTool with 100% Windows Explorer, macOS, and Cloud Photo parity.
pub fn write_metadata_exiftool(
    exiftool_path: &Path,
    path: &Path,
    datetime: &DateTime<Utc>,
    location: Option<&Location>,
    caption: Option<&str>,
    is_video: bool,
) -> Result<()> {
    let date_str = datetime.format("%Y:%m:%d %H:%M:%S").to_string();
    let date_sub_str = datetime.format("%Y:%m:%d %H:%M:%S%.3f").to_string();
    let tz_str = datetime.format("%:z").to_string();

    let mut cmd = Command::new(exiftool_path);
    cmd.arg("-overwrite_original")
        .arg("-m")
        .arg(format!("-AllDates={}", date_str))
        .arg(format!("-DateTimeOriginal={}", date_sub_str))
        .arg(format!("-CreateDate={}", date_sub_str))
        .arg(format!("-ModifyDate={}", date_sub_str))
        .arg(format!("-OffsetTime={}", tz_str))
        .arg(format!("-OffsetTimeOriginal={}", tz_str))
        .arg(format!("-OffsetTimeDigitized={}", tz_str))
        .arg(format!("-FileCreateDate={}", date_str))
        .arg(format!("-FileModifyDate={}", date_str));

    if is_video {
        cmd.arg(format!("-QuickTime:CreateDate={}", date_str))
            .arg(format!("-QuickTime:ModifyDate={}", date_str))
            .arg(format!("-TrackCreateDate={}", date_str))
            .arg(format!("-MediaCreateDate={}", date_str));
    }

    if let Some(loc) = location {
        let lat = loc.latitude;
        let lon = loc.longitude;
        let lat_ref = if lat >= 0.0 { "N" } else { "S" };
        let lon_ref = if lon >= 0.0 { "E" } else { "W" };

        cmd.arg(format!("-GPSLatitude={}", lat.abs()))
            .arg(format!("-GPSLatitudeRef={}", lat_ref))
            .arg(format!("-GPSLongitude={}", lon.abs()))
            .arg(format!("-GPSLongitudeRef={}", lon_ref))
            .arg(format!("-GPSPosition={}, {}", lat, lon));
    }

    if let Some(c) = caption {
        if !c.trim().is_empty() {
            cmd.arg(format!("-ImageDescription={}", c))
                .arg(format!("-Caption-Abstract={}", c))
                .arg(format!("-Description={}", c))
                .arg(format!("-Title={}", c))
                .arg(format!("-XPComment={}", c))
                .arg(format!("-XPSubject={}", c))
                .arg(format!("-UserComment={}", c));
        }
    }

    cmd.arg("-Source=BeReal app")
        .arg("-Software=BeReal Studio")
        .arg("-CreatorTool=BeReal Studio")
        .arg("-OriginatingProgram=BeReal Studio");

    cmd.arg(path);

    let output = cmd.output().with_context(|| format!("Failed to run ExifTool on {}", path.display()))?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        log::warn!("ExifTool note on {}: {}", path.display(), err);
    }

    // Explicitly synchronize filesystem creation/modification timestamps
    let ft = filetime::FileTime::from_unix_time(datetime.timestamp(), 0);
    let _ = filetime::set_file_times(path, ft, ft);

    Ok(())
}

/// Write combined EXIF and IPTC metadata to a file in-place and synchronize filesystem timestamps.
/// Prioritizes ExifTool for 100% metadata fidelity, falling back to native byte-level segment injection.
pub fn write_metadata(
    path: &Path,
    datetime: &DateTime<Utc>,
    location: Option<&Location>,
    caption: Option<&str>,
) -> Result<()> {
    let is_video = path
        .extension()
        .map(|e| e.eq_ignore_ascii_case("mp4") || e.eq_ignore_ascii_case("mov"))
        .unwrap_or(false);

    if let Some(exiftool) = detect_exiftool() {
        return write_metadata_exiftool(&exiftool, path, datetime, location, caption, is_video);
    }

    if is_video {
        let ft = filetime::FileTime::from_unix_time(datetime.timestamp(), 0);
        let _ = filetime::set_file_times(path, ft, ft);
        return Ok(());
    }

    let data = std::fs::read(path)
        .with_context(|| format!("Cannot read {}", path.display()))?;
    let mut jpeg = Jpeg::from_bytes(data.into())
        .with_context(|| "Not a valid JPEG file")?;

    // 1. Build and set EXIF APP1
    let exif_bytes = build_exif_bytes(datetime, location, caption);
    jpeg.set_exif(Some(exif_bytes.into()));

    // 2. Build and inject IPTC APP13
    let iptc_bytes = build_iptc_bytes(caption, "BeReal app", "BeReal Studio");
    let segments = jpeg.segments_mut();
    segments.retain(|s| s.marker() != img_parts::jpeg::markers::APP13);
    let app13 = img_parts::jpeg::JpegSegment::new_with_contents(
        img_parts::jpeg::markers::APP13,
        iptc_bytes.into(),
    );
    let insert_pos = segments
        .iter()
        .position(|s| s.marker() == img_parts::jpeg::markers::APP1)
        .map(|i| i + 1)
        .unwrap_or(1);
    segments.insert(insert_pos, app13);

    // 3. Write out modified JPEG
    let out = std::fs::File::create(path)
        .with_context(|| format!("Cannot write {}", path.display()))?;
    jpeg.encoder().write_to(out)?;

    // 4. Synchronize filesystem creation & modification timestamps with the photo capture time
    let ft = filetime::FileTime::from_unix_time(datetime.timestamp(), 0);
    let _ = filetime::set_file_times(path, ft, ft);

    Ok(())
}

/// Write EXIF metadata (date, GPS, description) to a JPEG file in-place.
pub fn write_exif(
    path: &Path,
    datetime: &DateTime<Utc>,
    location: Option<&Location>,
    description: Option<&str>,
) -> Result<()> {
    let data = std::fs::read(path)
        .with_context(|| format!("Cannot read {}", path.display()))?;
    let mut jpeg = Jpeg::from_bytes(data.into())
        .with_context(|| "Not a valid JPEG file")?;

    let exif_bytes = build_exif_bytes(datetime, location, description);
    jpeg.set_exif(Some(exif_bytes.into()));

    let out = std::fs::File::create(path)
        .with_context(|| format!("Cannot write {}", path.display()))?;
    jpeg.encoder().write_to(out)?;

    let ft = filetime::FileTime::from_unix_time(datetime.timestamp(), 0);
    let _ = filetime::set_file_times(path, ft, ft);

    Ok(())
}

/// Write IPTC metadata (caption, source info) to a JPEG file in-place.
pub fn write_iptc(path: &Path, caption: Option<&str>, source: &str, program: &str) -> Result<()> {
    let iptc_bytes = build_iptc_bytes(caption, source, program);

    let data = std::fs::read(path)
        .with_context(|| format!("Cannot read {}", path.display()))?;
    let mut jpeg = Jpeg::from_bytes(data.into())
        .with_context(|| "Not a valid JPEG")?;

    let segments = jpeg.segments_mut();
    segments.retain(|s| s.marker() != img_parts::jpeg::markers::APP13);
    let app13 = img_parts::jpeg::JpegSegment::new_with_contents(
        img_parts::jpeg::markers::APP13,
        iptc_bytes.into(),
    );
    let insert_pos = segments
        .iter()
        .position(|s| s.marker() == img_parts::jpeg::markers::APP1)
        .map(|i| i + 1)
        .unwrap_or(1);
    segments.insert(insert_pos, app13);

    let out = std::fs::File::create(path)?;
    jpeg.encoder().write_to(out)?;
    Ok(())
}

/// Build an IPTC-IIM APP13 segment payload.
/// Format: "Photoshop 3.0\0" + 8BIM resource block + IPTC records
fn build_iptc_bytes(caption: Option<&str>, source: &str, program: &str) -> Vec<u8> {
    let mut iptc_records: Vec<u8> = Vec::new();

    // Each IPTC record: 0x1C + dataset_num + record_num + length (2 bytes BE) + data
    let mut write_record = |dataset: u8, record: u8, data: &[u8]| {
        let safe_data = if data.len() > 65530 { &data[..65530] } else { data };
        iptc_records.push(0x1C);
        iptc_records.push(dataset);
        iptc_records.push(record);
        let len = safe_data.len() as u16;
        iptc_records.extend_from_slice(&len.to_be_bytes());
        iptc_records.extend_from_slice(safe_data);
    };

    if let Some(cap) = caption {
        if !cap.is_empty() {
            write_record(2, 120, cap.as_bytes()); // Caption/Abstract
        }
    }
    if !source.is_empty() {
        write_record(2, 115, source.as_bytes()); // Source
    }
    if !program.is_empty() {
        write_record(2, 65, program.as_bytes()); // Originating Program
    }

    // Wrap in 8BIM IPTC resource block
    let mut block: Vec<u8> = Vec::new();
    block.extend_from_slice(b"8BIM");
    block.extend_from_slice(&0x0404u16.to_be_bytes()); // IPTC-NAA resource
    block.extend_from_slice(b"\0\0"); // Pascal string (empty)
    let records_len = iptc_records.len() as u32;
    block.extend_from_slice(&records_len.to_be_bytes());
    block.extend_from_slice(&iptc_records);
    if iptc_records.len() % 2 != 0 {
        block.push(0); // Pad to even
    }

    // Full APP13 payload
    let mut payload: Vec<u8> = Vec::new();
    payload.extend_from_slice(b"Photoshop 3.0\0");
    payload.extend(block);
    payload
}

// ─── Compliant EXIF TIFF Builder ─────────────────────────────────────────────

struct TiffEntry {
    tag: u16,
    typ: u16, // 2 = ASCII, 4 = LONG, 5 = RATIONAL, 7 = UNDEFINED
    data: Vec<u8>,
}

struct ExifWriter {
    ifd0_entries: Vec<TiffEntry>,
    exif_entries: Vec<TiffEntry>,
    gps_lat: Option<(char, u32, u32, u32, u32, u32, u32)>, // (ref, deg_n, deg_d, min_n, min_d, sec_n, sec_d)
    gps_lon: Option<(char, u32, u32, u32, u32, u32, u32)>,
}

impl ExifWriter {
    fn new() -> Self {
        Self {
            ifd0_entries: Vec::new(),
            exif_entries: Vec::new(),
            gps_lat: None,
            gps_lon: None,
        }
    }

    fn add_ascii_ifd0(&mut self, tag: u16, value: &str) {
        let mut data = value.as_bytes().to_vec();
        data.push(0);
        self.ifd0_entries.push(TiffEntry { tag, typ: 2, data });
    }

    fn add_ascii_exif(&mut self, tag: u16, value: &str) {
        let mut data = value.as_bytes().to_vec();
        data.push(0);
        self.exif_entries.push(TiffEntry { tag, typ: 2, data });
    }

    fn add_user_comment(&mut self, text: &str) {
        let mut data = Vec::with_capacity(8 + text.len());
        data.extend_from_slice(b"ASCII\0\0\0");
        data.extend_from_slice(text.as_bytes());
        self.exif_entries.push(TiffEntry { tag: 0x9286, typ: 7, data });
    }

    fn add_gps(&mut self, lat: f64, lon: f64) {
        let lat_ref = if lat >= 0.0 { 'N' } else { 'S' };
        let lon_ref = if lon >= 0.0 { 'E' } else { 'W' };

        let to_dms = |coord: f64| -> (u32, u32, u32, u32, u32, u32) {
            let abs = coord.abs();
            let deg = abs.floor() as u32;
            let min_float = (abs - deg as f64) * 60.0;
            let min = min_float.floor() as u32;
            let sec_float = (min_float - min as f64) * 60.0;
            let sec = (sec_float * 1000.0).round() as u32;
            (deg, 1, min, 1, sec, 1000)
        };

        let (ld, ld_d, lm, lm_d, ls, ls_d) = to_dms(lat);
        self.gps_lat = Some((lat_ref, ld, ld_d, lm, lm_d, ls, ls_d));

        let (od, od_d, om, om_d, os, os_d) = to_dms(lon);
        self.gps_lon = Some((lon_ref, od, od_d, om, om_d, os, os_d));
    }

    fn build(&mut self) -> Vec<u8> {
        // Standard software identifier
        self.add_ascii_ifd0(0x0131, "BeReal Studio");

        // Sort entries by tag ID ascending (TIFF standard compliance)
        self.ifd0_entries.sort_by_key(|e| e.tag);
        self.exif_entries.sort_by_key(|e| e.tag);

        let mut buf = Vec::new();
        // 1. TIFF Header (Little-Endian "II")
        buf.extend_from_slice(b"II");
        buf.extend_from_slice(&42u16.to_le_bytes());
        buf.extend_from_slice(&8u32.to_le_bytes()); // IFD0 starts at offset 8

        let has_gps = self.gps_lat.is_some() && self.gps_lon.is_some();

        // 2. Count entries for IFD0
        // Entries in IFD0 = user entries + ExifIFDPointer (0x8769) + optional GPSInfoIFDPointer (0x8825)
        let ifd0_count = (self.ifd0_entries.len() + 1 + if has_gps { 1 } else { 0 }) as u16;

        // Size calculation for offsets
        let ifd0_size = 2 + (ifd0_count as u32 * 12) + 4;
        let exif_sub_ifd_offset = 8 + ifd0_size;
        let exif_sub_ifd_count = self.exif_entries.len() as u16;
        let exif_sub_ifd_size = 2 + (exif_sub_ifd_count as u32 * 12) + 4;
        let gps_ifd_offset = exif_sub_ifd_offset + exif_sub_ifd_size;
        let gps_ifd_count = if has_gps { 4u16 } else { 0u16 };
        let gps_ifd_size = if has_gps { 2 + (gps_ifd_count as u32 * 12) + 4 } else { 0u32 };

        let mut data_offset = if has_gps {
            gps_ifd_offset + gps_ifd_size
        } else {
            exif_sub_ifd_offset + exif_sub_ifd_size
        };

        let mut data_pool = Vec::new();

        // Helper to encode a single 12-byte TIFF entry
        let mut encode_entry = |tag: u16, typ: u16, data: &[u8]| -> [u8; 12] {
            let mut entry = [0u8; 12];
            entry[0..2].copy_from_slice(&tag.to_le_bytes());
            entry[2..4].copy_from_slice(&typ.to_le_bytes());

            let count = match typ {
                2 | 7 => data.len() as u32,
                4 => (data.len() / 4) as u32,
                5 => (data.len() / 8) as u32,
                _ => data.len() as u32,
            };
            entry[4..8].copy_from_slice(&count.to_le_bytes());

            if data.len() <= 4 {
                entry[8..8 + data.len()].copy_from_slice(data);
            } else {
                let offset = data_offset;
                entry[8..12].copy_from_slice(&offset.to_le_bytes());
                data_offset += data.len() as u32;
                data_pool.extend_from_slice(data);
            }
            entry
        };

        // Write IFD0
        buf.extend_from_slice(&ifd0_count.to_le_bytes());
        for entry in &self.ifd0_entries {
            buf.extend_from_slice(&encode_entry(entry.tag, entry.typ, &entry.data));
        }
        // ExifIFD pointer (0x8769)
        buf.extend_from_slice(&encode_entry(0x8769, 4, &exif_sub_ifd_offset.to_le_bytes()));
        if has_gps {
            // GPSInfo pointer (0x8825)
            buf.extend_from_slice(&encode_entry(0x8825, 4, &gps_ifd_offset.to_le_bytes()));
        }
        buf.extend_from_slice(&0u32.to_le_bytes()); // Next IFD = 0

        // Write ExifSubIFD
        buf.extend_from_slice(&exif_sub_ifd_count.to_le_bytes());
        for entry in &self.exif_entries {
            buf.extend_from_slice(&encode_entry(entry.tag, entry.typ, &entry.data));
        }
        buf.extend_from_slice(&0u32.to_le_bytes()); // Next IFD = 0

        // Write GPS IFD (if available)
        if let (Some((lat_ref, ld, ld_d, lm, lm_d, ls, ls_d)), Some((lon_ref, od, od_d, om, om_d, os, os_d))) =
            (self.gps_lat, self.gps_lon)
        {
            buf.extend_from_slice(&gps_ifd_count.to_le_bytes());
            // GPSLatitudeRef (0x0001)
            let lat_ref_bytes = [lat_ref as u8, 0];
            buf.extend_from_slice(&encode_entry(0x0001, 2, &lat_ref_bytes));

            // GPSLatitude (0x0002) - 3 Rationals (24 bytes)
            let mut lat_bytes = Vec::with_capacity(24);
            lat_bytes.extend_from_slice(&ld.to_le_bytes());
            lat_bytes.extend_from_slice(&ld_d.to_le_bytes());
            lat_bytes.extend_from_slice(&lm.to_le_bytes());
            lat_bytes.extend_from_slice(&lm_d.to_le_bytes());
            lat_bytes.extend_from_slice(&ls.to_le_bytes());
            lat_bytes.extend_from_slice(&ls_d.to_le_bytes());
            buf.extend_from_slice(&encode_entry(0x0002, 5, &lat_bytes));

            // GPSLongitudeRef (0x0003)
            let lon_ref_bytes = [lon_ref as u8, 0];
            buf.extend_from_slice(&encode_entry(0x0003, 2, &lon_ref_bytes));

            // GPSLongitude (0x0004) - 3 Rationals (24 bytes)
            let mut lon_bytes = Vec::with_capacity(24);
            lon_bytes.extend_from_slice(&od.to_le_bytes());
            lon_bytes.extend_from_slice(&od_d.to_le_bytes());
            lon_bytes.extend_from_slice(&om.to_le_bytes());
            lon_bytes.extend_from_slice(&om_d.to_le_bytes());
            lon_bytes.extend_from_slice(&os.to_le_bytes());
            lon_bytes.extend_from_slice(&os_d.to_le_bytes());
            buf.extend_from_slice(&encode_entry(0x0004, 5, &lon_bytes));

            buf.extend_from_slice(&0u32.to_le_bytes()); // Next IFD = 0
        }

        // Append Data Pool
        buf.extend_from_slice(&data_pool);
        buf
    }
}

fn build_exif_bytes(
    datetime: &DateTime<Utc>,
    location: Option<&Location>,
    description: Option<&str>,
) -> Vec<u8> {
    let mut writer = ExifWriter::new();
    let date_str = datetime.format("%Y:%m:%d %H:%M:%S").to_string();

    // Standard EXIF Date Tags
    writer.add_ascii_ifd0(0x0132, &date_str); // DateTime
    writer.add_ascii_exif(0x9003, &date_str); // DateTimeOriginal
    writer.add_ascii_exif(0x9004, &date_str); // CreateDate

    // Captions & Descriptions
    if let Some(desc) = description {
        if !desc.is_empty() {
            writer.add_ascii_ifd0(0x010E, desc); // ImageDescription
            writer.add_user_comment(desc);       // UserComment
        }
    }

    // Geocoded Coordinates
    if let Some(loc) = location {
        if loc.latitude.is_finite() && loc.longitude.is_finite() {
            writer.add_gps(loc.latitude.clamp(-90.0, 90.0), loc.longitude.clamp(-180.0, 180.0));
        }
    }

    let mut payload = b"Exif\0\0".to_vec();
    payload.extend(writer.build());
    payload
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_build_iptc_has_photoshop_header() {
        let bytes = build_iptc_bytes(Some("Test caption"), "BeReal app", "BeReal Studio");
        assert!(bytes.starts_with(b"Photoshop 3.0\0"));
    }

    #[test]
    fn test_build_exif_has_header_and_tags() {
        let dt = Utc.with_ymd_and_hms(2024, 3, 15, 12, 0, 0).unwrap();
        let loc = Location { latitude: 48.8566, longitude: 2.3522 };
        let bytes = build_exif_bytes(&dt, Some(&loc), Some("Sunset in Paris"));
        assert!(bytes.starts_with(b"Exif\0\0"));
        assert!(bytes.len() > 100);
    }
}
