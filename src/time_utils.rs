// SPDX-License-Identifier: GPL-3.0-or-later

//! Centralized time management utilities
//!
//! **Time Handling Philosophy:**
//! - **Storage**: Always use UTC timestamps (Unix epoch) for internal data
//! - **Display**: Always convert to Local timezone when showing to users
//! - **API**: Tempest API returns UTC timestamps - store as-is
//! - **Comparisons**: Duration calculations work the same in any timezone

use chrono::{DateTime, Local, TimeZone, Utc};
use cosmic::cosmic_config::ConfigGet;
use std::sync::RwLock;

/// Cached time format preference (12h vs 24h)
/// Using RwLock instead of OnceLock to allow refreshing when system settings change
static TIME_FORMAT_CACHE: RwLock<Option<bool>> = RwLock::new(None);

/// Detect if the system uses 12-hour time format based on COSMIC settings
/// Returns true for 12-hour format (AM/PM), false for 24-hour format
fn detect_12h_format() -> bool {
    // COSMIC Desktop stores time format preference in cosmic-applet-time config
    // Read the "military_time" setting: false = 12h, true = 24h
    let is_24h = cosmic::cosmic_config::Config::new("com.system76.CosmicAppletTime", 1)
        .ok()
        .and_then(|config| config.get::<bool>("military_time").ok())
        .unwrap_or(false); // Default to 12-hour if config not found

    let is_12h = !is_24h; // Invert: military_time=false means 12-hour format

    tracing::info!(
        "[TIME] Time format detection: military_time={} -> {}",
        is_24h,
        if is_12h {
            "12h (AM/PM)"
        } else {
            "24h (military)"
        }
    );

    is_12h
}

/// Get the system's time format preference (cached, but refreshable)
pub fn is_12h_format() -> bool {
    // Try to read from cache first
    if let Ok(cache) = TIME_FORMAT_CACHE.read() {
        if let Some(cached_value) = *cache {
            tracing::trace!(
                "Time format cache hit: {}",
                if cached_value { "12h" } else { "24h" }
            );
            return cached_value;
        }
    }

    // Cache miss or read error - detect and cache the value
    tracing::debug!("Time format cache miss - detecting...");
    let detected = detect_12h_format();
    if let Ok(mut cache) = TIME_FORMAT_CACHE.write() {
        *cache = Some(detected);
    }
    detected
}

/// Refresh the time format detection (call this when system settings might have changed)
/// Returns true if the format changed, false if it stayed the same
pub fn refresh_time_format() -> bool {
    let detected = detect_12h_format();
    if let Ok(mut cache) = TIME_FORMAT_CACHE.write() {
        let old_value = *cache;
        *cache = Some(detected);
        if old_value != Some(detected) {
            tracing::info!(
                "⏰ Time format changed: {} -> {}",
                if old_value == Some(true) {
                    "12h"
                } else if old_value == Some(false) {
                    "24h"
                } else {
                    "unknown"
                },
                if detected { "12h" } else { "24h" }
            );
            return true;
        }
    }
    false
}

/// Convert UTC timestamp to Local DateTime for display
pub fn timestamp_to_local(timestamp: u64) -> DateTime<Local> {
    let utc: DateTime<Utc> = Utc.timestamp_opt(timestamp as i64, 0).unwrap();
    utc.with_timezone(&Local)
}

/// Format timestamp for display in local timezone
pub fn format_timestamp(timestamp: u64, format: &str) -> String {
    timestamp_to_local(timestamp).format(format).to_string()
}

/// Format timestamp as time only with system-appropriate format
/// Uses 12-hour (HH:MM AM/PM) or 24-hour (HH:MM:SS) based on system locale
pub fn format_time(timestamp: u64) -> String {
    if is_12h_format() {
        format_timestamp(timestamp, "%I:%M:%S %p")
    } else {
        format_timestamp(timestamp, "%H:%M:%S")
    }
}

/// Format timestamp as date and time with system-appropriate time format
pub fn format_datetime(timestamp: u64) -> String {
    if is_12h_format() {
        format_timestamp(timestamp, "%Y-%m-%d %I:%M:%S %p")
    } else {
        format_timestamp(timestamp, "%Y-%m-%d %H:%M:%S")
    }
}

/// Format timestamp as hour and minute for forecast with system-appropriate format
/// Uses 12-hour (h:MM AM/PM) or 24-hour (HH:MM) based on system locale
pub fn format_hour_minute(timestamp: u64) -> String {
    if is_12h_format() {
        // 12-hour format: "12/31 1:30 PM" (no leading zero for hour)
        format_timestamp(timestamp, "%m/%d %I:%M %p")
    } else {
        // 24-hour format: "12/31 13:30"
        format_timestamp(timestamp, "%m/%d %H:%M")
    }
}
