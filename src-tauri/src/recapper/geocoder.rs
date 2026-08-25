use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

use crate::pipeline::types::{GeocodingMode, LocationRule};
use crate::recapper::location_rules::{apply_rules, GeocodedAddress};

// Nominatim requires 1 req/sec max
const NOMINATIM_RATE_LIMIT_MS: u64 = 1100;

// Cache: (lat_rounded, lon_rounded) → resolved string
static GEOCODE_CACHE: std::sync::OnceLock<Mutex<HashMap<(i64, i64), String>>> =
    std::sync::OnceLock::new();

fn cache() -> &'static Mutex<HashMap<(i64, i64), String>> {
    GEOCODE_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn cache_key(lat: f64, lon: f64) -> (i64, i64) {
    ((lat * 1000.0).round() as i64, (lon * 1000.0).round() as i64)
}

/// Resolve GPS coordinates to a display string using the chosen geocoding mode.
pub fn resolve_location(
    lat: f64,
    lon: f64,
    rules: &[LocationRule],
    mode: &GeocodingMode,
) -> String {
    let key = cache_key(lat, lon);
    if let Ok(cache) = cache().lock() {
        if let Some(cached) = cache.get(&key) {
            return cached.clone();
        }
    }

    let result = match mode {
        GeocodingMode::Online => geocode_online(lat, lon, rules).unwrap_or_default(),
        GeocodingMode::Offline => geocode_offline(lat, lon, rules).unwrap_or_default(),
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
    // Rate limit
    std::thread::sleep(Duration::from_millis(NOMINATIM_RATE_LIMIT_MS));

    let url = format!(
        "https://nominatim.openstreetmap.org/reverse?format=json&lat={}&lon={}&zoom=18&addressdetails=1",
        lat, lon
    );

    let client = reqwest::blocking::Client::builder()
        .user_agent("BeRealStudio/0.1 (https://github.com/berealstudio)")
        .timeout(Duration::from_secs(10))
        .build()
        .context("Failed to build HTTP client")?;

    let response: serde_json::Value = client
        .get(&url)
        .send()
        .context("Nominatim request failed")?
        .json()
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

/// Offline geocoding stub — returns empty string until an offline DB is downloaded.
fn geocode_offline(lat: f64, lon: f64, rules: &[LocationRule]) -> Result<String> {
    // TODO: Implement offline reverse geocoding using a downloaded spatial DB
    // For now, fall back to online if offline DB not available
    log::warn!("Offline geocoding DB not yet downloaded. Falling back to online.");
    geocode_online(lat, lon, rules)
}

fn first_non_empty(options: &[Option<&str>]) -> String {
    options
        .iter()
        .flatten()
        .find(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_default()
}
