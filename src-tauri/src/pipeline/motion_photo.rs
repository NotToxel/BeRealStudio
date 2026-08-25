use anyhow::{Context, Result};
use std::{io::Write, path::Path};

// Samsung Motion Photo constants (ported from process-photos.py)
const SAMSUNG_TAG_MOTION_PHOTO_DATA: &[u8] = &[0x00, 0x00, 0x30, 0x0a];
const SAMSUNG_TAG_MOTION_PHOTO_VERSION: &[u8] = &[0x00, 0x00, 0x31, 0x0a];
const SAMSUNG_SEFH_VERSION: i32 = 107;
const MOTION_PHOTO_VERSION_STR: &str = "mpv3";

/// Mux a BTS video into a JPEG to create a Samsung/Google Motion Photo.
/// Pure Rust — no exiftool dependency.
/// This matches the Python SamsungTags class exactly.
pub fn create_motion_photo(image_path: &Path, video_path: &Path) -> Result<()> {
    let video_bytes = std::fs::read(video_path)
        .with_context(|| format!("Cannot read video: {}", video_path.display()))?;
    let image_bytes = std::fs::read(image_path)
        .with_context(|| format!("Cannot read image: {}", image_path.display()))?;

    // Build Samsung video footer
    let footer = build_samsung_footer(&video_bytes);

    // Compute sizes for XMP metadata
    let image_size = image_bytes.len();
    let video_footer = &footer;
    let image_padding = get_image_padding(&video_bytes);
    let video_size = video_footer.len().saturating_sub(image_padding);

    // Inject XMP into the JPEG
    let xmp_content = build_motion_photo_xmp(image_size, video_size, image_padding);
    let jpeg_with_xmp = inject_xmp_into_jpeg(&image_bytes, &xmp_content)
        .with_context(|| "Failed to inject XMP into JPEG")?;

    // Write: JPEG-with-XMP + Samsung video footer
    let mut out = std::fs::File::create(image_path)
        .with_context(|| format!("Cannot write motion photo: {}", image_path.display()))?;
    out.write_all(&jpeg_with_xmp)?;
    out.write_all(video_footer)?;

    Ok(())
}

/// Build the Samsung SEF (Samsung EXIF Footer) binary blob.
/// Mirrors SamsungTags.video_footer() in the Python script.
fn build_samsung_footer(video_bytes: &[u8]) -> Vec<u8> {
    let version_data = MOTION_PHOTO_VERSION_STR.as_bytes();
    let mut tag_data: Vec<u8> = Vec::new();
    let mut tag_offsets: std::collections::HashMap<&str, i32> = std::collections::HashMap::new();
    let mut tag_lengths: std::collections::HashMap<&str, i32> = std::collections::HashMap::new();

    // Build in order: MotionPhoto_Data first, then MotionPhoto_Version
    let tags: &[(&str, &[u8], &[u8])] = &[
        ("MotionPhoto_Data", SAMSUNG_TAG_MOTION_PHOTO_DATA, video_bytes),
        ("MotionPhoto_Version", SAMSUNG_TAG_MOTION_PHOTO_VERSION, version_data),
    ];

    for (name, id_bytes, data) in tags {
        let name_bytes = name.as_bytes();
        let name_len = name_bytes.len() as i32;
        let mut tag_bytes: Vec<u8> = Vec::new();
        tag_bytes.extend_from_slice(id_bytes);
        tag_bytes.extend_from_slice(&name_len.to_le_bytes());
        tag_bytes.extend_from_slice(name_bytes);
        tag_bytes.extend_from_slice(data);

        let tag_len = tag_bytes.len() as i32;
        tag_data.extend_from_slice(&tag_bytes);
        tag_lengths.insert(name, tag_len);

        // Update offsets: each preceding tag's offset increases by this tag's length
        for (other_name, _, _) in tags {
            let cur = tag_offsets.entry(other_name).or_insert(0);
            *cur += tag_len;
            if *other_name == *name {
                break;
            }
        }
    }

    // Build SEFH header
    let mut sefh: Vec<u8> = Vec::new();
    sefh.extend_from_slice(b"SEFH");
    sefh.extend_from_slice(&SAMSUNG_SEFH_VERSION.to_le_bytes());
    sefh.extend_from_slice(&(tags.len() as i32).to_le_bytes());

    for (name, id_bytes, _) in tags {
        let offset = *tag_offsets.get(name).unwrap_or(&0);
        let length = *tag_lengths.get(name).unwrap_or(&0);
        sefh.extend_from_slice(id_bytes);
        sefh.extend_from_slice(&offset.to_le_bytes());
        sefh.extend_from_slice(&length.to_le_bytes());
    }

    let sefh_len = sefh.len() as i32;
    sefh.extend_from_slice(&sefh_len.to_le_bytes());
    sefh.extend_from_slice(b"SEFT");

    let mut result = tag_data;
    result.extend_from_slice(&sefh);
    result
}

/// Get the image padding (bytes before the video data in the footer).
/// Mirrors SamsungTags.get_image_padding().
fn get_image_padding(video_bytes: &[u8]) -> usize {
    let version_data = MOTION_PHOTO_VERSION_STR.as_bytes();
    let mut size = 0usize;

    let tags: &[(&str, &[u8], usize)] = &[
        ("MotionPhoto_Data", SAMSUNG_TAG_MOTION_PHOTO_DATA, video_bytes.len()),
        ("MotionPhoto_Version", SAMSUNG_TAG_MOTION_PHOTO_VERSION, version_data.len()),
    ];

    for (name, id_bytes, data_len) in tags {
        size += id_bytes.len();
        size += 4; // name length field (i32)
        size += name.len();
        if *name == "MotionPhoto_Data" {
            return size;
        }
        size += data_len;
    }
    0
}

/// Build the GCamera XMP for a Motion Photo.
fn build_motion_photo_xmp(_image_size: usize, video_size: usize, image_padding: usize) -> String {
    format!(
        r#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?><x:xmpmeta xmlns:x="adobe:ns:meta/" x:xmptk="Adobe XMP Core 5.1.0-jc003"><rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"><rdf:Description rdf:about="" xmlns:GCamera="http://ns.google.com/photos/1.0/camera/" xmlns:Container="http://ns.google.com/photos/1.0/container/" xmlns:Item="http://ns.google.com/photos/1.0/container/item/" GCamera:MotionPhoto="1" GCamera:MotionPhotoVersion="1" GCamera:MotionPhotoPresentationTimestampUs="-1"><Container:Directory><rdf:Seq><rdf:li rdf:parseType="Resource"><Container:Item Item:Mime="image/jpeg" Item:Semantic="Primary" Item:Length="0" Item:Padding="{padding}"/></rdf:li><rdf:li rdf:parseType="Resource"><Container:Item Item:Mime="video/mp4" Item:Semantic="MotionPhoto" Item:Length="{video_size}" Item:Padding="0"/></rdf:li></rdf:Seq></Container:Directory></rdf:Description></rdf:RDF></x:xmpmeta><?xpacket end="w"?>"#,
        padding = image_padding,
        video_size = video_size,
    )
}

/// Inject an XMP string into a JPEG's APP1 segment.
/// If an XMP APP1 already exists (starts with "http://ns.adobe.com/xap/"), it is replaced.
fn inject_xmp_into_jpeg(jpeg_bytes: &[u8], xmp: &str) -> Result<Vec<u8>> {
    // XMP in JPEG APP1: marker 0xFFE1 + length(2) + "http://ns.adobe.com/xap/1.0/\0" + xmp
    let xmp_header = b"http://ns.adobe.com/xap/1.0/\0";
    let xmp_payload_bytes = xmp.as_bytes();

    let new_segment_data: Vec<u8> = {
        let mut d = Vec::new();
        d.extend_from_slice(xmp_header);
        d.extend_from_slice(xmp_payload_bytes);
        d
    };

    if new_segment_data.len() + 2 > 65535 {
        anyhow::bail!("XMP metadata payload is too large for JPEG APP1 segment (exceeds 64KB)");
    }

    // APP1 length includes the 2-byte length field itself
    let segment_len = (new_segment_data.len() + 2) as u16;
    let mut new_app1: Vec<u8> = Vec::new();
    new_app1.push(0xFF);
    new_app1.push(0xE1);
    new_app1.extend_from_slice(&segment_len.to_be_bytes());
    new_app1.extend_from_slice(&new_segment_data);

    // Parse existing JPEG and replace/insert XMP APP1
    let mut out: Vec<u8> = Vec::with_capacity(jpeg_bytes.len() + new_app1.len());

    // Copy SOI (FF D8)
    if jpeg_bytes.len() < 2 || jpeg_bytes[0] != 0xFF || jpeg_bytes[1] != 0xD8 {
        anyhow::bail!("Not a valid JPEG (missing SOI)");
    }
    out.extend_from_slice(&jpeg_bytes[0..2]);
    let mut i = 2usize;

    let mut xmp_inserted = false;

    while i + 3 < jpeg_bytes.len() {
        if jpeg_bytes[i] != 0xFF {
            break;
        }
        let marker = jpeg_bytes[i + 1];
        let seg_len = u16::from_be_bytes([jpeg_bytes[i + 2], jpeg_bytes[i + 3]]) as usize;
        if seg_len < 2 {
            break;
        }
        let seg_end = (i + 2).saturating_add(seg_len);
        if seg_end > jpeg_bytes.len() {
            break;
        }

        if marker == 0xE1 {
            let payload = &jpeg_bytes[i + 4..seg_end];
            if payload.starts_with(xmp_header) {
                // Replace existing XMP APP1
                out.extend_from_slice(&new_app1);
                xmp_inserted = true;
                i = seg_end;
                continue;
            }
            // Keep non-XMP APP1 (e.g., EXIF)
            out.extend_from_slice(&jpeg_bytes[i..seg_end]);
            // Insert XMP right after first APP1 if not yet inserted
            if !xmp_inserted {
                out.extend_from_slice(&new_app1);
                xmp_inserted = true;
            }
        } else {
            // If we reach a non-APP segment and XMP not yet inserted, insert now
            if !xmp_inserted {
                out.extend_from_slice(&new_app1);
                xmp_inserted = true;
            }
            out.extend_from_slice(&jpeg_bytes[i..seg_end]);
        }
        i = seg_end;
    }

    if !xmp_inserted {
        out.extend_from_slice(&new_app1);
    }

    // Copy remainder (SOS + compressed data)
    out.extend_from_slice(&jpeg_bytes[i..]);

    Ok(out)
}
