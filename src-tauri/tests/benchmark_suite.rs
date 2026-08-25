use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use bereal_studio_lib::pipeline::{
    image_ops::{combine_pip, combine_side_by_side},
    parser::scan_archive,
};

/// Helper: build a synthetic BeReal GDPR zip archive with `n` simulated posts.
fn create_mock_bereal_zip(dir: &std::path::Path, n_posts: usize) -> PathBuf {
    let zip_path = dir.join(format!("bereal_mock_{}_posts.zip", n_posts));
    let file = std::fs::File::create(&zip_path).expect("failed to create zip file");
    let mut zip = ZipWriter::new(file);

    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(0o755);

    // 1. user.json
    zip.start_file("user.json", options).unwrap();
    let user_json = r#"{"username":"testuser","fullname":"Test User"}"#;
    zip.write_all(user_json.as_bytes()).unwrap();

    // 2. photos & posts.json
    let mut posts = Vec::new();

    // Create a 100x133 valid dummy JPEG for mock photos
    let dummy_jpeg = create_dummy_jpeg(100, 133);

    for i in 0..n_posts {
        let primary_name = format!("Photos/post_{}_primary.jpg", i);
        let secondary_name = format!("Photos/post_{}_secondary.jpg", i);

        zip.start_file(&primary_name, options).unwrap();
        zip.write_all(&dummy_jpeg).unwrap();

        zip.start_file(&secondary_name, options).unwrap();
        zip.write_all(&dummy_jpeg).unwrap();

        let post = serde_json::json!({
            "id": format!("post_{}", i),
            "takenAt": format!("2024-{:02}-{:02}T12:00:00.000Z", (i % 12) + 1, (i % 28) + 1),
            "primary": { "path": primary_name, "mediaType": "image" },
            "secondary": { "path": secondary_name, "mediaType": "image" },
            "location": { "latitude": 51.5074, "longitude": -0.1278 }
        });
        posts.push(post);
    }

    zip.start_file("posts.json", options).unwrap();
    let posts_str = serde_json::to_string_pretty(&posts).unwrap();
    zip.write_all(posts_str.as_bytes()).unwrap();

    zip.finish().unwrap();
    zip_path
}

fn create_dummy_jpeg(width: u32, height: u32) -> Vec<u8> {
    let img = image::RgbImage::from_pixel(width, height, image::Rgb([45, 55, 72]));
    let mut buffer = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buffer);
    img.write_to(&mut cursor, image::ImageFormat::Jpeg).unwrap();
    buffer
}

#[test]
fn test_benchmark_archive_scan_performance() {
    let temp_dir = std::env::temp_dir().join(format!("bereal_bench_scan_50_{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let zip_path = create_mock_bereal_zip(&temp_dir, 50);

    let start = Instant::now();
    let archive_info = scan_archive(zip_path.to_str().unwrap()).expect("scan_archive failed");
    let elapsed = start.elapsed();

    println!(
        "\n⚡ [BENCHMARK] scan_archive on 50 mock posts took: {:.2}ms",
        elapsed.as_secs_f64() * 1000.0
    );

    assert_eq!(archive_info.archive_type, "Zip");
    assert_eq!(archive_info.valid_post_count, 50);
    assert_eq!(archive_info.user_name.as_deref(), Some("testuser"));
    assert!(archive_info.validation_errors.is_empty());
    assert!(elapsed.as_millis() < 500, "Archive scan should be sub-500ms");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_benchmark_large_archive_scan() {
    let temp_dir = std::env::temp_dir().join(format!("bereal_bench_scan_120_{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let zip_path = create_mock_bereal_zip(&temp_dir, 120);

    let start = Instant::now();
    let archive_info = scan_archive(zip_path.to_str().unwrap()).expect("scan_archive failed");
    let elapsed = start.elapsed();

    println!(
        "⚡ [BENCHMARK] scan_archive on 120 mock posts took: {:.2}ms",
        elapsed.as_secs_f64() * 1000.0
    );

    assert_eq!(archive_info.valid_post_count, 120);
    assert!(elapsed.as_millis() < 1000, "Large archive scan should be sub-1s");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_benchmark_compositing_pip_throughput() {
    let temp_dir = std::env::temp_dir().join(format!("bereal_bench_pip_{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).unwrap();

    let primary_path = temp_dir.join("primary.jpg");
    let secondary_path = temp_dir.join("secondary.jpg");

    let primary = image::RgbImage::from_pixel(600, 800, image::Rgb([30, 41, 59]));
    let secondary = image::RgbImage::from_pixel(600, 800, image::Rgb([49, 46, 129]));
    primary.save(&primary_path).unwrap();
    secondary.save(&secondary_path).unwrap();

    let iterations = 5;
    let start = Instant::now();

    for _ in 0..iterations {
        let _composite = combine_pip(&primary_path, &secondary_path)
            .expect("combine_pip failed");
    }

    let elapsed = start.elapsed();
    let per_image_ms = (elapsed.as_secs_f64() * 1000.0) / iterations as f64;
    let throughput = iterations as f64 / elapsed.as_secs_f64();

    println!(
        "⚡ [BENCHMARK] Picture-in-Picture Compositing: {:.2}ms/image ({:.1} images/sec)",
        per_image_ms, throughput
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_benchmark_compositing_side_by_side_throughput() {
    let temp_dir = std::env::temp_dir().join(format!("bereal_bench_sbs_{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).unwrap();

    let primary_path = temp_dir.join("primary.jpg");
    let secondary_path = temp_dir.join("secondary.jpg");

    let primary = image::RgbImage::from_pixel(600, 800, image::Rgb([30, 41, 59]));
    let secondary = image::RgbImage::from_pixel(600, 800, image::Rgb([49, 46, 129]));
    primary.save(&primary_path).unwrap();
    secondary.save(&secondary_path).unwrap();

    let iterations = 5;
    let start = Instant::now();

    for _ in 0..iterations {
        let _composite = combine_side_by_side(&primary_path, &secondary_path)
            .expect("combine_side_by_side failed");
    }

    let elapsed = start.elapsed();
    let per_image_ms = (elapsed.as_secs_f64() * 1000.0) / iterations as f64;
    let throughput = iterations as f64 / elapsed.as_secs_f64();

    println!(
        "⚡ [BENCHMARK] Side-by-Side Compositing: {:.2}ms/image ({:.1} images/sec)",
        per_image_ms, throughput
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}
