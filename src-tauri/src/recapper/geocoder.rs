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

pub fn country_code_to_name(code: &str) -> &'static str {
    match code.trim().to_uppercase().as_str() {
        "GB" | "UK" => "United Kingdom",
        "US" => "United States",
        "RO" => "Romania",
        "FR" => "France",
        "DE" => "Germany",
        "IT" => "Italy",
        "ES" => "Spain",
        "PT" => "Portugal",
        "NL" => "Netherlands",
        "BE" => "Belgium",
        "CH" => "Switzerland",
        "AT" => "Austria",
        "SE" => "Sweden",
        "NO" => "Norway",
        "DK" => "Denmark",
        "FI" => "Finland",
        "PL" => "Poland",
        "CZ" => "Czech Republic",
        "HU" => "Hungary",
        "GR" => "Greece",
        "TR" => "Turkey",
        "IE" => "Ireland",
        "CA" => "Canada",
        "AU" => "Australia",
        "NZ" => "New Zealand",
        "JP" => "Japan",
        "KR" => "South Korea",
        "CN" => "China",
        "IN" => "India",
        "BR" => "Brazil",
        "MX" => "Mexico",
        "AR" => "Argentina",
        "CL" => "Chile",
        "ZA" => "South Africa",
        "EG" => "Egypt",
        "AE" => "United Arab Emirates",
        "SA" => "Saudi Arabia",
        "SG" => "Singapore",
        "TH" => "Thailand",
        "VN" => "Vietnam",
        "ID" => "Indonesia",
        "MY" => "Malaysia",
        "PH" => "Philippines",
        "IL" => "Israel",
        "HR" => "Croatia",
        "BG" => "Bulgaria",
        "RS" => "Serbia",
        "UA" => "Ukraine",
        "MD" => "Moldova",
        "IS" => "Iceland",
        "LU" => "Luxembourg",
        "MC" => "Monaco",
        "CY" => "Cyprus",
        "MT" => "Malta",
        _ => "",
    }
}

pub struct SpatialGrid {
    // Key: (floor(lat) as i16, floor(lon) as i16) -> cities in that 1°x1° cell (~111km)
    pub bins: HashMap<(i16, i16), Vec<GeoCity>>,
    pub total_count: usize,
}

impl SpatialGrid {
    pub fn new() -> Self {
        let mut grid = Self {
            bins: HashMap::new(),
            total_count: 0,
        };
        grid.load_embedded_baseline();
        grid
    }

    pub fn insert(&mut self, city: GeoCity) {
        let key = (city.lat.floor() as i16, city.lon.floor() as i16);
        self.bins.entry(key).or_default().push(city);
        self.total_count += 1;
    }

    fn load_embedded_baseline(&mut self) {
        // Embedded baseline of major world cities & locations
        let baseline: &[(&str, f64, f64, &str, &str)] = &[
            // UK & Ireland
            ("London", 51.5074, -0.1278, "GB", "ENG"),
            ("Manchester", 53.4808, -2.2426, "GB", "ENG"),
            ("Birmingham", 52.4862, -1.8904, "GB", "ENG"),
            ("Edinburgh", 55.9533, -3.1883, "GB", "SCT"),
            ("Glasgow", 55.8642, -4.2518, "GB", "SCT"),
            ("Liverpool", 53.4084, -2.9916, "GB", "ENG"),
            ("Bristol", 51.4545, -2.5879, "GB", "ENG"),
            ("Oxford", 51.7520, -1.2577, "GB", "ENG"),
            ("Cambridge", 52.2053, 0.1218, "GB", "ENG"),
            ("Dublin", 53.3498, -6.2603, "IE", "L"),
            // Romania
            ("Constanța", 44.1792, 28.6498, "RO", "CT"),
            ("Bucharest", 44.4268, 26.1025, "RO", "B"),
            ("Cluj-Napoca", 46.7712, 23.6236, "RO", "CJ"),
            ("Timișoara", 45.7537, 21.2257, "RO", "TM"),
            ("Iași", 47.1585, 27.6014, "RO", "IS"),
            ("Brașov", 45.6579, 25.6012, "RO", "BV"),
            ("Sibiu", 45.7983, 24.1256, "RO", "SB"),
            ("Mamaia", 44.2464, 28.6200, "RO", "CT"),
            // Western Europe
            ("Paris", 48.8566, 2.3522, "FR", "IDF"),
            ("Marseille", 43.2965, 5.3698, "FR", "PAC"),
            ("Lyon", 45.7640, 4.8357, "FR", "ARA"),
            ("Nice", 43.7102, 7.2620, "FR", "PAC"),
            ("Berlin", 52.5200, 13.4050, "DE", "BE"),
            ("Munich", 48.1351, 11.5820, "DE", "BY"),
            ("Frankfurt", 50.1109, 8.6821, "DE", "HE"),
            ("Hamburg", 53.5511, 9.9937, "DE", "HH"),
            ("Amsterdam", 52.3676, 4.9041, "NL", "NH"),
            ("Rotterdam", 51.9244, 4.4777, "NL", "ZH"),
            ("Brussels", 50.8503, 4.3517, "BE", "BRU"),
            ("Vienna", 48.2082, 16.3738, "AT", "W"),
            ("Zurich", 47.3769, 8.5417, "CH", "ZH"),
            ("Geneva", 46.2044, 6.1432, "CH", "GE"),
            ("Rome", 41.9028, 12.4964, "IT", "LAZ"),
            ("Milan", 45.4642, 9.1900, "IT", "LOM"),
            ("Venice", 45.4408, 12.3155, "IT", "VEN"),
            ("Florence", 43.7696, 11.2558, "IT", "TOS"),
            ("Madrid", 40.4168, -3.7038, "ES", "MD"),
            ("Barcelona", 41.3879, 2.1699, "ES", "CT"),
            ("Lisbon", 38.7223, -9.1393, "PT", "11"),
            ("Porto", 41.1579, -8.6291, "PT", "13"),
            ("Stockholm", 59.3293, 18.0686, "SE", "AB"),
            ("Oslo", 59.9139, 10.7522, "NO", "03"),
            ("Copenhagen", 55.6761, 12.5683, "DK", "84"),
            ("Helsinki", 60.1699, 24.9384, "FI", "18"),
            ("Athens", 37.9838, 23.7275, "GR", "I"),
            ("Prague", 50.0755, 14.4378, "CZ", "52"),
            ("Budapest", 47.4979, 19.0402, "HU", "BU"),
            ("Warsaw", 52.2297, 21.0122, "PL", "14"),
            // North America
            ("New York", 40.7128, -74.0060, "US", "NY"),
            ("Los Angeles", 34.0522, -118.2437, "US", "CA"),
            ("Chicago", 41.8781, -87.6298, "US", "IL"),
            ("San Francisco", 37.7749, -122.4194, "US", "CA"),
            ("Miami", 25.7617, -80.1918, "US", "FL"),
            ("Seattle", 47.6062, -122.3321, "US", "WA"),
            ("Toronto", 43.6532, -79.3832, "CA", "ON"),
            ("Vancouver", 49.2827, -123.1207, "CA", "BC"),
            ("Montreal", 45.5017, -73.5673, "CA", "QC"),
            // Asia & Oceania
            ("Tokyo", 35.6762, 139.6503, "JP", "13"),
            ("Seoul", 37.5665, 126.9780, "KR", "11"),
            ("Singapore", 1.3521, 103.8198, "SG", "00"),
            ("Sydney", -33.8688, 151.2093, "AU", "NSW"),
            ("Melbourne", -37.8136, 144.9631, "AU", "VIC"),
            ("Dubai", 25.2048, 55.2708, "AE", "DU"),
        ];

        for &(name, lat, lon, country_code, admin1_code) in baseline {
            self.insert(GeoCity {
                name: name.to_string(),
                lat,
                lon,
                country_code: country_code.to_string(),
                admin1_code: admin1_code.to_string(),
            });
        }
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
        } else if let Ok(mut store) = spatial_grid_store().lock() {
            if store.is_none() {
                *store = Some(SpatialGrid::new());
            }
        }
    }

    if let Ok(store) = spatial_grid_store().lock() {
        if let Some(ref grid) = *store {
            if let Some(city) = grid.find_nearest(lat, lon) {
                let country_name = country_code_to_name(&city.country_code);
                let geocoded = GeocodedAddress {
                    city: city.name.clone(),
                    suburb: String::new(),
                    state: city.admin1_code.clone(),
                    country: if !country_name.is_empty() {
                        country_name.to_string()
                    } else {
                        city.country_code.clone()
                    },
                    country_code: city.country_code.clone(),
                    road: String::new(),
                };
                return Ok(apply_rules(&geocoded, rules));
            }
        }
    }

    // Baseline grid fallback
    let baseline_grid = SpatialGrid::new();
    if let Some(city) = baseline_grid.find_nearest(lat, lon) {
        let country_name = country_code_to_name(&city.country_code);
        let geocoded = GeocodedAddress {
            city: city.name.clone(),
            suburb: String::new(),
            state: city.admin1_code.clone(),
            country: if !country_name.is_empty() {
                country_name.to_string()
            } else {
                city.country_code.clone()
            },
            country_code: city.country_code.clone(),
            road: String::new(),
        };
        return Ok(apply_rules(&geocoded, rules));
    }

    Ok(String::new())
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
