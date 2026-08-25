use anyhow::{Context, Result};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    sync::{Mutex, OnceLock},
    time::Duration,
};
use tauri::{AppHandle, Emitter, Manager};
use zip::ZipArchive;

use crate::pipeline::types::{DownloadProgressEvent, GeoTierInfo, GeocodingMode, LocationRule, OfflineGeoDbStatus};
use crate::recapper::location_rules::{apply_rules, GeocodedAddress};

// Nominatim requires 1 req/sec max
const NOMINATIM_RATE_LIMIT_MS: u64 = 1100;

// Cache: (lat_rounded, lon_rounded) → resolved string
static GEOCODE_CACHE: OnceLock<Mutex<HashMap<(i64, i64), String>>> = OnceLock::new();

// In-memory Spatial Grid for 200,000+ cities
static SPATIAL_GRID: OnceLock<Mutex<Option<SpatialGrid>>> = OnceLock::new();

fn cache() -> &'static Mutex<HashMap<(i64, i64), String>> {
    GEOCODE_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn spatial_grid_store() -> &'static Mutex<Option<SpatialGrid>> {
    SPATIAL_GRID.get_or_init(|| Mutex::new(None))
}

fn cache_key(lat: f64, lon: f64) -> (i64, i64) {
    ((lat * 1000.0).round() as i64, (lon * 1000.0).round() as i64)
}

#[derive(Debug, Clone)]
pub struct GeoCity {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub country_code: String,
    pub admin1_code: String,
}

pub struct SpatialGrid {
    // Key: (floor(lat) as i16, floor(lon) as i16) -> cities in that 1°x1° cell (~111km)
    pub bins: HashMap<(i16, i16), Vec<GeoCity>>,
    pub total_count: usize,
}

impl SpatialGrid {
    pub fn new() -> Self {
        Self {
            bins: HashMap::new(),
            total_count: 0,
        }
    }

    pub fn insert(&mut self, city: GeoCity) {
        let key = (city.lat.floor() as i16, city.lon.floor() as i16);
        self.bins.entry(key).or_default().push(city);
        self.total_count += 1;
    }

    pub fn find_nearest(&self, lat: f64, lon: f64) -> Option<&GeoCity> {
        let center_lat = lat.floor() as i16;
        let center_lon = lon.floor() as i16;

        let mut best_city: Option<&GeoCity> = None;
        let mut best_dist_sq = f64::MAX;

        // Search 3x3 surrounding bins (±1 degree covers ~220km diameter)
        for dlat in -1..=1 {
            for dlon in -1..=1 {
                let key = (center_lat + dlat, center_lon + dlon);
                if let Some(cities) = self.bins.get(&key) {
                    for city in cities {
                        let d_lat = city.lat - lat;
                        let d_lon = (city.lon - lon) * (lat.to_radians().cos());
                        let dist_sq = d_lat * d_lat + d_lon * d_lon;
                        if dist_sq < best_dist_sq {
                            best_dist_sq = dist_sq;
                            best_city = Some(city);
                        }
                    }
                }
            }
        }

        // If no city in 3x3 (e.g. remote ocean / desert), expand search to 5x5 bins
        if best_city.is_none() {
            for dlat in -2..=2 {
                for dlon in -2..=2 {
                    let key = (center_lat + dlat, center_lon + dlon);
                    if let Some(cities) = self.bins.get(&key) {
                        for city in cities {
                            let d_lat = city.lat - lat;
                            let d_lon = (city.lon - lon) * (lat.to_radians().cos());
                            let dist_sq = d_lat * d_lat + d_lon * d_lon;
                            if dist_sq < best_dist_sq {
                                best_dist_sq = dist_sq;
                                best_city = Some(city);
                            }
                        }
                    }
                }
            }
        }

        best_city
    }
}

pub fn get_geodata_dir(app: &AppHandle) -> Result<PathBuf> {
    let app_dir = app
        .path()
        .app_data_dir()
        .context("Failed to get app data directory")?;
    let geodata_dir = app_dir.join("geodata");
    std::fs::create_dir_all(&geodata_dir)?;
    Ok(geodata_dir)
}

pub fn get_geodata_path(app: &AppHandle, tier: &str) -> Result<PathBuf> {
    let dir = get_geodata_dir(app)?;
    let filename = match tier {
        "cities15000" => "cities15000.txt",
        "cities5000" => "cities5000.txt",
        _ => "cities500.txt",
    };
    Ok(dir.join(filename))
}

pub fn get_active_tier(app: &AppHandle) -> String {
    if let Ok(dir) = get_geodata_dir(app) {
        let active_file = dir.join("active_tier.txt");
        if active_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&active_file) {
                let trimmed = content.trim();
                if ["cities15000", "cities5000", "cities500"].contains(&trimmed) {
                    return trimmed.to_string();
                }
            }
        }
        // Fallback: pick largest installed tier on disk
        for candidate in ["cities500", "cities5000", "cities15000"] {
            if let Ok(p) = get_geodata_path(app, candidate) {
                if p.exists() {
                    return candidate.to_string();
                }
            }
        }
    }
    "cities500".to_string()
}

pub fn set_active_tier_file(app: &AppHandle, tier: &str) -> Result<()> {
    let dir = get_geodata_dir(app)?;
    let active_file = dir.join("active_tier.txt");
    std::fs::write(&active_file, tier)?;
    
    // Switch in-memory grid
    let target_path = get_geodata_path(app, tier)?;
    if target_path.exists() {
        load_spatial_grid(app, Some(tier))?;
    } else if let Ok(mut g) = spatial_grid_store().lock() {
        *g = None;
    }
    clear_cache();
    Ok(())
}

pub fn check_offline_geodb_status(app: &AppHandle) -> OfflineGeoDbStatus {
    let active_tier = get_active_tier(app);
    let tier_defs = [
        (
            "cities15000",
            "Lite",
            "Major Cities (>15,000 Pop)",
            15_000,
            "25,000+ Cities",
            2.5f32,
        ),
        (
            "cities5000",
            "Standard",
            "Towns & Cities (>5,000 Pop)",
            5_000,
            "55,000+ Towns",
            4.5f32,
        ),
        (
            "cities500",
            "Ultra Detailed",
            "Villages & Towns (>500 Pop)",
            500,
            "200,000+ Towns & Villages",
            12.5f32,
        ),
    ];

    let mut tier_statuses = Vec::new();
    let mut active_is_installed = false;
    let mut active_size = 0u64;
    let mut active_path = String::new();

    for (id, name, subtitle, min_pop, approx_cities, approx_mb) in tier_defs {
        let is_active = id == active_tier;
        let path = get_geodata_path(app, id).ok();
        let exists = path.as_ref().map(|p| p.exists()).unwrap_or(false);
        let size = path
            .as_ref()
            .and_then(|p| std::fs::metadata(p).ok())
            .map(|m| m.len())
            .unwrap_or(0);
        let path_str = path
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let count = if exists {
            match id {
                "cities15000" => 25_000,
                "cities5000" => 55_000,
                _ => 200_000,
            }
        } else {
            0
        };

        if is_active {
            active_is_installed = exists && size > 500_000;
            active_size = size;
            active_path = path_str.clone();
        }

        tier_statuses.push(GeoTierInfo {
            id: id.to_string(),
            name: name.to_string(),
            subtitle: subtitle.to_string(),
            min_population: min_pop,
            approx_cities: approx_cities.to_string(),
            approx_download_mb: approx_mb,
            is_installed: exists && size > 500_000,
            is_active,
            file_size_bytes: size,
            city_count: count,
            path: path_str,
        });
    }

    let loaded_count = spatial_grid_store()
        .lock()
        .ok()
        .and_then(|g| g.as_ref().map(|s| s.total_count))
        .unwrap_or(if active_is_installed { 200_000 } else { 0 });

    OfflineGeoDbStatus {
        is_installed: active_is_installed,
        active_tier: active_tier.clone(),
        file_size_bytes: active_size,
        city_count: loaded_count,
        path: active_path,
        version: format!("GeoNames {}", active_tier),
        tiers: tier_statuses,
    }
}

pub fn delete_offline_geodb_file(app: &AppHandle, tier: Option<String>) -> Result<()> {
    if let Some(t) = tier {
        let path = get_geodata_path(app, &t)?;
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        let active = get_active_tier(app);
        if active == t {
            // Find another installed tier
            for candidate in ["cities500", "cities5000", "cities15000"] {
                if let Ok(p) = get_geodata_path(app, candidate) {
                    if p.exists() {
                        let _ = set_active_tier_file(app, candidate);
                        break;
                    }
                }
            }
        }
    } else {
        // Delete all
        for candidate in ["cities500", "cities5000", "cities15000"] {
            if let Ok(p) = get_geodata_path(app, candidate) {
                if p.exists() {
                    let _ = std::fs::remove_file(&p);
                }
            }
        }
        if let Ok(mut g) = spatial_grid_store().lock() {
            *g = None;
        }
    }
    clear_cache();
    Ok(())
}

pub fn load_spatial_grid(app: &AppHandle, tier: Option<&str>) -> Result<()> {
    let active = match tier {
        Some(t) => t.to_string(),
        None => get_active_tier(app),
    };
    let path = get_geodata_path(app, &active)?;
    if !path.exists() {
        anyhow::bail!("Offline geocoding database does not exist: {}", path.display());
    }

    let file = File::open(&path)?;
    let reader = BufReader::new(file);
    let mut grid = SpatialGrid::new();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 11 {
            continue;
        }

        let name = cols[1].trim().to_string();
        let lat: f64 = match cols[4].parse() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let lon: f64 = match cols[5].parse() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let country_code = cols[8].trim().to_uppercase();
        let admin1_code = cols[10].trim().to_string();

        grid.insert(GeoCity {
            name,
            lat,
            lon,
            country_code,
            admin1_code,
        });
    }

    if let Ok(mut g) = spatial_grid_store().lock() {
        *g = Some(grid);
    }

    Ok(())
}

pub async fn download_offline_geodb_stream(app: AppHandle, tier: Option<String>) -> Result<()> {
    let target_tier = tier.unwrap_or_else(|| get_active_tier(&app));
    let (url, txt_filename, approx_bytes) = match target_tier.as_str() {
        "cities15000" => (
            "https://download.geonames.org/export/dump/cities15000.zip",
            "cities15000.txt",
            2_600_000u64,
        ),
        "cities5000" => (
            "https://download.geonames.org/export/dump/cities5000.zip",
            "cities5000.txt",
            4_700_000u64,
        ),
        _ => (
            "https://download.geonames.org/export/dump/cities500.zip",
            "cities500.txt",
            12_800_000u64,
        ),
    };

    let target_txt_path = get_geodata_path(&app, &target_tier)?;
    let temp_zip_path = target_txt_path.with_extension("zip.tmp");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(180))
        .build()?;

    let response = client.get(url).send().await?;
    let total_size = response.content_length().unwrap_or(approx_bytes);

    let bytes = response.bytes().await?;
    let downloaded = bytes.len() as u64;

    let _ = app.emit(
        "download-progress",
        DownloadProgressEvent {
            bytes_downloaded: downloaded,
            total_bytes: total_size,
            percentage: 95.0,
            speed_mbps: 0.0,
            status: format!("Extracting & indexing {} dataset...", target_tier),
        },
    );

    let mut temp_file = File::create(&temp_zip_path)?;
    temp_file.write_all(&bytes)?;
    temp_file.flush()?;
    drop(temp_file);

    // Extract txt from the zip
    let zip_file = File::open(&temp_zip_path)?;
    let mut archive = ZipArchive::new(zip_file)?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        if entry.name() == txt_filename {
            let mut out = File::create(&target_txt_path)?;
            std::io::copy(&mut entry, &mut out)?;
            break;
        }
    }

    let _ = std::fs::remove_file(&temp_zip_path);

    // Set as active tier and load in-memory spatial index
    let _ = set_active_tier_file(&app, &target_tier);
    load_spatial_grid(&app, Some(&target_tier))?;

    let _ = app.emit(
        "download-progress",
        DownloadProgressEvent {
            bytes_downloaded: downloaded,
            total_bytes: total_size,
            percentage: 100.0,
            speed_mbps: 0.0,
            status: format!("Offline {} dataset ready!", target_tier),
        },
    );

    Ok(())
}

/// Resolve GPS coordinates to a display string using the chosen geocoding mode.
pub fn resolve_location(
    lat: f64,
    lon: f64,
    rules: &[LocationRule],
    mode: &GeocodingMode,
    app: Option<&AppHandle>,
) -> String {
    let key = cache_key(lat, lon);
    if let Ok(cache) = cache().lock() {
        if let Some(cached) = cache.get(&key) {
            return cached.clone();
        }
    }

    let result = match mode {
        GeocodingMode::Online => geocode_online(lat, lon, rules).unwrap_or_default(),
        GeocodingMode::Offline => geocode_offline(lat, lon, rules, app).unwrap_or_default(),
    };

    if let Ok(mut cache) = cache().lock() {
        cache.insert(key, result.clone());
    }

    result
}

/// Clear the geocode cache (e.g. when settings change).
pub fn clear_cache() {
    if let Ok(mut cache) = cache().lock() {
        cache.clear();
    }
}

/// Geocode via Nominatim public API. Rate-limited to 1 req/sec.
fn geocode_online(lat: f64, lon: f64, rules: &[LocationRule]) -> Result<String> {
    std::thread::sleep(Duration::from_millis(NOMINATIM_RATE_LIMIT_MS));

    let url = format!(
        "https://nominatim.openstreetmap.org/reverse?format=json&lat={}&lon={}&zoom=18&addressdetails=1",
        lat, lon
    );

    let response: serde_json::Value = ureq::get(&url)
        .set("User-Agent", "BeRealStudio/1.2 (https://github.com/berealstudio)")
        .timeout(Duration::from_secs(10))
        .call()
        .context("Nominatim HTTP request failed")?
        .into_json()
        .context("Nominatim returned invalid JSON")?;

    let addr = &response["address"];
    let geocoded = GeocodedAddress {
        city: first_non_empty(&[
            addr["city"].as_str(),
            addr["town"].as_str(),
            addr["village"].as_str(),
            addr["hamlet"].as_str(),
            addr["county"].as_str(),
        ]),
        suburb: first_non_empty(&[
            addr["suburb"].as_str(),
            addr["neighbourhood"].as_str(),
            addr["district"].as_str(),
        ]),
        state: first_non_empty(&[addr["state"].as_str(), addr["province"].as_str()]),
        country: addr["country"].as_str().unwrap_or("").to_string(),
        country_code: addr["country_code"].as_str().unwrap_or("").to_string(),
        road: addr["road"].as_str().unwrap_or("").to_string(),
    };

    Ok(apply_rules(&geocoded, rules))
}

pub fn spatial_grid_city_count() -> usize {
    spatial_grid_store()
        .lock()
        .ok()
        .and_then(|g| g.as_ref().map(|s| s.total_count))
        .unwrap_or(0)
}

/// Offline geocoding with sub-millisecond 2D spatial grid lookup.
fn geocode_offline(lat: f64, lon: f64, rules: &[LocationRule], app: Option<&AppHandle>) -> Result<String> {
    let needs_load = {
        spatial_grid_store().lock().map(|g| g.is_none()).unwrap_or(true)
    };

    if needs_load {
        if let Some(app_handle) = app {
            let _ = load_spatial_grid(app_handle, None);
        }
    }

    if let Ok(store) = spatial_grid_store().lock() {
        if let Some(ref grid) = *store {
            if let Some(city) = grid.find_nearest(lat, lon) {
                let geocoded = GeocodedAddress {
                    city: city.name.clone(),
                    suburb: String::new(),
                    state: city.admin1_code.clone(),
                    country: String::new(),
                    country_code: city.country_code.clone(),
                    road: String::new(),
                };
                return Ok(apply_rules(&geocoded, rules));
            }
        }
    }

    // If offline DB not loaded, format coordinates directly as fallback instead of blocking on 1.1s HTTP rate-limit
    log::warn!("Offline geocoding DB not available. Returning formatted coordinates.");
    let geocoded = GeocodedAddress {
        city: format!("{:.2}°, {:.2}°", lat, lon),
        suburb: String::new(),
        state: String::new(),
        country: String::new(),
        country_code: String::new(),
        road: String::new(),
    };
    Ok(apply_rules(&geocoded, rules))
}

fn first_non_empty(options: &[Option<&str>]) -> String {
    options
        .iter()
        .flatten()
        .find(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_grid_nearest() {
        let mut grid = SpatialGrid::new();
        grid.insert(GeoCity {
            name: "London".into(),
            lat: 51.5074,
            lon: -0.1278,
            country_code: "GB".into(),
            admin1_code: "ENG".into(),
        });
        grid.insert(GeoCity {
            name: "Paris".into(),
            lat: 48.8566,
            lon: 2.3522,
            country_code: "FR".into(),
            admin1_code: "IDF".into(),
        });

        // Query point near London
        let nearest = grid.find_nearest(51.52, -0.10).expect("Should find London");
        assert_eq!(nearest.name, "London");
        assert_eq!(nearest.country_code, "GB");

        // Query point near Paris
        let nearest_fr = grid.find_nearest(48.80, 2.30).expect("Should find Paris");
        assert_eq!(nearest_fr.name, "Paris");
    }
}
