// SPDX-License-Identifier: GPL-3.0-or-later

//! Weather data fetch and update handlers

use crate::app::{AppState, Message};
use crate::app::messages::AppTab;
use cosmic::Task;

pub fn handle_fetch_weather_data(state: &mut AppState) -> Task<cosmic::Action<Message>> {
    // Track API attempt time and set status to Fetching
    state.last_api_attempt_time = Some(chrono::Utc::now().timestamp() as u64);
    state.rest_api_status = crate::weather::RestApiStatus::Fetching;

    let station_manager = state.station_manager.clone();
    let api_key = state.config.get_api_key();
    let station = state.selected_station.clone();

    Task::perform(
        async move {
            if let (Some(key), Some(sta)) = (api_key, station) {
                station_manager
                    .get_station_weather_detailed(&sta, &key)
                    .await
                    .map_err(|e| e.to_string())
            } else {
                Err("Missing API key or station".to_string())
            }
        },
        |result| cosmic::Action::App(Message::WeatherDataFetched(result)),
    )
}

pub fn handle_weather_data_fetched(
    state: &mut AppState,
    result: Result<(crate::weather::WeatherData, crate::weather::TempestObservation), String>,
) -> Task<cosmic::Action<Message>> {
    match result {
        Ok((weather, obs)) => {
            // Track successful API call and update REST API status
            let now = chrono::Utc::now().timestamp() as u64;
            state.rest_api_status = crate::weather::RestApiStatus::Connected;
            state.rest_api_last_success = Some(now);
            state.rest_api_last_error = None; // Clear any previous errors

            // Update detailed sensor data
            state.wind_gust_mps = obs.wind_gust;
            state.sea_level_pressure = obs.sea_level_pressure;
            state.lightning_last_epoch = obs.lightning_strike_last_epoch;
            state.lightning_last_distance = obs.lightning_strike_last_distance;
            state.lightning_count_3h = obs.lightning_strike_count;
            state.rain_amount_1h = obs.precip_accum_last_1hr;
            state.uv_index = obs.uv;
            state.solar_radiation = obs.solar_radiation;
            state.brightness = obs.brightness;

            // Update basic weather data
            // Use the weather observation timestamp (not current time)
            // This ensures consistency between header and individual station displays
            state.last_data_update = Some(weather.timestamp);

            // Calculate observation age to determine device states
            let obs_age = now.saturating_sub(weather.timestamp);

            // Update the selected station's last_updated timestamp
            if let Some(station) = &mut state.selected_station {
                station.last_updated = Some(weather.timestamp);

                // IMPORTANT: Don't override device states here!
                // Device states (Hub/Sensor) are managed by PeriodicHealthCheck
                // based on ConnectionState (WebSocket + REST API)
                //
                // Setting states here creates a conflict where:
                // 1. WeatherDataFetched sets Hub=Online (REST API works)
                // 2. PeriodicHealthCheck sets Hub=Degraded (WebSocket down)
                // 3. They fight each other every health check cycle!
                //
                // Let PeriodicHealthCheck be the single source of truth for device states.
                // We only update last_updated timestamp here, which PeriodicHealthCheck uses
                // to determine if data is fresh.

                tracing::info!(
                    "Station '{}': Weather data updated, Data age: {}s (device states managed by PeriodicHealthCheck)",
                    station.name,
                    obs_age
                );
            }

            // Also update in available_stations list to keep it in sync
            if let Some(selected_station) = &state.selected_station {
                if let Some(station_in_list) = state
                    .available_stations
                    .iter_mut()
                    .find(|s| s.id == selected_station.id)
                {
                    station_in_list.last_updated = Some(weather.timestamp);
                    // Don't set is_active or device states here - managed by PeriodicHealthCheck

                    // Update station health for this station
                    if let Some(health) = state
                        .station_health
                        .iter_mut()
                        .find(|h| h.station_id == selected_station.id)
                    {
                        let now = chrono::Utc::now().timestamp() as u64;
                        let last_update_age = weather.timestamp;
                        let age_seconds = now.saturating_sub(last_update_age);

                        health.status = if age_seconds > 3600 {
                            crate::weather::StationStatus::Stale
                        } else if age_seconds > 600 {
                            crate::weather::StationStatus::Warning
                        } else {
                            crate::weather::StationStatus::Online
                        };
                        health.last_update_age_seconds = age_seconds;
                        // Signal strength requires separate device status API call
                        health.signal_strength = None;
                        // Battery level is not available from observation endpoint
                        // It requires a separate device status API call
                        health.battery_level = None;
                        health.error_message = None; // Clear any errors
                    }
                }
            }

            // Only update wind data from REST API if WebSocket is not connected
            // WebSocket provides real-time wind updates (every 2-3 sec)
            // REST API data is stale (5-30 min old)
            if !state.websocket_connected {
                state.wind_speed_mps = Some(weather.wind_speed);
                state.wind_direction_degrees = Some(weather.wind_direction as f64);
                tracing::debug!("Wind data updated from REST API (WebSocket not connected)");
            } else {
                tracing::debug!(
                    "Wind data from REST API ignored (using WebSocket real-time data)"
                );
            }

            state.current_weather = Some(weather.clone());
            tracing::debug!("Weather data updated successfully");

            // Also refresh forecast when weather updates
            return Task::done(cosmic::Action::App(Message::RefreshForecast));
        }
        Err(e) => {
            // Track API error and update REST API status
            state.rest_api_status = crate::weather::RestApiStatus::Failed;
            state.rest_api_last_error = Some(e.clone());
            tracing::error!("Failed to fetch weather data: {}", e);
        }
    }
    Task::none()
}

pub fn handle_detailed_observation_received(
    state: &mut AppState,
    obs: crate::weather::TempestObservation,
) -> Task<cosmic::Action<Message>> {
    // Extract detailed sensor data from observation
    state.wind_gust_mps = obs.wind_gust;
    state.sea_level_pressure = obs.sea_level_pressure;
    state.lightning_last_epoch = obs.lightning_strike_last_epoch;
    state.lightning_last_distance = obs.lightning_strike_last_distance;
    state.lightning_count_3h = obs.lightning_strike_count;
    state.rain_amount_1h = obs.precip_accum_last_1hr;
    state.uv_index = obs.uv;
    state.solar_radiation = obs.solar_radiation;
    state.brightness = obs.brightness;
    // Battery voltage needs to come from device status endpoint

    tracing::debug!("Detailed observation data updated");
    Task::none()
}

pub fn handle_refresh_weather(state: &mut AppState) -> Task<cosmic::Action<Message>> {
    // Manual refresh triggered by user
    if state.selected_station.is_some() {
        tracing::info!("Manual weather refresh requested");
        state.update(Message::FetchWeatherData)
    } else {
        tracing::warn!("Cannot refresh weather: no station selected");
        Task::none()
    }
}

pub fn handle_set_default_station(
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
        "Default station set: {} - WebSocket initialized, fetching weather data",
        station.name
    );

    // Immediately fetch weather data after station selection
    let save_task = state.save_config();
    let fetch_task = state.update(Message::FetchWeatherData);

    Task::batch(vec![save_task, fetch_task])
}
