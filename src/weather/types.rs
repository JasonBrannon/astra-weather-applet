// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemperatureUnit {
    Celsius,
    Fahrenheit,
}

impl Default for TemperatureUnit {
    fn default() -> Self {
        Self::Celsius
    }
}

impl std::fmt::Display for TemperatureUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Celsius => write!(f, "°C"),
            Self::Fahrenheit => write!(f, "°F"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindSpeedUnit {
    #[serde(rename = "mps")]
    MetersPerSecond,
    #[serde(rename = "mph")]
    MilesPerHour,
    #[serde(rename = "kph")]
    KilometersPerHour,
    #[serde(rename = "kts")]
    Knots,
    #[serde(rename = "bft")]
    Beaufort,
    #[serde(rename = "lfm")]
    FeetPerMinute,
}

impl Default for WindSpeedUnit {
    fn default() -> Self {
        Self::MetersPerSecond
    }
}

impl std::fmt::Display for WindSpeedUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MetersPerSecond => write!(f, "m/s"),
            Self::MilesPerHour => write!(f, "mph"),
            Self::KilometersPerHour => write!(f, "km/h"),
            Self::Knots => write!(f, "kts"),
            Self::Beaufort => write!(f, "Bft"),
            Self::FeetPerMinute => write!(f, "ft/min"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PressureUnit {
    #[serde(rename = "mb")]
    Millibar,
    #[serde(rename = "hpa")]
    Hectopascal,
    #[serde(rename = "inhg")]
    InchesOfMercury,
    #[serde(rename = "mmhg")]
    MillimetersOfMercury,
}

impl Default for PressureUnit {
    fn default() -> Self {
        Self::Millibar
    }
}

impl std::fmt::Display for PressureUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Millibar => write!(f, "mb"),
            Self::Hectopascal => write!(f, "hPa"),
            Self::InchesOfMercury => write!(f, "inHg"),
            Self::MillimetersOfMercury => write!(f, "mmHg"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrecipitationUnit {
    #[serde(rename = "mm")]
    Millimeters,
    #[serde(rename = "cm")]
    Centimeters,
    #[serde(rename = "in")]
    Inches,
}

impl Default for PrecipitationUnit {
    fn default() -> Self {
        Self::Millimeters
    }
}

impl std::fmt::Display for PrecipitationUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Millimeters => write!(f, "mm"),
            Self::Centimeters => write!(f, "cm"),
            Self::Inches => write!(f, "in"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistanceUnit {
    #[serde(rename = "km")]
    Kilometers,
    #[serde(rename = "mi")]
    Miles,
}

impl Default for DistanceUnit {
    fn default() -> Self {
        Self::Kilometers
    }
}

impl std::fmt::Display for DistanceUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Kilometers => write!(f, "km"),
            Self::Miles => write!(f, "mi"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    pub city: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherData {
    pub temperature: f64,
    pub humidity: u8,
    pub pressure: f64,
    pub wind_speed: f64,
    pub wind_direction: u16,
    pub wind_gust: Option<f64>,
    pub description: String,
    pub icon: String,
    pub timestamp: u64,
    // Additional observation fields
    pub uv: Option<f64>,
    pub solar_radiation: Option<f64>,
    pub brightness: Option<f64>,
    pub feels_like: Option<f64>,
    pub precip: Option<f64>,
    pub precip_accum_last_1hr: Option<f64>,
    // obs_st WebSocket fields
    pub wind_lull: Option<f64>,
    pub station_pressure: Option<f64>,
    pub sea_level_pressure: Option<f64>, // From REST API
    pub dew_point: Option<f64>,          // Calculated from temp + humidity
    pub precip_type: Option<u8>,         // 0=none, 1=rain, 2=hail, 3=rain+hail
    pub lightning_avg_distance: Option<f64>,
    pub lightning_strike_count: Option<u32>,
    pub lightning_strike_last_epoch: Option<i64>, // Timestamp of last lightning strike
    pub battery_voltage: Option<f64>,
    pub report_interval: Option<u16>,
    pub local_day_rain_accumulation: Option<f64>,
    pub nc_rain_accumulation: Option<f64>, // Noise-corrected rain accumulation
    pub local_day_nc_rain_accumulation: Option<f64>, // Noise-corrected daily rain
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightningEvent {
    pub timestamp: u64,
    pub distance_km: Option<f64>,
    pub intensity: LightningIntensity,
    pub strike_count: u32,
    pub strike_type: LightningStrikeType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LightningIntensity {
    Low,
    Medium,
    High,
    Severe,
}

impl Default for LightningIntensity {
    fn default() -> Self {
        Self::Severe
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LightningStrikeType {
    CloudToCloud,  // ic - Intra-cloud/Inter-cloud
    CloudToGround, // cg - Cloud to Ground
    All,           // all - Mixed/Multiple types
}

impl Default for LightningStrikeType {
    fn default() -> Self {
        Self::CloudToGround // Most dangerous and commonly reported
    }
}

impl std::fmt::Display for LightningStrikeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CloudToCloud => write!(f, "Cloud-to-Cloud"),
            Self::CloudToGround => write!(f, "Cloud-to-Ground"),
            Self::All => write!(f, "Mixed Types"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RainStartEvent {
    pub timestamp: u64,
    pub device_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RapidWindEvent {
    pub timestamp: u64,
    pub device_id: i64,
    pub wind_speed_mps: f64,
    pub wind_direction_deg: f64,
}

/// Unified alert event type for event history tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertEvent {
    Lightning {
        timestamp: u64,
        distance_km: Option<f64>,
        strike_count: u32,
    },
    Rain {
        timestamp: u64,
    },
    Wind {
        timestamp: u64,
        speed_mps: f64,
        direction_deg: f64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherStation {
    pub id: String,
    pub name: String,
    pub location: Location,
    pub is_active: bool,
    pub last_updated: Option<u64>,
    #[serde(default)]
    pub devices: Vec<TempestDevice>,
}

#[derive(Debug, thiserror::Error)]
pub enum WeatherError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("API error: {0}")]
    ApiError(String),
}

// Tempest API response types
#[derive(Debug, Deserialize)]
pub struct TempestStationsResponse {
    pub stations: Vec<TempestStation>,
}

#[derive(Debug, Deserialize)]
pub struct TempestStation {
    pub station_id: i64,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    #[serde(default)]
    pub last_observation: Option<i64>,
    #[serde(default)]
    pub devices: Vec<TempestDevice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TempestDevice {
    pub device_id: i64,
    pub serial_number: String,
    pub device_type: String, // "HB" = Hub, "AR" = Air, "SK" = Sky, "ST" = Tempest
    #[serde(default)]
    pub hardware_revision: Option<String>,
    #[serde(default)]
    pub firmware_revision: Option<String>,
}

// Tempest Station Observation Response
#[derive(Debug, Deserialize)]
pub struct TempestObservationResponse {
    pub status: ObservationStatus,
    #[serde(default)]
    pub station_units: Option<StationUnits>,
    pub obs: Vec<TempestObservation>,
}

#[derive(Debug, Deserialize)]
pub struct ObservationStatus {
    pub status_code: i32,
    pub status_message: String,
}

#[derive(Debug, Deserialize)]
pub struct StationUnits {
    // All unit fields removed - not used in application
}

#[derive(Debug, Clone, Deserialize)]
pub struct TempestObservation {
    pub timestamp: i64,
    #[serde(default)]
    pub air_temperature: Option<f64>,
    #[serde(default)]
    pub barometric_pressure: Option<f64>,
    #[serde(default)]
    pub station_pressure: Option<f64>,
    #[serde(default)]
    pub sea_level_pressure: Option<f64>,
    #[serde(default)]
    pub relative_humidity: Option<f64>,
    #[serde(default)]
    pub precip: Option<f64>,
    #[serde(default)]
    pub precip_accum_last_1hr: Option<f64>,
    #[serde(default)]
    pub wind_avg: Option<f64>,
    #[serde(default)]
    pub wind_direction: Option<u16>,
    #[serde(default)]
    pub wind_gust: Option<f64>,
    #[serde(default)]
    pub wind_lull: Option<f64>,
    #[serde(default)]
    pub solar_radiation: Option<f64>,
    #[serde(default)]
    pub uv: Option<f64>,
    #[serde(default)]
    pub brightness: Option<f64>,
    #[serde(default)]
    pub lightning_strike_last_epoch: Option<i64>,
    #[serde(default)]
    pub lightning_strike_last_distance: Option<f64>,
    #[serde(default)]
    pub lightning_strike_count: Option<u32>,
    #[serde(default)]
    pub feels_like: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyForecast {
    pub timestamp: u64,
    pub temperature: f64,
    pub humidity: u8,
    pub pressure: f64,
    pub wind_speed: f64,
    pub wind_direction: u16,
    pub precipitation_probability: u8,
    pub precipitation_amount: f64,
    pub description: String,
    pub icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyForecast {
    pub date: String,
    pub temperature_high: f64,
    pub temperature_low: f64,
    pub humidity: u8,
    pub pressure: f64,
    pub wind_speed: f64,
    pub wind_direction: u16,
    pub precipitation_probability: u8,
    pub precipitation_amount: f64,
    pub description: String,
    pub icon: String,
    pub sunrise: u64,
    pub sunset: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StormPrediction {
    pub severity: StormSeverity,
    pub probability: u8,
    pub estimated_arrival: Option<u64>,
    pub duration_hours: Option<u32>,
    pub description: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StormSeverity {
    Light,
    Moderate,
    Severe,
    Extreme,
}

impl std::fmt::Display for StormSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Light => write!(f, "Light"),
            Self::Moderate => write!(f, "Moderate"),
            Self::Severe => write!(f, "Severe"),
            Self::Extreme => write!(f, "Extreme"),
        }
    }
}

/// Better Forecast API response from Tempest
#[derive(Debug, Clone, Deserialize)]
pub struct BetterForecastResponse {
    pub current_conditions: BetterForecastCurrent,
    pub forecast: BetterForecastData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BetterForecastData {
    pub daily: Vec<BetterForecastDaily>,
    pub hourly: Vec<BetterForecastHourly>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BetterForecastCurrent {
    pub conditions: String,
    pub icon: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BetterForecastHourly {
    pub time: u64,
    pub conditions: String,
    /// Weather icon with automatic day/night variants (e.g., "clear-day", "clear-night")
    /// The API determines this based on the station's location and actual sunrise/sunset times
    pub icon: String,
    pub air_temperature: f64,
    pub sea_level_pressure: Option<f64>,
    pub relative_humidity: Option<u8>,
    pub precip: Option<f64>,
    pub precip_probability: Option<u8>,
    pub wind_avg: f64,
    pub wind_direction: u16,
    /// Hour in station's local timezone (0-23), accounting for DST
    pub local_hour: Option<u8>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BetterForecastDaily {
    pub day_start_local: u64,
    pub month_num: Option<u8>,
    pub day_num: Option<u8>,
    pub conditions: String,
    /// Weather icon with automatic day/night variants (e.g., "clear-day", "clear-night")
    /// The API determines this based on the station's location and actual sunrise/sunset times
    pub icon: String,
    /// Actual sunrise time (Unix timestamp) for this day at the station's location
    pub sunrise: u64,
    /// Actual sunset time (Unix timestamp) for this day at the station's location
    pub sunset: u64,
    pub air_temp_high: f64,
    pub air_temp_low: f64,
    pub precip: Option<f64>,
    pub precip_probability: Option<u8>,
    pub wind_avg: Option<f64>,
    pub wind_direction: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedForecast {
    pub location: Location,
    pub hourly: Vec<HourlyForecast>,
    pub daily: Vec<DailyForecast>,
    pub storm_predictions: Vec<StormPrediction>,
    pub generated_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StationStatus {
    Online,
    Warning,
    Stale,
    Offline,
}

impl std::fmt::Display for StationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Online => write!(f, "Online"),
            Self::Warning => write!(f, "Warning"),
            Self::Stale => write!(f, "Stale"),
            Self::Offline => write!(f, "Offline"),
        }
    }
}

/// REST API connection status - completely independent from WebSocket
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RestApiStatus {
    /// Never attempted
    NotAttempted,
    /// Currently fetching
    Fetching,
    /// Last fetch succeeded
    Connected,
    /// Last fetch failed
    Failed,
}

impl std::fmt::Display for RestApiStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotAttempted => write!(f, "Not Attempted"),
            Self::Fetching => write!(f, "Fetching..."),
            Self::Connected => write!(f, "Connected"),
            Self::Failed => write!(f, "Failed"),
        }
    }
}

impl Default for RestApiStatus {
    fn default() -> Self {
        Self::NotAttempted
    }
}

/// WebSocket connection status - completely independent from REST API
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WebSocketStatus {
    /// Not connected
    Disconnected,
    /// Attempting to connect
    Connecting,
    /// Connected and receiving data
    Connected,
    /// Connected but no data received yet
    ConnectedIdle,
    /// Connection failed
    Failed,
}

impl std::fmt::Display for WebSocketStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disconnected => write!(f, "Disconnected"),
            Self::Connecting => write!(f, "Connecting..."),
            Self::Connected => write!(f, "Connected"),
            Self::ConnectedIdle => write!(f, "Connected (waiting for data)"),
            Self::Failed => write!(f, "Failed"),
        }
    }
}

impl Default for WebSocketStatus {
    fn default() -> Self {
        Self::Disconnected
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationHealth {
    pub station_id: String,
    pub status: StationStatus,
    pub last_update_age_seconds: u64,
    pub signal_strength: Option<u8>,
    pub battery_level: Option<u8>,
    pub error_message: Option<String>,
}

pub type WeatherResult<T> = Result<T, WeatherError>;
