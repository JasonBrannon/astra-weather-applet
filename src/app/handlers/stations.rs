// SPDX-License-Identifier: GPL-3.0-or-later

//! Station management handlers

use crate::app::{AppState, Message};
use crate::app::messages::AppTab;
use cosmic::Task;

pub fn handle_fetch_stations(state: &mut AppState) -> Task<cosmic::Action<Message>> {
    state.fetching_stations = true;
    let api_key = state.config.get_api_key();
    let station_manager = state.station_manager.clone();

    Task::perform(
        async move {
            if let Some(key) = api_key {
                match station_manager.get_all_stations(&key).await {
                    Ok(stations) => Some(stations),
                    Err(e) => {
                        tracing::error!("Failed to fetch stations: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        },
        |result| {
            cosmic::Action::App(if let Some(stations) = result {
                Message::StationsListReceived(stations)
            } else {
                Message::StationsListReceived(Vec::new())
            })
        },
    )
}

pub fn handle_stations_list_received(
    state: &mut AppState,
    mut stations: Vec<crate::weather::WeatherStation>,
) -> Task<cosmic::Action<Message>> {
    state.fetching_stations = false;

    // Track successful API call if we got stations
    if !stations.is_empty() {
        state.rest_api_last_success = Some(chrono::Utc::now().timestamp() as u64);
        state.rest_api_last_error = None;
    }

    tracing::warn!(
        "[STATIONS] StationsListReceived: {} stations, configured_id={:?}, selected_station={:?}",
        stations.len(),
        state.config.selected_station_id,
        state.selected_station.as_ref().map(|s| &s.id)
    );

    // Track if we failed to fetch stations when we expected to have some
    let expected_station = state.config.selected_station_id.is_some();
    let got_stations = !stations.is_empty();
    state.last_station_fetch_failed = expected_station && !got_stations;

    if state.last_station_fetch_failed {
        state.station_fetch_retry_count += 1;

        tracing::warn!(
            "[STATIONS] Failed to fetch stations (attempt {}) - expected configured station '{}' but got none (network may be down)",
            state.station_fetch_retry_count,
            state.config.selected_station_id.as_ref().unwrap()
        );

        // Retry immediately up to 3 times, then fall back to PeriodicHealthCheck (every 60s)
        if state.station_fetch_retry_count < 3 {
            tracing::warn!("[RETRY] Scheduling immediate retry in 5 seconds...");

            // Retry station fetch after 5 seconds
            return Task::perform(
                async {
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                },
                |_| cosmic::Action::App(Message::FetchStations),
            );
        }
        tracing::warn!(
            "[RETRY] Initial retries exhausted - station will auto-restore when network comes back (checked every 60s via PeriodicHealthCheck)"
        );
        // Don't give up! PeriodicHealthCheck will keep retrying every 60 seconds
        // until the station is successfully loaded
    } else {
        state.last_station_fetch_failed = false;
        state.station_fetch_retry_count = 0; // Reset counter on success
    }

    tracing::info!("[STATIONS] Received {} real stations from API", stations.len());

    // Add mock stations in DEV_MODE for UI/UX testing
    if crate::constants::DEV_MODE {
        let mock_stations = generate_mock_stations();
        tracing::info!(
            "[STATIONS] DEV_MODE enabled: Adding {} mock stations",
            mock_stations.len()
        );
        stations.extend(mock_stations);
    } else {
        tracing::debug!("DEV_MODE is disabled");
    }

    tracing::info!("[STATIONS] Total stations available: {}", stations.len());
    state.available_stations = stations;

    // Initialize station health for all stations
    let now = chrono::Utc::now().timestamp() as u64;
    state.station_health = state
        .available_stations
        .iter()
        .map(|station| {
            let last_update_age = station.last_updated.map(|ts| now - ts).unwrap_or(u64::MAX);

            tracing::debug!(
                "Station {} health check: last_updated={:?}, age={} seconds, is_active={}",
                station.name,
                station.last_updated,
                last_update_age,
                station.is_active
            );

            let status = if !station.is_active {
                crate::weather::StationStatus::Offline
            } else if last_update_age > 3600 {
                crate::weather::StationStatus::Stale
            } else if last_update_age > 600 {
                crate::weather::StationStatus::Warning
            } else {
                crate::weather::StationStatus::Online
            };

            crate::weather::StationHealth {
                station_id: station.id.clone(),
                status,
                last_update_age_seconds: last_update_age,
                // Signal strength requires separate device status API call
                signal_strength: None,
                // Battery data requires separate device status API call
                battery_level: None,
                error_message: match status {
                    crate::weather::StationStatus::Offline => {
                        Some("Station is offline".to_string())
                    }
                    crate::weather::StationStatus::Stale => {
                        Some("Data is outdated".to_string())
                    }
                    _ => None,
                },
            }
        })
        .collect();

    // If we have a configured station ID, ensure it's properly loaded
    // This handles both:
    // 1. Initial load (selected_station is None)
    // 2. API key change/reload (selected_station exists but WebSocket needs reinit)
    if let Some(station_id) = &state.config.selected_station_id {
        let needs_reload = state.selected_station.is_none();
        let station_already_loaded = state
            .selected_station
            .as_ref()
            .map(|s| s.id == *station_id)
            .unwrap_or(false);

        tracing::warn!(
            "[STATIONS] Checking station restoration: configured_id='{}', needs_reload={}, already_loaded={}",
            station_id, needs_reload, station_already_loaded
        );

        if needs_reload {
            tracing::warn!(
                "[STATIONS] Searching for station '{}' in {} available stations",
                station_id,
                state.available_stations.len()
            );

            // Debug: print all station IDs
            for (idx, s) in state.available_stations.iter().enumerate() {
                tracing::warn!("  Station[{}]: id='{}', name='{}'", idx, s.id, s.name);
            }

            if let Some(station) = state.available_stations.iter().find(|s| s.id == *station_id) {
                tracing::info!(
                    "[STATIONS] Auto-restoring configured station: {} (ID: {})",
                    station.name,
                    station.id
                );

                // Load station from API
                state.selected_station = Some(station.clone());

                // Set flag to prevent fetch loop when WebSocket connects
                state.websocket_just_reconnected = true;

                // Initialize WebSocket connection for real-time data
                state.initialize_weather_service_sync();

                // Fetch both weather data AND forecast immediately
                // This ensures full state restoration after suspend/hibernate
                let fetch_weather = state.update(Message::FetchWeatherData);
                let fetch_forecast = state.update(Message::RefreshForecast);

                tracing::info!(
                    "[STATIONS] Station restored - WebSocket initialized, fetching weather and forecast data"
                );
                return Task::batch(vec![fetch_weather, fetch_forecast]);
            } else {
                tracing::warn!(
                    "Configured station ID '{}' not found in fetched stations (network may be down)",
                    station_id
                );
                // DO NOT clear selected_station_id - network might just be down!
                // The station ID is still valid, we just couldn't fetch the details.
                // PeriodicHealthCheck will retry automatically.
                // Only clear if user explicitly removes the station or changes it.
                tracing::info!(
                    "Keeping station ID '{}' in config - will retry fetch on next health check",
                    station_id
                );
            }
        } else if station_already_loaded {
            // Station is already loaded, but we're re-fetching (e.g., API key change)
            // Reinitialize WebSocket to ensure connection is fresh
            tracing::info!(
                "[STATIONS] Station '{}' already loaded - reinitializing WebSocket after API key change",
                station_id
            );

            // Update station data from fresh fetch
            if let Some(station) = state.available_stations.iter().find(|s| s.id == *station_id) {
                state.selected_station = Some(station.clone());
            }

            // Set flag to prevent reconnect loop when WebSocket connects
            state.websocket_just_reconnected = true;

            // Reinitialize WebSocket connection
            state.initialize_weather_service_sync();

            // Fetch fresh data
            let fetch_weather = state.update(Message::FetchWeatherData);
            let fetch_forecast = state.update(Message::RefreshForecast);

            tracing::info!(
                "[STATIONS] Station reloaded - WebSocket reinitialized, fetching fresh data"
            );
            return Task::batch(vec![fetch_weather, fetch_forecast]);
        }
    }

    Task::none()
}

pub fn handle_select_station(
    state: &mut AppState,
    station: crate::weather::WeatherStation,
) -> Task<cosmic::Action<Message>> {
    state.config.selected_station_id = Some(station.id.clone());
    state.selected_station = Some(station.clone());
    state.needs_station_selection = false;
    state.current_tab = AppTab::Weather;

    // Set flag to prevent fetch loop when WebSocket connects
    state.websocket_just_reconnected = true;

    // Initialize WebSocket connection for real-time data
    state.initialize_weather_service_sync();

    tracing::info!(
        "Station selected: {} - WebSocket initialized, fetching weather data",
        station.name
    );

    // Immediately fetch weather data after station selection
    let save_task = state.save_config();
    let fetch_task = state.update(Message::FetchWeatherData);

    Task::batch(vec![save_task, fetch_task])
}

pub fn handle_show_stations_view(state: &mut AppState) -> Task<cosmic::Action<Message>> {
    // Switch to Stations tab
    state.current_tab = AppTab::Stations;

    // Open popup if not already open
    if state.popup.is_none() {
        let new_id = cosmic::iced::window::Id::unique();
        state.popup = Some(new_id);

        let popup_settings = state.core.applet.get_popup_settings(
            state.core.main_window_id().unwrap(),
            new_id,
            Some((720, 500)),
            None,
            None,
        );
        cosmic::iced::platform_specific::shell::commands::popup::get_popup(popup_settings)
    } else {
        Task::none()
    }
}

/// Generate mock stations for development/testing
fn generate_mock_stations() -> Vec<crate::weather::WeatherStation> {
    use crate::weather::{Location, TempestDevice, WeatherStation};

    let cities = [
        ("Denver", 39.7392, -104.9903, "ST-00001", 123001),
        ("Seattle", 47.6062, -122.3321, "ST-00002", 123002),
        ("Miami", 25.7617, -80.1918, "ST-00003", 123003),
        ("Chicago", 41.8781, -87.6298, "ST-00004", 123004),
        ("Phoenix", 33.4484, -112.0740, "ST-00005", 123005),
        ("Boston", 42.3601, -71.0589, "ST-00006", 123006),
        ("Portland", 45.5152, -122.6784, "ST-00007", 123007),
        ("Austin", 30.2672, -97.7431, "ST-00008", 123008),
        ("San Diego", 32.7157, -117.1611, "ST-00009", 123009),
        ("Atlanta", 33.7490, -84.3880, "ST-00010", 123010),
    ];

    let now = chrono::Utc::now().timestamp() as u64;

    cities
        .iter()
        .enumerate()
        .map(|(i, (city, lat, lon, serial, device_id))| {
            let age_minutes = (i as u64) * 5; // Vary staleness for testing
            let is_active = i < 8; // First 8 active, last 2 offline
            let data_age_seconds = age_minutes * 60;

            // Create Hub device
            let hub_device = TempestDevice {
                device_id: device_id + 1000,
                serial_number: format!("HB-{:05}", i + 1),
                device_type: "HB".to_string(),
                hardware_revision: Some("1".to_string()),
                firmware_revision: Some("171".to_string()),
            };

            // Create Sensor device
            let sensor_device = TempestDevice {
                device_id: *device_id,
                serial_number: serial.to_string(),
                device_type: "ST".to_string(),
                hardware_revision: Some("rev_a".to_string()),
                firmware_revision: Some("156".to_string()),
            };

            WeatherStation {
                id: format!("mock-{}", i + 1),
                name: format!("{} Test Station", city),
                location: Location {
                    latitude: *lat,
                    longitude: *lon,
                    city: Some(city.to_string()),
                    country: Some("US".to_string()),
                },
                is_active,
                last_updated: Some(now - data_age_seconds),
                devices: vec![hub_device, sensor_device],
            }
        })
        .collect()
}
