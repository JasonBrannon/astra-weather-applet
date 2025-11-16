// SPDX-License-Identifier: GPL-3.0-or-later

use super::{AppState, Message};
use crate::app::messages::AppTab;
use crate::config::Config;
use cosmic::cosmic_config::CosmicConfigEntry;
use cosmic::{Task, cosmic_config};

impl AppState {
    pub fn update(&mut self, message: Message) -> Task<cosmic::Action<Message>> {
        match message {
            Message::Tick => {
                // Weather data refresh - this happens at the configured interval
                // Also refresh time format detection in case system settings changed
                crate::time_utils::refresh_time_format();

                // Only fetch if we have a station selected
                if self.selected_station.is_some() {
                    tracing::debug!("Tick: Refreshing weather data");
                    self.update(Message::FetchWeatherData)
                } else {
                    Task::none()
                }
            }
            Message::ForecastTick => {
                // Forecast refresh - happens hourly
                // Also refresh time format detection in case system settings changed
                crate::time_utils::refresh_time_format();

                // Only refresh if we have current weather data
                if self.current_weather.is_some() {
                    tracing::debug!("ForecastTick: Refreshing forecast data");
                    self.update(Message::RefreshForecast)
                } else {
                    Task::none()
                }
            }
            Message::WebSocketPoll => {
                // Poll WebSocket event receiver
                if let Some(ref mut receiver) = self.websocket_event_receiver {
                    match receiver.try_recv() {
                        Ok(event) => {
                            return self.update(Message::WebSocketEventReceived(event));
                        }
                        Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                            // No events available, continue
                        }
                        Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                            // Only log and update state once when first disconnected
                            if self.websocket_connected {
                                tracing::warn!(
                                    "WebSocket event channel disconnected - will attempt reconnect in 5 seconds"
                                );
                                self.websocket_connected = false;
                            }
                            // Clear the receiver to stop polling
                            self.websocket_event_receiver = None;

                            // Schedule reconnection attempt after 5 seconds
                            return Task::perform(
                                async {
                                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                                },
                                |_| cosmic::Action::App(Message::WebSocketReconnect),
                            );
                        }
                    }
                }
                Task::none()
            }
            Message::WebSocketReconnect => {
                // Station Connection Manager - WebSocket Reconnect with exponential backoff

                // 1. Check if network is available before attempting
                if !self.network_available {
                    tracing::warn!("🌐 Cannot reconnect WebSocket: network unavailable");
                    return Task::none();
                }

                // 2. Check if we have a station to connect to
                if self.selected_station.is_none() {
                    tracing::warn!("Cannot reconnect WebSocket: no station selected");
                    return Task::none();
                }

                // 3. Skip if already connected
                if self.websocket_connected {
                    tracing::debug!("WebSocket already connected, skipping reconnect");
                    return Task::none();
                }

                // 4. Check exponential backoff timing
                let now = chrono::Utc::now().timestamp() as u64;
                if let Some(last_attempt) = self.last_websocket_reconnect_attempt {
                    let delay = self.calculate_reconnect_delay();
                    let elapsed = now.saturating_sub(last_attempt);

                    if elapsed < delay {
                        let remaining = delay.saturating_sub(elapsed);
                        tracing::debug!(
                            "⏳ Reconnect backoff active: {}s remaining (attempt {})",
                            remaining,
                            self.websocket_reconnect_attempts
                        );

                        // Schedule retry after remaining delay
                        return Task::perform(
                            async move {
                                tokio::time::sleep(std::time::Duration::from_secs(remaining)).await;
                            },
                            |_| cosmic::Action::App(Message::WebSocketReconnect),
                        );
                    }
                }

                // 5. Attempt reconnection
                self.last_websocket_reconnect_attempt = Some(now);
                self.websocket_reconnect_attempts += 1;

                tracing::info!(
                    "🔄 WebSocket reconnection attempt {} (delay: {}s)",
                    self.websocket_reconnect_attempts,
                    self.calculate_reconnect_delay()
                );

                self.initialize_weather_service_sync();
                Task::none()
            }
            Message::LightningCheck => {
                // DEPRECATED: WebSocket now provides real-time lightning events
                // This handler kept for backwards compatibility but does nothing
                // Lightning flash animation is updated via UpdateLightningFlash message
                Task::none()
            }
            Message::ToggleWindow => {
                if let Some(id) = self.popup.take() {
                    cosmic::iced::platform_specific::shell::commands::popup::destroy_popup(id)
                } else {
                    // Check for time format changes when opening popup
                    let format_changed = crate::time_utils::refresh_time_format();
                    if format_changed {
                        tracing::info!("⏰ Time format changed on popup open - forcing UI update");
                        self.time_format_update_counter =
                            self.time_format_update_counter.wrapping_add(1);
                    }

                    // Force Stations tab if station selection is required
                    if self.needs_station_selection && self.current_tab != super::AppTab::Stations {
                        tracing::info!(
                            "🔄 Forcing Stations tab because station selection is required"
                        );
                        self.current_tab = super::AppTab::Stations;
                    }

                    let new_id = cosmic::iced::window::Id::unique();
                    self.popup = Some(new_id);

                    let popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        Some((720, 500)),
                        None,
                        None,
                    );
                    cosmic::iced::platform_specific::shell::commands::popup::get_popup(
                        popup_settings,
                    )
                }
            }
            Message::PopupClosed(id) => {
                if self.popup == Some(id) {
                    self.popup = None;
                }
                Task::none()
            }
            Message::RefreshWeather => {
                // Manual refresh triggered by user
                if self.selected_station.is_some() {
                    tracing::info!("Manual weather refresh requested");
                    self.update(Message::FetchWeatherData)
                } else {
                    tracing::warn!("Cannot refresh weather: no station selected");
                    Task::none()
                }
            }
            Message::WeatherDataReceived(weather) => {
                self.current_weather = Some(weather);
                Task::none()
            }
            Message::StationSelected(station) => {
                self.selected_station = Some(station.clone());
                self.config.selected_station_id = Some(station.id);
                self.save_config()
            }
            Message::ConfigUpdate(config) => {
                self.config = config;
                Task::none()
            }
            Message::ChangeTemperatureUnit(unit) => {
                self.config.temperature_unit = unit;
                self.save_config()
            }
            Message::SelectStation(station) => {
                self.config.selected_station_id = Some(station.id.clone());
                self.selected_station = Some(station.clone());
                self.needs_station_selection = false;
                self.current_tab = super::AppTab::Weather;

                // Set flag to prevent fetch loop when WebSocket connects
                self.websocket_just_reconnected = true;

                // Initialize WebSocket connection for real-time data
                self.initialize_weather_service_sync();

                tracing::info!(
                    "Station selected: {} - WebSocket initialized, fetching weather data",
                    station.name
                );

                // Immediately fetch weather data after station selection
                let save_task = self.save_config();
                let fetch_task = self.update(Message::FetchWeatherData);

                Task::batch(vec![save_task, fetch_task])
            }
            Message::ToggleAutoLocation => {
                self.config.auto_location = !self.config.auto_location;
                self.save_config()
            }
            Message::SetRefreshInterval(minutes) => {
                // Ensure interval is within safe bounds
                let safe_interval = minutes
                    .max(crate::constants::MIN_REFRESH_INTERVAL_MINUTES)
                    .min(crate::constants::MAX_REFRESH_INTERVAL_MINUTES);

                if safe_interval != minutes {
                    tracing::warn!(
                        "Refresh interval {} minutes out of bounds, clamping to {}",
                        minutes,
                        safe_interval
                    );
                }

                self.config.refresh_interval_minutes = safe_interval;
                tracing::info!("Refresh interval set to {} minutes", safe_interval);
                self.save_config()
            }
            Message::LightningDetected(_event) => {
                self.start_lightning_flash();
                Task::none()
            }
            Message::UpdateLightningFlash => {
                self.update_lightning_flash();
                Task::none()
            }
            Message::SetNotifications(enabled) => {
                self.config.notifications_enabled = enabled;
                tracing::info!(
                    "Notifications {}",
                    if enabled { "enabled" } else { "disabled" }
                );
                self.save_config()
            }
            Message::SetLightningNotifications(enabled) => {
                self.config.lightning_notifications_enabled = enabled;
                tracing::info!(
                    "Lightning notifications {}",
                    if enabled { "enabled" } else { "disabled" }
                );
                self.save_config()
            }
            Message::SetRainNotifications(enabled) => {
                self.config.rain_notifications_enabled = enabled;
                tracing::info!(
                    "Rain notifications {}",
                    if enabled { "enabled" } else { "disabled" }
                );
                self.save_config()
            }
            Message::SetWindNotifications(enabled) => {
                self.config.wind_notifications_enabled = enabled;
                tracing::info!(
                    "Wind notifications {}",
                    if enabled { "enabled" } else { "disabled" }
                );
                self.save_config()
            }
            Message::SelectTab(tab) => {
                self.current_tab = tab;

                // Refresh time format when switching tabs (especially useful for Forecast tab)
                // If format changed, increment counter to force UI re-render with new format
                let format_changed = crate::time_utils::refresh_time_format();

                if format_changed {
                    tracing::info!("⏰ Time format changed on tab switch - forcing UI update");
                    self.time_format_update_counter =
                        self.time_format_update_counter.wrapping_add(1);
                }

                Task::none()
            }
            Message::ForecastDataReceived(_forecast) => {
                // DEPRECATED: This message is never sent - all forecasts use BetterForecastReceived
                // Kept for backwards compatibility only
                tracing::warn!(
                    "[DEPRECATED] ForecastDataReceived called but should not be used - use BetterForecastReceived instead"
                );
                Task::none()
            }
            Message::BetterForecastReceived(result) => {
                match result {
                    Ok(better_forecast) => {
                        // Convert BetterForecastResponse to ExtendedForecast
                        // NOTE: All temperatures are in Celsius (SI units from API)
                        // UI layer will convert to user's preferred units
                        // Icon values from API are preserved and mapped in UI layer
                        let location = self
                            .selected_station
                            .as_ref()
                            .map(|s| s.location.clone())
                            .unwrap_or(crate::weather::Location {
                                latitude: 0.0,
                                longitude: 0.0,
                                city: None,
                                country: None,
                            });

                        // Convert hourly forecasts
                        let hourly = better_forecast
                            .forecast
                            .hourly
                            .into_iter()
                            .enumerate()
                            .map(|(idx, h)| {
                                // Use the icon directly from API - it already includes correct day/night variants
                                // The Tempest API automatically provides "clear-day", "clear-night", etc.
                                // based on the station's actual sunrise/sunset times and timezone
                                let icon = h.icon.clone();

                                // Log first 24 hours to verify precipitation probability from API
                                let precip_prob = h.precip_probability.unwrap_or(0);
                                if idx < 24 {
                                    tracing::info!(
                                        "[FORECAST] Hour {}: icon='{}', local_hour={:?}, conditions='{}', precip_prob={}%, precip={:.2}mm",
                                        idx,
                                        icon,
                                        h.local_hour,
                                        h.conditions,
                                        precip_prob,
                                        h.precip.unwrap_or(0.0)
                                    );
                                }

                                crate::weather::HourlyForecast {
                                    timestamp: h.time,
                                    temperature: h.air_temperature,
                                    humidity: h.relative_humidity.unwrap_or(50),
                                    pressure: h.sea_level_pressure.unwrap_or(1013.0),
                                    wind_speed: h.wind_avg,
                                    wind_direction: h.wind_direction,
                                    precipitation_probability: h.precip_probability.unwrap_or(0),
                                    precipitation_amount: h.precip.unwrap_or(0.0),
                                    description: h.conditions.clone(),
                                    icon,
                                }
                            })
                            .collect();

                        // Convert daily forecasts
                        let daily = better_forecast
                            .forecast
                            .daily
                            .into_iter()
                            .map(|d| {
                                // Format date as MM/DD
                                let date =
                                    if let (Some(month), Some(day)) = (d.month_num, d.day_num) {
                                        format!("{:02}/{:02}", month, day)
                                    } else {
                                        // Fallback: format from timestamp
                                        let dt = chrono::DateTime::from_timestamp(
                                            d.day_start_local as i64,
                                            0,
                                        )
                                        .unwrap_or_else(|| chrono::Utc::now());
                                        dt.format("%m/%d").to_string()
                                    };

                                // Use the main icon field from API (not precip_icon)
                                let icon = d.icon.clone();
                                let precip_prob = d.precip_probability.unwrap_or(0);

                                tracing::info!(
                                    "[FORECAST] Daily {}: icon='{}', conditions='{}', precip_prob={}%, precip={:.2}mm",
                                    date,
                                    d.icon,
                                    d.conditions,
                                    precip_prob,
                                    d.precip.unwrap_or(0.0)
                                );

                                crate::weather::DailyForecast {
                                    date,
                                    temperature_high: d.air_temp_high,
                                    temperature_low: d.air_temp_low,
                                    humidity: 50,     // Not provided by API
                                    pressure: 1013.0, // Not provided by API
                                    wind_speed: d.wind_avg.unwrap_or(0.0),
                                    wind_direction: d.wind_direction.unwrap_or(0),
                                    precipitation_probability: d.precip_probability.unwrap_or(0),
                                    precipitation_amount: d.precip.unwrap_or(0.0),
                                    description: d.conditions.clone(),
                                    icon,
                                    sunrise: d.sunrise,
                                    sunset: d.sunset,
                                }
                            })
                            .collect();

                        let extended_forecast = crate::weather::ExtendedForecast {
                            location,
                            hourly,
                            daily,
                            storm_predictions: Vec::new(), // Not provided by better_forecast API
                            generated_at: chrono::Utc::now().timestamp() as u64,
                        };

                        self.extended_forecast = Some(extended_forecast);
                        self.last_forecast_update = Some(chrono::Utc::now().timestamp() as u64);

                        // Update current weather icon from Better Forecast current_conditions
                        if let Some(ref mut weather) = self.current_weather {
                            weather.icon = better_forecast.current_conditions.icon.clone();
                            weather.description =
                                better_forecast.current_conditions.conditions.clone();
                            tracing::debug!(
                                "🎨 Updated current weather icon: '{}' ({})",
                                weather.icon,
                                weather.description
                            );
                        }

                        tracing::info!("[FORECAST] Real forecast data loaded from Tempest API");
                    }
                    Err(e) => {
                        tracing::error!("[FORECAST] Failed to fetch forecast: {}", e);
                    }
                }
                Task::none()
            }
            Message::RefreshForecast => {
                // Fetch real forecast data from Tempest API in SI units
                // UI will convert to user's preferred units
                let station_manager = self.station_manager.clone();
                let api_key = self.config.get_api_key();
                let station = self.selected_station.clone();

                Task::perform(
                    async move {
                        if let (Some(key), Some(sta)) = (api_key, station) {
                            station_manager
                                .get_better_forecast(&sta, &key)
                                .await
                                .map_err(|e| e.to_string())
                        } else {
                            Err("Missing API key or station".to_string())
                        }
                    },
                    |result| cosmic::Action::App(Message::BetterForecastReceived(result)),
                )
            }
            Message::StationHealthReceived(health_data) => {
                self.station_health = health_data;
                Task::none()
            }
            Message::RefreshStations => Task::none(),
            Message::ToggleStationStatus(_station_id) => Task::none(),
            Message::SetApiKey(api_key) => {
                match self.config.set_api_key(&api_key) {
                    Ok(()) => {
                        tracing::info!("API key stored successfully");
                        self.unlock();
                        self.api_key_input.clear();

                        // Always fetch stations when API key is set
                        // This will either populate the station list for selection
                        // or reload the configured station
                        return self.update(Message::FetchStations);
                    }
                    Err(e) => {
                        tracing::error!("Failed to store API key: {}", e);
                    }
                }
                Task::none()
            }
            Message::UpdateApiKeyInput(input) => {
                self.api_key_input = input;
                Task::none()
            }
            Message::RemoveApiKey => {
                match self.config.remove_api_key() {
                    Ok(()) => {
                        tracing::info!("API key removed successfully");
                        self.lock();
                    }
                    Err(e) => {
                        tracing::error!("Failed to remove API key: {}", e);
                    }
                }
                Task::none()
            }
            Message::ShowApiKeyInput => {
                self.show_api_key_input = true;
                self.api_key_input.clear();
                Task::none()
            }
            Message::HideApiKeyInput => {
                self.show_api_key_input = false;
                self.api_key_input.clear();
                Task::none()
            }
            Message::SearchLocation(query) => {
                self.location_search_query = query.clone();
                // In a real implementation, this would perform an async search
                // For now, we'll simulate some search results
                let mock_results = vec![
                    crate::weather::Location {
                        latitude: 40.7128,
                        longitude: -74.0060,
                        city: Some("New York".to_string()),
                        country: Some("US".to_string()),
                    },
                    crate::weather::Location {
                        latitude: 34.0522,
                        longitude: -118.2437,
                        city: Some("Los Angeles".to_string()),
                        country: Some("US".to_string()),
                    },
                ];
                self.location_search_results = mock_results
                    .into_iter()
                    .filter(|loc| {
                        loc.city
                            .as_ref()
                            .map(|city| city.to_lowercase().contains(&query.to_lowercase()))
                            .unwrap_or(false)
                    })
                    .collect();
                Task::none()
            }
            Message::LocationSearchResult(results) => {
                self.location_search_results = results;
                Task::none()
            }
            Message::FetchStations => {
                self.fetching_stations = true;
                let api_key = self.config.get_api_key();
                let station_manager = self.station_manager.clone();

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
            Message::StationsListReceived(mut stations) => {
                self.fetching_stations = false;

                // Track successful API call if we got stations
                if !stations.is_empty() {
                    self.rest_api_last_success = Some(chrono::Utc::now().timestamp() as u64);
                    self.rest_api_last_error = None;
                }

                tracing::warn!(
                    "📥 StationsListReceived: {} stations, configured_id={:?}, selected_station={:?}",
                    stations.len(),
                    self.config.selected_station_id,
                    self.selected_station.as_ref().map(|s| &s.id)
                );

                // Track if we failed to fetch stations when we expected to have some
                let expected_station = self.config.selected_station_id.is_some();
                let got_stations = !stations.is_empty();
                self.last_station_fetch_failed = expected_station && !got_stations;

                if self.last_station_fetch_failed {
                    self.station_fetch_retry_count += 1;

                    tracing::warn!(
                        "❌ Failed to fetch stations (attempt {}) - expected configured station '{}' but got none (network may be down)",
                        self.station_fetch_retry_count,
                        self.config.selected_station_id.as_ref().unwrap()
                    );

                    // Retry immediately up to 3 times, then fall back to PeriodicHealthCheck (every 60s)
                    if self.station_fetch_retry_count < 3 {
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
                    self.last_station_fetch_failed = false;
                    self.station_fetch_retry_count = 0; // Reset counter on success
                }

                tracing::info!("📥 Received {} real stations from API", stations.len());

                // Add mock stations in DEV_MODE for UI/UX testing
                if crate::constants::DEV_MODE {
                    let mock_stations = generate_mock_stations();
                    tracing::info!(
                        "🧪 DEV_MODE enabled: Adding {} mock stations",
                        mock_stations.len()
                    );
                    stations.extend(mock_stations);
                } else {
                    tracing::debug!("DEV_MODE is disabled");
                }

                tracing::info!("[STATIONS] Total stations available: {}", stations.len());
                self.available_stations = stations;

                // Initialize station health for all stations
                let now = chrono::Utc::now().timestamp() as u64;
                self.station_health = self
                    .available_stations
                    .iter()
                    .map(|station| {
                        let last_update_age =
                            station.last_updated.map(|ts| now - ts).unwrap_or(u64::MAX);

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
                if let Some(station_id) = &self.config.selected_station_id {
                    let needs_reload = self.selected_station.is_none();
                    let station_already_loaded = self
                        .selected_station
                        .as_ref()
                        .map(|s| s.id == *station_id)
                        .unwrap_or(false);

                    tracing::warn!(
                        "🔍 Checking station restoration: configured_id='{}', needs_reload={}, already_loaded={}",
                        station_id,
                        needs_reload,
                        station_already_loaded
                    );

                    if needs_reload {
                        tracing::warn!(
                            "🔍 Searching for station '{}' in {} available stations",
                            station_id,
                            self.available_stations.len()
                        );

                        // Debug: print all station IDs
                        for (idx, s) in self.available_stations.iter().enumerate() {
                            tracing::warn!("  Station[{}]: id='{}', name='{}'", idx, s.id, s.name);
                        }

                        if let Some(station) =
                            self.available_stations.iter().find(|s| s.id == *station_id)
                        {
                            tracing::info!(
                                "🔄 Auto-restoring configured station: {} (ID: {})",
                                station.name,
                                station.id
                            );

                            // Load station from API
                            self.selected_station = Some(station.clone());

                            // Set flag to prevent fetch loop when WebSocket connects
                            self.websocket_just_reconnected = true;

                            // Initialize WebSocket connection for real-time data
                            self.initialize_weather_service_sync();

                            // Fetch both weather data AND forecast immediately
                            // This ensures full state restoration after suspend/hibernate
                            let fetch_weather = self.update(Message::FetchWeatherData);
                            let fetch_forecast = self.update(Message::RefreshForecast);

                            tracing::info!(
                                "✅ Station restored - WebSocket initialized, fetching weather and forecast data"
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
                            "🔄 Station '{}' already loaded - reinitializing WebSocket after API key change",
                            station_id
                        );

                        // Update station data from fresh fetch
                        if let Some(station) =
                            self.available_stations.iter().find(|s| s.id == *station_id)
                        {
                            self.selected_station = Some(station.clone());
                        }

                        // Set flag to prevent reconnect loop when WebSocket connects
                        self.websocket_just_reconnected = true;

                        // Reinitialize WebSocket connection
                        self.initialize_weather_service_sync();

                        // Fetch fresh data
                        let fetch_weather = self.update(Message::FetchWeatherData);
                        let fetch_forecast = self.update(Message::RefreshForecast);

                        tracing::info!(
                            "✅ Station reloaded - WebSocket reinitialized, fetching fresh data"
                        );
                        return Task::batch(vec![fetch_weather, fetch_forecast]);
                    }
                }

                Task::none()
            }
            Message::SetDefaultStation(station) => {
                self.config.selected_station_id = Some(station.id.clone());
                self.selected_station = Some(station.clone());
                self.needs_station_selection = false;
                self.current_tab = super::AppTab::Weather;

                // Set flag to prevent fetch loop when WebSocket connects
                self.websocket_just_reconnected = true;

                // Initialize WebSocket connection for real-time data
                self.initialize_weather_service_sync();

                tracing::info!(
                    "Default station set: {} - WebSocket initialized, fetching weather data",
                    station.name
                );

                // Immediately fetch weather data after station selection
                let save_task = self.save_config();
                let fetch_task = self.update(Message::FetchWeatherData);

                Task::batch(vec![save_task, fetch_task])
            }
            Message::FetchWeatherData => {
                // Track API attempt time and set status to Fetching
                self.last_api_attempt_time = Some(chrono::Utc::now().timestamp() as u64);
                self.rest_api_status = crate::weather::RestApiStatus::Fetching;

                let station_manager = self.station_manager.clone();
                let api_key = self.config.get_api_key();
                let station = self.selected_station.clone();

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
            Message::WeatherDataFetched(result) => {
                match result {
                    Ok((weather, obs)) => {
                        // Track successful API call and update REST API status
                        let now = chrono::Utc::now().timestamp() as u64;
                        self.rest_api_status = crate::weather::RestApiStatus::Connected;
                        self.rest_api_last_success = Some(now);
                        self.rest_api_last_error = None; // Clear any previous errors

                        // Update detailed sensor data
                        self.wind_gust_mps = obs.wind_gust;
                        self.sea_level_pressure = obs.sea_level_pressure;
                        self.lightning_last_epoch = obs.lightning_strike_last_epoch;
                        self.lightning_last_distance = obs.lightning_strike_last_distance;
                        self.lightning_count_3h = obs.lightning_strike_count;
                        self.rain_amount_1h = obs.precip_accum_last_1hr;
                        self.uv_index = obs.uv;
                        self.solar_radiation = obs.solar_radiation;
                        self.brightness = obs.brightness;

                        // Update basic weather data
                        // Use the weather observation timestamp (not current time)
                        // This ensures consistency between header and individual station displays
                        self.last_data_update = Some(weather.timestamp);

                        // Calculate observation age to determine device states
                        let obs_age = now.saturating_sub(weather.timestamp);

                        // Update the selected station's last_updated timestamp
                        if let Some(station) = &mut self.selected_station {
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
                        if let Some(selected_station) = &self.selected_station {
                            if let Some(station_in_list) = self
                                .available_stations
                                .iter_mut()
                                .find(|s| s.id == selected_station.id)
                            {
                                station_in_list.last_updated = Some(weather.timestamp);
                                // Don't set is_active or device states here - managed by PeriodicHealthCheck

                                // Update station health for this station
                                if let Some(health) = self
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
                        if !self.websocket_connected {
                            self.wind_speed_mps = Some(weather.wind_speed);
                            self.wind_direction_degrees = Some(weather.wind_direction as f64);
                            tracing::debug!(
                                "Wind data updated from REST API (WebSocket not connected)"
                            );
                        } else {
                            tracing::debug!(
                                "Wind data from REST API ignored (using WebSocket real-time data)"
                            );
                        }

                        self.current_weather = Some(weather.clone());
                        tracing::debug!("Weather data updated successfully");

                        // Also refresh forecast when weather updates
                        return Task::done(cosmic::Action::App(Message::RefreshForecast));
                    }
                    Err(e) => {
                        // Track API error and update REST API status
                        self.rest_api_status = crate::weather::RestApiStatus::Failed;
                        self.rest_api_last_error = Some(e.clone());
                        tracing::error!("Failed to fetch weather data: {}", e);
                    }
                }
                Task::none()
            }
            Message::WebSocketConnected => {
                self.websocket_connected = true;
                tracing::info!("WebSocket connection status: Connected");
                Task::none()
            }
            Message::WebSocketDisconnected => {
                self.websocket_connected = false;
                tracing::warn!("WebSocket connection status: Disconnected");
                Task::none()
            }
            Message::WebSocketLightningStrike(event) => {
                // Use existing lightning flash animation
                self.start_lightning_flash();

                // Send notification if enabled
                if self.config.notifications_enabled {
                    if let Err(e) = self
                        .notification_manager
                        .send_lightning_notification(&event, self.config.distance_unit)
                    {
                        tracing::warn!("Failed to send lightning notification: {}", e);
                    }
                }

                tracing::warn!(
                    "[ALERT] Lightning strike: {} km away",
                    event.distance_km.unwrap_or(0.0)
                );
                Task::none()
            }
            Message::WebSocketRainStart(event) => {
                // Store the rain start event
                self.last_rain_start = Some(event.clone());

                // Send rain start notification if enabled
                if self.config.notifications_enabled {
                    if let Err(e) = self.notification_manager.send_notification(
                        "Rain Started",
                        "Precipitation detected at your weather station",
                        notify_rust::Timeout::Milliseconds(5000),
                    ) {
                        tracing::warn!("Failed to send rain notification: {}", e);
                    }
                }

                tracing::info!("[EVENT] Rain started at station {}", event.device_id);
                Task::none()
            }
            Message::WebSocketRapidWind(event) => {
                // Store the rapid wind event
                self.last_rapid_wind = Some(event.clone());

                // Update wind speed and direction for compass display
                self.wind_speed_mps = Some(event.wind_speed_mps);
                self.wind_direction_degrees = Some(event.wind_direction_deg);

                // Calculate wind speed in mph for display
                let wind_mph = event.wind_speed_mps * 2.237;

                // Send high wind notification (>33 mph) if enabled
                if wind_mph > 33.0 && self.config.notifications_enabled {
                    if let Err(e) = self.notification_manager.send_notification(
                        "High Wind Alert",
                        &format!(
                            "Wind gust: {:.1} mph from {}°",
                            wind_mph, event.wind_direction_deg
                        ),
                        notify_rust::Timeout::Milliseconds(5000),
                    ) {
                        tracing::warn!("Failed to send wind notification: {}", e);
                    }
                }

                tracing::info!(
                    "[ALERT] Wind gust: {:.1} m/s ({:.1} mph) from {}°",
                    event.wind_speed_mps,
                    wind_mph,
                    event.wind_direction_deg
                );
                Task::none()
            }
            Message::WebSocketWindDirection(degrees) => {
                self.wind_direction_degrees = Some(degrees);
                Task::none()
            }
            Message::WebSocketEventReceived(event) => {
                use crate::weather::WebSocketEvent;
                match event {
                    WebSocketEvent::ConnectionEstablishedWithHandle(handle) => {
                        self.websocket_connected = true;
                        self.websocket_task_handle = Some(handle);

                        // Update WebSocket status
                        self.websocket_status = crate::weather::WebSocketStatus::Connected;
                        self.websocket_last_connected = Some(chrono::Utc::now().timestamp() as u64);

                        // Reset exponential backoff on successful connection
                        self.reset_reconnect_backoff();

                        tracing::info!(
                            "✅ WebSocket connection established with message handler AbortHandle stored"
                        );

                        // Only refresh station data if this is a RECONNECTION (not initial connection)
                        // BUT: Don't fetch if we just manually reinitialized WebSocket (e.g., after API key change)
                        // because that would create a reconnect loop!
                        let is_manual_reinit = self.websocket_just_reconnected;

                        if self.selected_station.is_some() && !is_manual_reinit {
                            // This is a TRUE reconnection scenario (laptop resume, network restore)
                            // Immediately fetch fresh station data to update hub/sensor status
                            self.websocket_just_reconnected = true;

                            tracing::info!(
                                "🔄 WebSocket reconnected (external event) - fetching fresh station data"
                            );
                            return Task::batch(vec![
                                self.update(Message::FetchStations),
                                self.update(Message::FetchWeatherData),
                            ]);
                        } else if is_manual_reinit {
                            // Manual reinit (from API key change or station reload) - don't trigger fetch loop
                            tracing::info!(
                                "✅ WebSocket connected after manual reinit - skipping station fetch to avoid loop"
                            );
                            // Reset flag after consuming it
                            self.websocket_just_reconnected = false;
                        } else {
                            // Initial connection during startup - don't interfere with station loading
                            tracing::info!(
                                "✅ Initial WebSocket connection - waiting for station to be loaded"
                            );
                        }
                    }
                    WebSocketEvent::RapidWind {
                        speed_mps,
                        direction_deg,
                    } => {
                        // Update WebSocket status and timestamp
                        self.websocket_status = crate::weather::WebSocketStatus::Connected;
                        self.websocket_last_message = Some(chrono::Utc::now().timestamp() as u64);

                        // Update separate state fields (used by panel view)
                        self.wind_speed_mps = Some(speed_mps);
                        self.wind_direction_degrees = Some(direction_deg);

                        // ALSO update current_weather (used by weather view)
                        if let Some(ref mut weather) = self.current_weather {
                            weather.wind_speed = speed_mps;
                            weather.wind_direction = direction_deg as u16;
                            weather.timestamp = chrono::Utc::now().timestamp() as u64;
                        }

                        // High wind threshold: 15 m/s (strong breeze/near gale force)
                        // This is ~33 mph, ~54 km/h, ~29 knots, or Beaufort 7
                        const HIGH_WIND_THRESHOLD_MPS: f64 = 15.0;

                        // Add to event history if high wind (keep last 50 events)
                        if speed_mps > HIGH_WIND_THRESHOLD_MPS {
                            let timestamp = chrono::Utc::now().timestamp() as u64;
                            self.alert_history.push(crate::weather::AlertEvent::Wind {
                                timestamp,
                                speed_mps,
                                direction_deg,
                            });
                            if self.alert_history.len() > 50 {
                                self.alert_history.remove(0);
                            }
                        }

                        // Send high wind notification if threshold exceeded AND per-event enabled AND throttle allows it
                        if speed_mps > HIGH_WIND_THRESHOLD_MPS
                            && self.config.notifications_enabled
                            && self.config.wind_notifications_enabled
                        {
                            let now = chrono::Utc::now().timestamp() as u64;
                            let should_notify =
                                if let Some(last_notif) = self.last_wind_notification_time {
                                    let elapsed = now.saturating_sub(last_notif);
                                    elapsed >= crate::constants::WIND_NOTIFICATION_COOLDOWN_SECONDS
                                } else {
                                    true // First notification
                                };

                            if should_notify {
                                // Format wind speed using user's preferred unit
                                let wind_formatted = crate::weather::conversions::format_wind_speed(
                                    speed_mps,
                                    self.config.wind_speed_unit,
                                );

                                if let Err(e) = self.notification_manager.send_notification(
                                    "[ALERT] High Wind Alert",
                                    &format!(
                                        "Wind gust: {} from {}°",
                                        wind_formatted, direction_deg
                                    ),
                                    notify_rust::Timeout::Milliseconds(5000),
                                ) {
                                    tracing::warn!("Failed to send wind notification: {}", e);
                                } else {
                                    // Update throttle timestamp only if notification succeeded
                                    self.last_wind_notification_time = Some(now);
                                    tracing::warn!(
                                        "[NOTIFICATION] High wind alert sent: {} ({:.1} m/s) from {}°",
                                        wind_formatted,
                                        speed_mps,
                                        direction_deg
                                    );
                                }
                            } else {
                                tracing::debug!(
                                    "[ALERT] High wind detected ({:.1} m/s) but notification throttled (cooldown active)",
                                    speed_mps
                                );
                            }
                        }

                        tracing::info!(
                            "[EVENT] Wind updated from WebSocket: {:.1} m/s from {:.0}°",
                            speed_mps,
                            direction_deg
                        );
                    }
                    WebSocketEvent::ObsStation {
                        wind_lull,
                        wind_avg,
                        wind_gust,
                        wind_direction,
                        station_pressure,
                        air_temperature,
                        relative_humidity,
                        illuminance,
                        uv,
                        solar_radiation,
                        precip_accumulated,
                        precip_type,
                        lightning_strike_avg_distance,
                        lightning_strike_count,
                        lightning_strike_last_epoch,
                        battery,
                        report_interval,
                        local_day_rain_accumulation,
                        nc_rain_accumulation,
                        local_day_nc_rain_accumulation,
                    } => {
                        // Update WebSocket message timestamp
                        self.websocket_last_message = Some(chrono::Utc::now().timestamp() as u64);

                        // Update all obs_st fields in current_weather
                        if let Some(ref mut weather) = self.current_weather {
                            // Wind data
                            weather.wind_lull = wind_lull;
                            weather.wind_speed = wind_avg;
                            weather.wind_gust = wind_gust;
                            weather.wind_direction = wind_direction as u16;
                            self.wind_speed_mps = Some(wind_avg);
                            self.wind_direction_degrees = Some(wind_direction);
                            self.wind_gust_mps = wind_gust;

                            // Pressure
                            if let Some(pressure) = station_pressure {
                                weather.station_pressure = Some(pressure);
                                weather.pressure = pressure;
                                self.sea_level_pressure = Some(pressure);
                            }

                            // Temperature and humidity
                            if let Some(temp) = air_temperature {
                                weather.temperature = temp;
                            }
                            if let Some(humidity) = relative_humidity {
                                weather.humidity = humidity;

                                // Recalculate dew point if we have both temp and humidity
                                if let Some(temp) = air_temperature {
                                    weather.dew_point =
                                        Some(crate::weather::conversions::calculate_dew_point(
                                            temp, humidity,
                                        ));
                                }
                            }

                            // Light measurements
                            weather.brightness = illuminance;
                            weather.uv = uv;
                            weather.solar_radiation = solar_radiation;
                            self.brightness = illuminance;
                            self.uv_index = uv;
                            self.solar_radiation = solar_radiation;

                            // Precipitation
                            weather.precip = precip_accumulated;
                            weather.precip_type = precip_type;
                            weather.local_day_rain_accumulation = local_day_rain_accumulation;
                            weather.nc_rain_accumulation = nc_rain_accumulation;
                            weather.local_day_nc_rain_accumulation = local_day_nc_rain_accumulation;

                            // Lightning
                            weather.lightning_avg_distance = lightning_strike_avg_distance;
                            weather.lightning_strike_count = lightning_strike_count;
                            weather.lightning_strike_last_epoch = lightning_strike_last_epoch;
                            self.lightning_last_distance = lightning_strike_avg_distance;
                            if let Some(count) = lightning_strike_count {
                                self.lightning_count_3h = Some(count);
                            }
                            if let Some(epoch) = lightning_strike_last_epoch {
                                self.lightning_last_epoch = Some(epoch);
                            }

                            // Battery and reporting
                            weather.battery_voltage = battery;
                            weather.report_interval = report_interval;
                            self.battery_voltage = battery;

                            // Update timestamp to current time
                            weather.timestamp = chrono::Utc::now().timestamp() as u64;
                            self.last_data_update = Some(weather.timestamp);

                            tracing::info!(
                                "[DATA] Complete obs_st data received via WebSocket - all metrics updated"
                            );

                            // Update WebSocket status - receiving obs_st data means connected
                            self.websocket_status = crate::weather::WebSocketStatus::Connected;

                            tracing::info!(
                                "[DATA] Complete obs_st data received via WebSocket - WebSocket status: Connected"
                            );
                        }

                        // Clear the reconnection flag
                        if self.websocket_just_reconnected {
                            self.websocket_just_reconnected = false;
                            tracing::info!(
                                "🔄 First obs_st after reconnect - WebSocket receiving data again"
                            );
                        }
                    }
                    WebSocketEvent::LightningStrike {
                        distance_km,
                        energy,
                        timestamp,
                    } => {
                        // Update WebSocket message timestamp
                        self.websocket_last_message = Some(chrono::Utc::now().timestamp() as u64);

                        // Create LightningEvent for notification
                        let event = crate::weather::LightningEvent {
                            timestamp,
                            distance_km: Some(distance_km),
                            intensity: if energy > 1000.0 {
                                crate::weather::LightningIntensity::High
                            } else if energy > 500.0 {
                                crate::weather::LightningIntensity::Medium
                            } else {
                                crate::weather::LightningIntensity::Low
                            },
                            strike_count: 1,
                            strike_type: crate::weather::LightningStrikeType::CloudToGround,
                        };

                        // Store in state
                        self.lightning_last_epoch = Some(timestamp as i64);
                        self.lightning_last_distance = Some(distance_km);

                        // Trigger flash animation
                        self.start_lightning_flash();

                        // Add to event history (keep last 50 events)
                        self.alert_history
                            .push(crate::weather::AlertEvent::Lightning {
                                timestamp,
                                distance_km: Some(distance_km),
                                strike_count: 1,
                            });
                        if self.alert_history.len() > 50 {
                            self.alert_history.remove(0);
                        }

                        // Send notification if enabled AND per-event enabled AND throttle allows it
                        if self.config.notifications_enabled
                            && self.config.lightning_notifications_enabled
                        {
                            let now = chrono::Utc::now().timestamp() as u64;
                            let should_notify = if let Some(last_notif) =
                                self.last_lightning_notification_time
                            {
                                let elapsed = now.saturating_sub(last_notif);
                                elapsed >= crate::constants::LIGHTNING_NOTIFICATION_COOLDOWN_SECONDS
                            } else {
                                true // First notification
                            };

                            if should_notify {
                                if let Err(e) = self
                                    .notification_manager
                                    .send_lightning_notification(&event, self.config.distance_unit)
                                {
                                    tracing::warn!("Failed to send lightning notification: {}", e);
                                } else {
                                    // Update throttle timestamp only if notification succeeded
                                    self.last_lightning_notification_time = Some(now);

                                    // Log with converted distance for user's unit
                                    let converted_distance =
                                        crate::weather::conversions::convert_distance(
                                            distance_km,
                                            self.config.distance_unit,
                                        );
                                    tracing::info!(
                                        "⚡ Lightning notification sent (distance: {:.1} {})",
                                        converted_distance,
                                        self.config.distance_unit
                                    );
                                }
                            } else {
                                // Log throttled event with converted distance
                                let converted_distance =
                                    crate::weather::conversions::convert_distance(
                                        distance_km,
                                        self.config.distance_unit,
                                    );
                                tracing::debug!(
                                    "⚡ Lightning strike detected ({:.1} {}) but notification throttled (cooldown active)",
                                    converted_distance,
                                    self.config.distance_unit
                                );
                            }
                        }

                        tracing::warn!(
                            "[ALERT] Lightning strike: {:.1} km away, energy: {}",
                            distance_km,
                            energy
                        );
                    }
                    WebSocketEvent::RainStart {
                        device_id,
                        timestamp,
                    } => {
                        // Update WebSocket message timestamp
                        self.websocket_last_message = Some(chrono::Utc::now().timestamp() as u64);

                        // Create RainStartEvent
                        let event = crate::weather::RainStartEvent {
                            timestamp,
                            device_id,
                        };

                        // Store the rain start event
                        self.last_rain_start = Some(event.clone());

                        // Add to event history (keep last 50 events)
                        self.alert_history
                            .push(crate::weather::AlertEvent::Rain { timestamp });
                        if self.alert_history.len() > 50 {
                            self.alert_history.remove(0);
                        }

                        // Send rain start notification if enabled AND per-event enabled AND throttle allows it
                        if self.config.notifications_enabled
                            && self.config.rain_notifications_enabled
                        {
                            let now = chrono::Utc::now().timestamp() as u64;
                            let should_notify =
                                if let Some(last_notif) = self.last_rain_notification_time {
                                    let elapsed = now.saturating_sub(last_notif);
                                    elapsed >= crate::constants::RAIN_NOTIFICATION_COOLDOWN_SECONDS
                                } else {
                                    true // First notification
                                };

                            if should_notify {
                                if let Err(e) = self.notification_manager.send_notification(
                                    "🌧️ Rain Started",
                                    "Precipitation detected at your weather station",
                                    notify_rust::Timeout::Milliseconds(5000),
                                ) {
                                    tracing::warn!("Failed to send rain notification: {}", e);
                                } else {
                                    // Update throttle timestamp only if notification succeeded
                                    self.last_rain_notification_time = Some(now);
                                    tracing::info!("[NOTIFICATION] Rain notification sent");
                                }
                            } else {
                                tracing::debug!(
                                    "🌧️ Rain detected but notification throttled (cooldown active)"
                                );
                            }
                        }

                        tracing::info!("[EVENT] Rain started at device {}", device_id);
                    }
                    WebSocketEvent::ConnectionFailed => {
                        // Update WebSocket status
                        self.websocket_status = crate::weather::WebSocketStatus::Failed;
                        self.websocket_connected = false;

                        tracing::warn!("[WEBSOCKET] WebSocket connection failed");
                    }
                }
                Task::none()
            }
            Message::SetWindSpeedUnit(unit) => {
                self.config.wind_speed_unit = unit;
                self.save_config()
            }
            Message::SetPressureUnit(unit) => {
                self.config.pressure_unit = unit;
                self.save_config()
            }
            Message::SetPrecipitationUnit(unit) => {
                self.config.precipitation_unit = unit;
                self.save_config()
            }
            Message::SetDistanceUnit(unit) => {
                self.config.distance_unit = unit;
                self.save_config()
            }
            Message::ToggleForecastView => {
                self.show_hourly_forecast = !self.show_hourly_forecast;
                self.forecast_loading = true;
                // Reset scroll position when switching views
                self.hourly_forecast_scroll_offset = 0.0;

                // Simulate brief loading animation, then clear loading state
                Task::perform(
                    async {
                        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                    },
                    |_| cosmic::Action::App(Message::ForecastLoadComplete),
                )
            }
            Message::ForecastLoadComplete => {
                self.forecast_loading = false;
                Task::none()
            }
            Message::HourlyForecastScrolled(offset) => {
                self.hourly_forecast_scroll_offset = offset;
                Task::none()
            }
            Message::ShowUnitSelector(selector_type) => {
                self.active_unit_selector = Some(selector_type);
                Task::none()
            }
            Message::CloseUnitSelector => {
                self.active_unit_selector = None;
                Task::none()
            }
            Message::OpenUrl(url) => {
                // Open URL in default browser
                if let Err(e) = open::that(&url) {
                    tracing::error!("Failed to open URL {}: {}", url, e);
                }
                Task::none()
            }
            Message::DetailedObservationReceived(obs) => {
                // Extract detailed sensor data from observation
                self.wind_gust_mps = obs.wind_gust;
                self.sea_level_pressure = obs.sea_level_pressure;
                self.lightning_last_epoch = obs.lightning_strike_last_epoch;
                self.lightning_last_distance = obs.lightning_strike_last_distance;
                self.lightning_count_3h = obs.lightning_strike_count;
                self.rain_amount_1h = obs.precip_accum_last_1hr;
                self.uv_index = obs.uv;
                self.solar_radiation = obs.solar_radiation;
                self.brightness = obs.brightness;
                // Battery voltage needs to come from device status endpoint

                tracing::debug!("Detailed observation data updated");
                Task::none()
            }
            Message::PeriodicHealthCheck => {
                // === STATION CONNECTION MANAGER - Health Check ===

                // Update network availability status
                let network_check = crate::app::state::AppState::check_network_availability();
                Task::perform(network_check, |available| {
                    cosmic::Action::App(Message::NetworkAvailabilityChecked(available))
                })

                // Note: The rest of PeriodicHealthCheck logic will be handled in NetworkAvailabilityChecked
            }
            Message::NetworkAvailabilityChecked(available) => {
                let now = chrono::Utc::now().timestamp() as u64;
                let old_network_state = self.network_available;

                self.network_available = available;

                // Detect network state changes
                if old_network_state != available {
                    if available {
                        tracing::info!("🌐 Network restored - triggering reconnection attempts");
                        // Reset backoff when network comes back
                        self.websocket_reconnect_attempts = 0;

                        // Immediately try to restore station if we have one configured
                        if self.selected_station.is_none()
                            && self.config.selected_station_id.is_some()
                        {
                            tracing::info!(
                                "🔄 Network restored - attempting to reload configured station"
                            );
                            return self.update(Message::FetchStations);
                        }

                        // If we have a station but WebSocket is disconnected, try reconnecting
                        if self.selected_station.is_some() && !self.websocket_connected {
                            tracing::info!(
                                "🔄 Network restored - attempting WebSocket reconnection"
                            );
                            return self.update(Message::WebSocketReconnect);
                        }
                    } else {
                        tracing::warn!("🌐 Network lost - marking all connections as OFFLINE");
                        // Don't trigger reconnects when network is gone
                    }
                }

                // 2. If we have no station but a configured station_id and last fetch failed, retry
                if self.selected_station.is_none()
                    && self.config.selected_station_id.is_some()
                    && self.last_station_fetch_failed
                    && !self.fetching_stations
                    && available
                // Only retry if network is available
                {
                    tracing::warn!(
                        "🔄 No station loaded but configured station exists - retrying FetchStations"
                    );
                    return self.update(Message::FetchStations);
                }

                // Simple WebSocket timeout check - no complex DeviceState logic
                // WebSocket should send data every ~3 seconds (rapid_wind) or ~60s (obs_st)
                if self.websocket_connected {
                    if let Some(last_ws_msg) = self.websocket_last_message {
                        let ws_silence = now.saturating_sub(last_ws_msg);

                        // Disconnect after 120 seconds of silence
                        if ws_silence > 120 {
                            tracing::warn!(
                                "❌ WebSocket timeout: no data for {} seconds - disconnecting",
                                ws_silence
                            );

                            // Update WebSocket status
                            self.websocket_status = crate::weather::WebSocketStatus::Disconnected;
                            self.websocket_connected = false;

                            // Close WebSocket task
                            if let Some(handle) = self.websocket_task_handle.take() {
                                handle.abort();
                            }

                            // Close receiver channel
                            self.websocket_event_receiver.take();

                            // Schedule reconnection
                            return Task::perform(
                                async {
                                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                                },
                                |_| cosmic::Action::App(Message::WebSocketReconnect),
                            );
                        }
                    }
                }

                Task::none()
            }
            Message::ShowStationsView => {
                // Switch to Stations tab
                self.current_tab = AppTab::Stations;

                // Open popup if not already open
                if self.popup.is_none() {
                    let new_id = cosmic::iced::window::Id::unique();
                    self.popup = Some(new_id);

                    let popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        Some((720, 500)),
                        None,
                        None,
                    );
                    cosmic::iced::platform_specific::shell::commands::popup::get_popup(
                        popup_settings,
                    )
                } else {
                    Task::none()
                }
            }
        }
    }

    fn save_config(&self) -> Task<cosmic::Action<Message>> {
        if let Ok(config_context) =
            cosmic_config::Config::new("com.slagmine.astra", Config::VERSION)
        {
            let _ = self.config.write_entry(&config_context);
        }
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
                    country: Some("USA".to_string()),
                },
                is_active,
                last_updated: if is_active {
                    Some(now - data_age_seconds)
                } else {
                    None
                },
                devices: vec![hub_device.clone(), sensor_device.clone()],
            }
        })
        .collect()
}
