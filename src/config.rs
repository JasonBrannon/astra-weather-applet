// SPDX-License-Identifier: GPL-3.0-or-later

use crate::security::ApiKeyManager;
use cosmic::cosmic_config::{self, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry};

#[derive(Debug, Clone, CosmicConfigEntry, PartialEq)]
#[version = 1]
pub struct Config {
    pub temperature_unit: crate::weather::TemperatureUnit,
    pub wind_speed_unit: crate::weather::WindSpeedUnit,
    pub pressure_unit: crate::weather::PressureUnit,
    pub precipitation_unit: crate::weather::PrecipitationUnit,
    pub distance_unit: crate::weather::DistanceUnit,
    pub selected_station_id: Option<String>,
    pub auto_location: bool,
    pub location_latitude: Option<f64>,
    pub location_longitude: Option<f64>,
    pub refresh_interval_minutes: u32,
    pub notifications_enabled: bool,
    // Per-event notification preferences
    pub lightning_notifications_enabled: bool,
    pub rain_notifications_enabled: bool,
    pub wind_notifications_enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            temperature_unit: crate::weather::TemperatureUnit::default(),
            wind_speed_unit: crate::weather::WindSpeedUnit::default(),
            pressure_unit: crate::weather::PressureUnit::default(),
            precipitation_unit: crate::weather::PrecipitationUnit::default(),
            distance_unit: crate::weather::DistanceUnit::default(),
            selected_station_id: None,
            auto_location: true,
            location_latitude: None,
            location_longitude: None,
            refresh_interval_minutes: crate::constants::DEFAULT_REFRESH_INTERVAL_MINUTES,
            notifications_enabled: true,
            // Per-event notifications enabled by default
            lightning_notifications_enabled: true,
            rain_notifications_enabled: true,
            wind_notifications_enabled: true,
        }
    }
}

impl Config {
    /// Check if the application has a valid API key
    pub fn has_api_key(&self) -> bool {
        ApiKeyManager::new().has_api_key()
    }

    /// Get the stored API key
    pub fn get_api_key(&self) -> Option<String> {
        ApiKeyManager::new().retrieve_api_key().ok()
    }

    /// Store a new API key
    pub fn set_api_key(&self, api_key: &str) -> Result<(), crate::security::SecurityError> {
        ApiKeyManager::new().store_api_key(api_key)
    }

    /// Remove the stored API key
    pub fn remove_api_key(&self) -> Result<(), crate::security::SecurityError> {
        ApiKeyManager::new().remove_api_key()
    }
}
