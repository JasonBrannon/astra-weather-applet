// SPDX-License-Identifier: GPL-3.0-or-later

use crate::app::Message;
use cosmic::iced::Length;
use cosmic::prelude::*;
use cosmic::widget::{self, image, text};
use rust_embed::RustEmbed;
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(RustEmbed)]
#[folder = "resources/icons/hicolor/128x128/apps/"]
struct BrandingAssets;

#[derive(RustEmbed)]
#[folder = "resources/weather-icons/"]
struct WeatherIcons;

#[derive(RustEmbed)]
#[folder = "resources/compass-icons/"]
struct CompassIcons;

/// Global cache for dynamically loaded weather icon SVG handles
/// Icons are loaded on-demand and cached to avoid recreating them on every render
static ICON_CACHE: std::sync::Mutex<OnceLock<HashMap<String, cosmic::widget::svg::Handle>>> =
    std::sync::Mutex::new(OnceLock::new());

/// Global cache for compass icon SVG handles
static COMPASS_CACHE: std::sync::Mutex<OnceLock<HashMap<String, cosmic::widget::svg::Handle>>> =
    std::sync::Mutex::new(OnceLock::new());

/// Get or load a weather icon SVG handle dynamically
/// Icons are named to match Tempest API keys (e.g., "clear-day.svg", "rainy.svg")
fn get_or_load_icon(icon_filename: &str) -> Option<cosmic::widget::svg::Handle> {
    // Try to get existing cache
    let lock = ICON_CACHE.lock().ok()?;
    let cache = lock.get_or_init(HashMap::new);

    // Check if icon is already cached (fast path - no logging to avoid spam)
    if let Some(handle) = cache.get(icon_filename) {
        return Some(handle.clone());
    }

    // Icon not cached, need to load it
    drop(lock);

    // Load icon from embedded resources
    let icon_data = WeatherIcons::get(icon_filename);
    if icon_data.is_none() {
        tracing::warn!("[UI] Icon file not found in resources: {}", icon_filename);
        tracing::warn!(
            "   Available files must match API values (e.g., 'clear-day.svg', 'rainy.svg')"
        );
        return None;
    }

    let handle = cosmic::widget::svg::Handle::from_memory(icon_data.unwrap().data.to_vec());
    tracing::info!("📦 Loaded new icon: {}", icon_filename);

    // Cache it for future use
    if let Ok(mut lock) = ICON_CACHE.lock() {
        let cache = lock.get_mut().unwrap();
        cache.insert(icon_filename.to_string(), handle.clone());
    }

    Some(handle)
}

/// Get or load a compass icon SVG handle dynamically
/// Icons are named by direction (e.g., "compass-n.svg", "compass-nne.svg")
fn get_or_load_compass_icon(direction: &str) -> Option<cosmic::widget::svg::Handle> {
    let icon_filename = format!("compass-{}.svg", direction.to_lowercase());

    // Try to get existing cache
    let lock = COMPASS_CACHE.lock().ok()?;
    let cache = lock.get_or_init(HashMap::new);

    // Check if icon is already cached
    if let Some(handle) = cache.get(&icon_filename) {
        return Some(handle.clone());
    }

    // Icon not cached, need to load it
    drop(lock);

    // Load icon from embedded resources
    let icon_data = CompassIcons::get(&icon_filename);
    if icon_data.is_none() {
        tracing::warn!("[UI] Compass icon file not found: {}", icon_filename);
        return None;
    }

    let handle = cosmic::widget::svg::Handle::from_memory(icon_data.unwrap().data.to_vec());

    // Cache it for future use
    if let Ok(mut lock) = COMPASS_CACHE.lock() {
        let cache = lock.get_mut().unwrap();
        cache.insert(icon_filename.clone(), handle.clone());
    }

    Some(handle)
}

/// Creates a small precipitation icon SVG widget
/// Uses rainy-1.svg (simple raindrop) for all precipitation types
/// size: Size of the icon in pixels
pub fn precipitation_icon_widget(size: u16) -> Element<'static, Message> {
    if let Some(handle) = get_or_load_icon("rainy-1.svg") {
        cosmic::widget::svg(handle)
            .width(Length::Fixed(size as f32))
            .height(Length::Fixed(size as f32))
            .into()
    } else {
        // Fallback to Unicode if SVG not found
        text("💧").size(size).into()
    }
}

/// Creates a small humidity icon SVG widget
/// Uses foggy.svg for humidity indication
/// size: Size of the icon in pixels
pub fn humidity_icon_widget(size: u16) -> Element<'static, Message> {
    if let Some(handle) = get_or_load_icon("foggy.svg") {
        cosmic::widget::svg(handle)
            .width(Length::Fixed(size as f32))
            .height(Length::Fixed(size as f32))
            .into()
    } else {
        // Fallback to Unicode if SVG not found
        text("💧").size(size).into()
    }
}

/// Creates a compass SVG widget showing wind direction
/// direction: Cardinal direction string (N, NNE, NE, ENE, E, etc.) or "calm" for no wind
/// size: Size of the icon in pixels
fn compass_svg_widget(direction: &str, size: u16) -> Element<'static, Message> {
    if let Some(handle) = get_or_load_compass_icon(direction) {
        cosmic::widget::svg(handle)
            .width(Length::Fixed(size as f32))
            .height(Length::Fixed(size as f32))
            .into()
    } else {
        // Fallback to Unicode arrow if SVG not found
        let arrow = match direction {
            "N" => "↑",
            "NNE" => "↑",
            "NE" => "↗",
            "ENE" => "→",
            "E" => "→",
            "ESE" => "→",
            "SE" => "↘",
            "SSE" => "↓",
            "S" => "↓",
            "SSW" => "↓",
            "SW" => "↙",
            "WSW" => "←",
            "W" => "←",
            "WNW" => "←",
            "NW" => "↖",
            "NNW" => "↑",
            "calm" => "·",
            _ => "·",
        };
        text(arrow).size(size).into()
    }
}

/// Loads bundled color weather icon as widget (dynamically loaded and cached)
///
/// Returns an Element with the color weather icon loaded from embedded SVG resources
/// Icons are named to match Tempest API values (e.g., "clear-day", "rainy", "partly-cloudy-night")
///
/// **Day/Night Handling**: The Tempest API automatically provides the correct day/night icon
/// variants (e.g., "clear-day" vs "clear-night", "partly-cloudy-day" vs "partly-cloudy-night")
/// based on:
/// - Station's actual latitude/longitude for solar calculations
/// - Station's timezone (including automatic DST handling)
/// - Real sunrise/sunset times for the station's location
///
/// Therefore, no manual day/night adjustment is needed - simply use the icon value from the API.
pub fn weather_icon_widget(
    icon_value: &str,
    description: &str,
    size: u16,
) -> Element<'static, Message> {
    let icon_filename = get_weather_icon_filename(icon_value, description);

    // Try to get or load the SVG icon handle (dynamic loading with caching)
    if let Some(handle) = get_or_load_icon(&icon_filename) {
        return cosmic::widget::svg(handle)
            .width(Length::Fixed(size as f32))
            .height(Length::Fixed(size as f32))
            .into();
    }

    // Fallback to symbolic icon if embedded icon not found
    tracing::warn!(
        "[UI] Could not load SVG icon '{}' for API value '{}', using symbolic fallback",
        icon_filename,
        icon_value
    );
    let symbolic_name = get_weather_icon_symbolic(icon_value, description);
    widget::icon::from_name(symbolic_name).size(size).into()
}

/// Maps Tempest API icon values to bundled SVG filenames
/// Icon files are named to match API values exactly (e.g., "clear-day" → "clear-day.svg")
/// This is purely a pass-through function with legacy name support
fn get_weather_icon_filename(icon_value: &str, _description: &str) -> String {
    // Legacy name support for backward compatibility
    // All modern Tempest API values pass through directly to "{value}.svg"
    let icon_name = match icon_value {
        "clear" => "clear-day",
        "partly-cloudy" => "partly-cloudy-day",
        "overcast" => "cloudy",
        "rain" => "rainy",
        _ => icon_value, // Direct pass-through: API key matches filename
    };

    format!("{}.svg", icon_name)
}

/// Maps Tempest API icon values to symbolic system icon names (fallback)
fn get_weather_icon_symbolic(icon_value: &str, description: &str) -> &'static str {
    let mapped_icon = match icon_value {
        "clear-day" | "clear" => "weather-clear",
        "clear-night" => "weather-clear-night",
        "cloudy" => "weather-overcast",
        "foggy" => "weather-fog",
        "partly-cloudy-day" | "partly-cloudy" => "weather-few-clouds",
        "partly-cloudy-night" => "weather-few-clouds-night",
        "possibly-rainy-day" | "possibly-rainy-night" => "weather-showers-scattered",
        "possibly-sleet-day" | "possibly-sleet-night" => "weather-snow-scattered",
        "possibly-snow-day" | "possibly-snow-night" => "weather-snow",
        "possibly-thunderstorm-day" | "possibly-thunderstorm-night" => "weather-storm",
        "rainy" | "rain" => "weather-showers",
        "sleet" => "weather-snow-scattered",
        "snow" => "weather-snow",
        "thunderstorm" => "weather-storm",
        "windy" => "weather-windy",
        "overcast" => "weather-overcast",
        _ => {
            // Fallback: try to match by description
            let desc_lower = description.to_lowercase();
            if desc_lower.contains("thunder") || desc_lower.contains("storm") {
                "weather-storm"
            } else if desc_lower.contains("snow") {
                "weather-snow"
            } else if desc_lower.contains("rain") || desc_lower.contains("shower") {
                "weather-showers"
            } else if desc_lower.contains("cloud") {
                "weather-few-clouds"
            } else if desc_lower.contains("clear") || desc_lower.contains("sunny") {
                "weather-clear"
            } else if desc_lower.contains("fog") {
                "weather-fog"
            } else if desc_lower.contains("wind") {
                "weather-windy"
            } else {
                "weather-few-clouds"
            }
        }
    };

    // COSMIC uses symbolic icons, so append -symbolic suffix
    match mapped_icon {
        "weather-clear" => "weather-clear-symbolic",
        "weather-clear-night" => "weather-clear-night-symbolic",
        "weather-overcast" => "weather-overcast-symbolic",
        "weather-fog" => "weather-fog-symbolic",
        "weather-few-clouds" => "weather-few-clouds-symbolic",
        "weather-few-clouds-night" => "weather-few-clouds-night-symbolic",
        "weather-showers-scattered" => "weather-showers-scattered-symbolic",
        "weather-snow-scattered" => "weather-snow-symbolic",
        "weather-snow" => "weather-snow-symbolic",
        "weather-storm" => "weather-storm-symbolic",
        "weather-showers" => "weather-showers-symbolic",
        "weather-windy" => "weather-few-clouds-symbolic",
        _ => "weather-few-clouds-symbolic",
    }
}

/// Global cache for branding icon handle
static BRANDING_ICON_CACHE: OnceLock<Option<widget::image::Handle>> = OnceLock::new();

pub fn branding_icon() -> Element<'static, Message> {
    let cached_handle = BRANDING_ICON_CACHE.get_or_init(|| {
        if let Some(icon_data) = BrandingAssets::get("com.slagmine.astra.png") {
            let bytes: Vec<u8> = icon_data.data.to_vec();
            Some(cosmic::iced::widget::image::Handle::from_bytes(bytes))
        } else {
            None
        }
    });

    if let Some(handle) = cached_handle {
        image(handle.clone())
            .width(cosmic::iced::Length::Fixed(24.0))
            .height(cosmic::iced::Length::Fixed(24.0))
            .into()
    } else {
        // Fallback to system icon if the PNG can't be loaded
        widget::icon::from_name("weather-storm-symbolic")
            .size(24)
            .into()
    }
}

/// Compass arrow showing wind direction
/// degrees: 0° = North, 90° = East, 180° = South, 270° = West
/// Arrow points in the direction wind is coming FROM (standard meteorological convention)

/// Returns a compass icon showing wind direction
pub fn compass_icon(
    degrees: Option<f64>,
    speed_mps: Option<f64>,
    size: u16,
) -> Element<'static, Message> {
    // Check if wind is calm (speed is 0 or very low, threshold: 0.5 m/s ≈ 1 mph)
    let is_calm = speed_mps.is_none_or(|speed| speed < 0.5);

    if is_calm {
        // Calm conditions - wind speed is 0 or very low
        compass_svg_widget("calm", size)
    } else if let Some(deg) = degrees {
        // Wind direction degrees indicate where wind is coming FROM
        // SVG icon shows arrow pointing in the SAME direction (standard meteorological convention)
        let direction = get_cardinal_direction(deg);
        compass_svg_widget(direction, size)
    } else {
        // No direction data but wind exists - show calm as fallback
        compass_svg_widget("calm", size)
    }
}

/// Formats wind information as a string (cardinal direction + speed)
pub fn format_wind_text(
    degrees: Option<f64>,
    speed_mps: Option<f64>,
    wind_unit: crate::weather::WindSpeedUnit,
) -> String {
    use crate::weather::format_wind_speed;

    // Check if wind is calm (speed is 0 or very low, threshold: 0.5 m/s ≈ 1 mph)
    let is_calm = speed_mps.is_none_or(|speed| speed < 0.5);

    // Get cardinal direction (N, NNE, NE, ENE, E, ESE, SE, SSE, S, SSW, SW, WSW, W, WNW, NW, NNW)
    let cardinal = if is_calm {
        "Calm"
    } else if let Some(deg) = degrees {
        get_cardinal_direction(deg)
    } else {
        "--  " // Padded to 4 chars for panel stability
    };

    // Format wind speed
    let wind_display = if let Some(speed) = speed_mps {
        format_wind_speed(speed, wind_unit)
    } else {
        format!("0.0 {}", wind_unit)
    };

    // Pad cardinal direction to 4 characters for panel stability (prevents resizing)
    // Longest: "Calm" (4 chars), shortest: "N" (1 char)
    let padded_cardinal = format!("{:4}", cardinal);

    format!("{} {}", padded_cardinal, wind_display)
}

/// Get cardinal direction from degrees (16-point compass)
pub fn get_cardinal_direction(degrees: f64) -> &'static str {
    match degrees as i32 {
        349..=360 | 0..=11 => "N",
        12..=33 => "NNE",
        34..=56 => "NE",
        57..=78 => "ENE",
        79..=101 => "E",
        102..=123 => "ESE",
        124..=146 => "SE",
        147..=168 => "SSE",
        169..=191 => "S",
        192..=213 => "SSW",
        214..=236 => "SW",
        237..=258 => "WSW",
        259..=281 => "W",
        282..=303 => "WNW",
        304..=326 => "NW",
        327..=348 => "NNW",
        _ => "--",
    }
}

/// Get arrow character based on direction (8-point compass) - kept for fallback

/// Get triangle character based on direction (8-point compass)

/// Get battery status text based on voltage
/// Tempest battery voltage ranges:
/// - ≥ 2.7V: Excellent (fully charged)
/// - ≥ 2.5V: Good (healthy)
/// - ≥ 2.3V: Fair (moderate)
/// - ≥ 2.1V: Low (needs attention)
/// - < 2.1V: Critical (replace soon)
pub fn get_battery_status(voltage: f64) -> &'static str {
    if voltage >= 2.7 {
        "Excellent"
    } else if voltage >= 2.5 {
        "Good"
    } else if voltage >= 2.3 {
        "Fair"
    } else if voltage >= 2.1 {
        "Low"
    } else {
        "Critical"
    }
}

/// Get precipitation type text from numeric code
/// Tempest precip_type values:
/// - 0: None (no precipitation)
/// - 1: Rain
/// - 2: Hail
/// - 3: Rain + Hail (mixed)
pub fn get_precip_type_text(precip_type: u8) -> &'static str {
    match precip_type {
        0 => "None",
        1 => "Rain",
        2 => "Hail",
        3 => "Rain+Hail",
        _ => "Unknown",
    }
}
