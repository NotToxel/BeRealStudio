use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── BeReal JSON Data Structures ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BeRealPost {
    pub primary: Option<MediaAsset>,
    pub primary_placeholder: Option<MediaAsset>,
    pub secondary: Option<MediaAsset>,
    pub secondary_placeholder: Option<MediaAsset>,
    pub bts_media: Option<MediaAsset>,
    pub taken_at: String,
    pub location: Option<Location>,
    pub caption: Option<String>,
    pub retake_counter: Option<u32>,
    pub visibility: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaAsset {
    pub path: String,
    #[serde(rename = "mediaType")]
    pub media_type: String, // "image" or "video"
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub bucket: Option<String>,
}

impl MediaAsset {
    pub fn is_video(&self) -> bool {
        self.media_type == "video"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DestinationStatus {
    pub exists: bool,
    pub is_directory: bool,
    pub is_file: bool,
    pub file_count: usize,
}

// ─── Processing Configuration ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolkitConfig {
    pub input_path: String,
    pub output_path: String,
    pub date_range_start: Option<String>, // ISO 8601 date, e.g. "2024-01-01"
    pub date_range_end: Option<String>,   // ISO 8601 date, e.g. "2024-12-31"
    pub convert_format: OutputFormat,
    pub quality: u8, // 50–100
    pub create_combined: bool,
    pub combine_mode: CombineMode,
    pub create_reversed: bool,
    pub create_motion_photos: bool,
    pub embed_exif: bool,
    pub keep_original_filename: bool,
    pub cleanup_intermediates: bool,
}

impl Default for ToolkitConfig {
    fn default() -> Self {
        Self {
            input_path: String::new(),
            output_path: String::new(),
            date_range_start: None,
            date_range_end: None,
            convert_format: OutputFormat::Jpeg,
            quality: 90,
            create_combined: true,
            combine_mode: CombineMode::PictureInPicture,
            create_reversed: false,
            create_motion_photos: false,
            embed_exif: true,
            keep_original_filename: false,
            cleanup_intermediates: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecapperConfig {
    pub input_folder: String,
    pub output_path: String,
    pub music_path: String,
    pub date_range_start: Option<String>,
    pub date_range_end: Option<String>,
    pub resolution: (u32, u32),
    pub fps: u32,
    pub start_padding: f64,
    pub end_padding: f64,
    pub speed_mode: SpeedMode,
    pub date_enabled: bool,
    pub date_format: String,
    pub date_position: TextPosition,
    pub date_offset: (i32, i32),
    pub font_path: String,
    pub font_size: u32,
    pub shadow_strength: u32,
    pub location_enabled: bool,
    pub location_position: TextPosition,
    pub location_offset: (i32, i32),
    pub location_rules: Vec<LocationRule>,
    pub geocoding_mode: GeocodingMode,
    pub min_duration_secs: Option<f64>,
    pub max_duration_secs: Option<f64>,
}

impl Default for RecapperConfig {
    fn default() -> Self {
        Self {
            input_folder: String::new(),
            output_path: String::new(),
            music_path: String::new(),
            date_range_start: None,
            date_range_end: None,
            resolution: (1440, 1920),
            fps: 30,
            start_padding: 3.0,
            end_padding: 3.0,
            speed_mode: SpeedMode::Ramp,
            date_enabled: true,
            date_format: "%d %B %Y".to_string(),
            date_position: TextPosition::BottomCenter,
            date_offset: (0, -150),
            font_path: String::new(),
            font_size: 100,
            shadow_strength: 5,
            location_enabled: true,
            location_position: TextPosition::BelowDate,
            location_offset: (0, 0),
            location_rules: vec![
                LocationRule {
                    comment: Some("Default fallback".to_string()),
                    condition: RuleCondition::Default,
                    format: "{city}, {country}".to_string(),
                },
            ],
            geocoding_mode: GeocodingMode::Online,
            min_duration_secs: None,
            max_duration_secs: None,
        }
    }
}

// ─── Enumerations ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OutputFormat {
    Jpeg,
    WebP,
    Png,
}

impl OutputFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            OutputFormat::Jpeg => "jpg",
            OutputFormat::WebP => "webp",
            OutputFormat::Png => "png",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CombineMode {
    PictureInPicture,
    SideBySide,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SpeedMode {
    Ramp,
    Even,
    Accelerate,
    Decelerate,
    Wave,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GeocodingMode {
    Online,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TextPosition {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    MiddleCenter,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
    BelowDate,
    AboveDate,
}

// ─── Location Rules ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationRule {
    pub comment: Option<String>,
    pub condition: RuleCondition,
    pub format: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleCondition {
    Default,
    Match(HashMap<String, String>),
}

impl<'de> serde::Deserialize<'de> for RuleCondition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct RuleConditionVisitor;

        impl<'de> serde::de::Visitor<'de> for RuleConditionVisitor {
            type Value = RuleCondition;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string 'Default'/'default', null, or a map of conditions")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v.eq_ignore_ascii_case("default") {
                    Ok(RuleCondition::Default)
                } else {
                    Err(serde::de::Error::custom(format!("unexpected rule condition string: {}", v)))
                }
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(RuleCondition::Default)
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(RuleCondition::Default)
            }

            fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                let mut map = HashMap::new();
                while let Some((k, v)) = access.next_entry()? {
                    map.insert(k, v);
                }
                Ok(RuleCondition::Match(map))
            }
        }

        deserializer.deserialize_any(RuleConditionVisitor)
    }
}

impl serde::Serialize for RuleCondition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            RuleCondition::Default => serializer.serialize_str("Default"),
            RuleCondition::Match(map) => map.serialize(serializer),
        }
    }
}

// ─── Archive Scan Results ────────────────────────────────────────────────────

/// Rich metadata for a single missing media file, shown in the diagnostic UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissingFileInfo {
    pub path: String,
    pub date: Option<String>,       // "YYYY-MM-DD"
    pub timestamp: Option<String>,  // "YYYY-MM-DD HH:MM:SS UTC"
    pub camera_type: Option<String>, // "primary" | "secondary" | "bts"
}

/// Retake counter statistics across all memories.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetakeStats {
    pub min: u32,
    pub max: u32,
    pub avg: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveInfo {
    pub is_valid: bool,
    pub archive_type: String, // "Zip" | "Directory"
    pub user_name: Option<String>,
    pub user_fullname: Option<String>,
    pub profile_picture_data_url: Option<String>,
    pub entry_count: usize,
    pub valid_post_count: usize,
    pub corrupted_post_count: usize,
    pub total_media_count: usize,
    pub found_media_count: usize,
    pub missing_media_count: usize,
    pub missing_files_sample: Vec<MissingFileInfo>,
    pub earliest_date: Option<String>,
    pub latest_date: Option<String>,
    // Rich media type breakdown
    pub primary_photo_count: usize,
    pub secondary_photo_count: usize,
    pub primary_video_count: usize,
    pub secondary_video_count: usize,
    pub bts_count: usize,
    pub with_location_count: usize,
    pub with_caption_count: usize,
    pub retake_stats: Option<RetakeStats>,
    pub has_posts_json: bool,
    pub has_photos_dir: bool,
    pub has_user_json: bool,
    pub has_videos: bool,
    pub has_bts: bool,
    pub monthly_histogram: Vec<MonthCount>,
    pub validation_errors: Vec<String>,
    pub warnings: Vec<String>,
    pub posts_json_path: String,
    pub media_base_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthCount {
    pub month: String, // "YYYY-MM"
    pub count: u32,
}

impl Default for ArchiveInfo {
    fn default() -> Self {
        Self {
            is_valid: false,
            archive_type: "Directory".into(),
            user_name: None,
            user_fullname: None,
            profile_picture_data_url: None,
            entry_count: 0,
            valid_post_count: 0,
            corrupted_post_count: 0,
            total_media_count: 0,
            found_media_count: 0,
            missing_media_count: 0,
            missing_files_sample: Vec::new(),
            earliest_date: None,
            latest_date: None,
            primary_photo_count: 0,
            secondary_photo_count: 0,
            primary_video_count: 0,
            secondary_video_count: 0,
            bts_count: 0,
            with_location_count: 0,
            with_caption_count: 0,
            retake_stats: None,
            has_posts_json: false,
            has_photos_dir: false,
            has_user_json: false,
            has_videos: false,
            has_bts: false,
            monthly_histogram: Vec::new(),
            validation_errors: Vec::new(),
            warnings: Vec::new(),
            posts_json_path: String::new(),
            media_base_path: String::new(),
        }
    }
}

// ─── Progress Events ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressEvent {
    pub job_id: Option<String>,
    pub stage: ProcessingStage,
    pub current: usize,
    pub total: usize,
    pub percentage: f32,
    pub current_file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEvent {
    pub job_id: Option<String>,
    pub level: LogLevel,
    pub message: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgressEvent {
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub percentage: f32,
    pub speed_mbps: f32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeoTierInfo {
    pub id: String,
    pub name: String,
    pub subtitle: String,
    pub min_population: u32,
    pub approx_cities: String,
    pub approx_download_mb: f32,
    pub is_installed: bool,
    pub is_active: bool,
    pub file_size_bytes: u64,
    pub city_count: usize,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfflineGeoDbStatus {
    pub is_installed: bool,
    pub active_tier: String,
    pub file_size_bytes: u64,
    pub city_count: usize,
    pub path: String,
    pub version: String,
    pub tiers: Vec<GeoTierInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessingStage {
    Scanning,
    Extracting,
    Parsing,
    Converting,
    Compositing,
    WritingExif,
    Cleanup,
    Complete,
    // Recapper-specific
    LoadingAudio,
    Geocoding,
    RenderingFrames,
    EncodingVideo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Debug,
}

// ─── Processing Results ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessingResult {
    pub entries_processed: usize,
    pub files_converted: usize,
    pub combined_created: usize,
    pub reversed_created: usize,
    pub motion_photos_created: usize,
    pub files_skipped: usize,
    pub errors: Vec<String>,
    pub duration_secs: f64,
    pub output_path: String,
}

// ─── Font Info ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontInfo {
    pub family: String,
    pub style: String,
    pub path: String,
}

// ─── App Settings (persisted) ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub toolkit: ToolkitConfig,
    pub recapper: RecapperConfig,
    pub last_input_path: Option<String>,
    pub last_output_path: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            toolkit: ToolkitConfig::default(),
            recapper: RecapperConfig::default(),
            last_input_path: None,
            last_output_path: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareAccelerationInfo {
    pub gpu_name: String,
    pub encoder_name: String,
    pub is_gpu_accelerated: bool,
    pub cpu_cores: usize,
    pub parallel_threads: usize,
}
