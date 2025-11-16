// SPDX-License-Identifier: GPL-3.0-or-later

//! Unit conversion utilities for weather data

use super::{DistanceUnit, PrecipitationUnit, PressureUnit, TemperatureUnit, WindSpeedUnit};

/// Convert temperature from Celsius to target unit
pub fn convert_temperature(celsius: f64, target_unit: TemperatureUnit) -> f64 {
    match target_unit {
        TemperatureUnit::Celsius => celsius,
        TemperatureUnit::Fahrenheit => celsius * 9.0 / 5.0 + 32.0,
    }
}

/// Convert wind speed from m/s to target unit
pub fn convert_wind_speed(mps: f64, target_unit: WindSpeedUnit) -> f64 {
    match target_unit {
        WindSpeedUnit::MetersPerSecond => mps,
        WindSpeedUnit::MilesPerHour => mps * 2.237,
        WindSpeedUnit::KilometersPerHour => mps * 3.6,
        WindSpeedUnit::Knots => mps * 1.944,
        WindSpeedUnit::Beaufort => ms_to_beaufort(mps),
        WindSpeedUnit::FeetPerMinute => mps * 196.85,
    }
}

/// Convert m/s to Beaufort scale (0-12)
fn ms_to_beaufort(mps: f64) -> f64 {
    match mps {
        x if x < 0.5 => 0.0,
        x if x < 1.5 => 1.0,
        x if x < 3.3 => 2.0,
        x if x < 5.5 => 3.0,
        x if x < 7.9 => 4.0,
        x if x < 10.7 => 5.0,
        x if x < 13.8 => 6.0,
        x if x < 17.1 => 7.0,
        x if x < 20.7 => 8.0,
        x if x < 24.4 => 9.0,
        x if x < 28.4 => 10.0,
        x if x < 32.6 => 11.0,
        _ => 12.0,
    }
}

/// Convert pressure from mb (millibar) to target unit
pub fn convert_pressure(mb: f64, target_unit: PressureUnit) -> f64 {
    match target_unit {
        PressureUnit::Millibar => mb,
        PressureUnit::Hectopascal => mb, // 1 mb = 1 hPa
        PressureUnit::InchesOfMercury => mb * 0.02953,
        PressureUnit::MillimetersOfMercury => mb * 0.7501,
    }
}

/// Convert precipitation from mm to target unit
pub fn convert_precipitation(mm: f64, target_unit: PrecipitationUnit) -> f64 {
    match target_unit {
        PrecipitationUnit::Millimeters => mm,
        PrecipitationUnit::Centimeters => mm / 10.0,
        PrecipitationUnit::Inches => mm * 0.03937,
    }
}

/// Convert distance from km to target unit
pub fn convert_distance(km: f64, target_unit: DistanceUnit) -> f64 {
    match target_unit {
        DistanceUnit::Kilometers => km,
        DistanceUnit::Miles => km * 0.6214,
    }
}

/// Format temperature with unit
pub fn format_temperature(celsius: f64, unit: TemperatureUnit) -> String {
    let converted = convert_temperature(celsius, unit);
    format!("{:.1}{}", converted, unit)
}

/// Format wind speed with unit
pub fn format_wind_speed(mps: f64, unit: WindSpeedUnit) -> String {
    let converted = convert_wind_speed(mps, unit);
    match unit {
        WindSpeedUnit::Beaufort => format!("{:.0} {}", converted, unit),
        _ => format!("{:.1} {}", converted, unit),
    }
}

/// Format pressure with unit
pub fn format_pressure(mb: f64, unit: PressureUnit) -> String {
    let converted = convert_pressure(mb, unit);
    format!("{:.1} {}", converted, unit)
}

/// Format precipitation with unit
pub fn format_precipitation(mm: f64, unit: PrecipitationUnit) -> String {
    let converted = convert_precipitation(mm, unit);
    format!("{:.1} {}", converted, unit)
}

/// Calculate dew point from temperature (°C) and relative humidity (%)
/// Uses Magnus-Tetens approximation
pub fn calculate_dew_point(temp_celsius: f64, humidity: u8) -> f64 {
    let h = humidity as f64;
    let a = 17.27;
    let b = 237.7;

    let alpha = ((a * temp_celsius) / (b + temp_celsius)) + (h / 100.0).ln();
    (b * alpha) / (a - alpha)
}
