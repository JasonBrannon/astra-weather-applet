// SPDX-License-Identifier: GPL-3.0-or-later

//! Application constants and tuning parameters

/// Default refresh interval for weather data in minutes
/// Set to 15 minutes for resource-friendly operation
pub const DEFAULT_REFRESH_INTERVAL_MINUTES: u32 = 15;

/// Minimum refresh interval to prevent API rate limiting and reduce resource usage (in minutes)
/// Tempest recommends no more frequent than 1 minute for REST API calls
pub const MIN_REFRESH_INTERVAL_MINUTES: u32 = 5;

/// Maximum refresh interval (in minutes)
pub const MAX_REFRESH_INTERVAL_MINUTES: u32 = 60;

/// Forecast refresh interval in minutes
/// Forecasts update less frequently than current conditions
/// Set to 60 minutes (1 hour) for balanced updates
pub const FORECAST_REFRESH_INTERVAL_MINUTES: u32 = 60;

/// Maximum API key length for validation
pub const MAX_API_KEY_LENGTH: usize = 256;

/// Minimum API key length for validation
pub const MIN_API_KEY_LENGTH: usize = 32;

/// Application name for keyring storage
#[allow(dead_code)]
pub const APP_NAME: &str = "astra-weather-applet";

/// Keyring service name for API key storage
/// Uses the app's reverse-DNS identifier for proper integration with GNOME Keyring
pub const KEYRING_SERVICE: &str = "com.slagmine.astra";

/// Keyring username for API key storage
/// Identifies the specific credential within the service
pub const KEYRING_USERNAME: &str = "api-key";

/// Development mode - adds mock stations and alerts for UI/UX testing
pub const DEV_MODE: bool = false;

/// Notification cooldown period in seconds
/// Prevents notification spam by enforcing minimum time between notifications of the same type
/// Lightning notifications: Don't send more than one per minute
pub const LIGHTNING_NOTIFICATION_COOLDOWN_SECONDS: u64 = 60;

/// Rain notification cooldown in seconds
/// Prevents multiple "rain started" notifications within a short period
pub const RAIN_NOTIFICATION_COOLDOWN_SECONDS: u64 = 300; // 5 minutes

/// Wind notification cooldown in seconds
/// Prevents repeated high wind alerts for sustained gusts
pub const WIND_NOTIFICATION_COOLDOWN_SECONDS: u64 = 180; // 3 minutes
