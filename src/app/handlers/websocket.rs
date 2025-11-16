// SPDX-License-Identifier: GPL-3.0-or-later

//! WebSocket connection and real-time event handlers

use crate::app::{AppState, Message};
use cosmic::Task;

pub fn handle_websocket_reconnect(state: &mut AppState) -> Task<cosmic::Action<Message>> {
    // Station Connection Manager - WebSocket Reconnect with exponential backoff

    // 1. Check if network is available before attempting
    if !state.network_available {
        tracing::warn!("[NETWORK] Cannot reconnect WebSocket: network unavailable");
        return Task::none();
    }

    // 2. Check if we have a station to connect to
    if state.selected_station.is_none() {
        tracing::warn!("Cannot reconnect WebSocket: no station selected");
        return Task::none();
    }

    // 3. Skip if already connected
    if state.websocket_connected {
        tracing::debug!("WebSocket already connected, skipping reconnect");
        return Task::none();
    }

    // 4. Check exponential backoff timing
    let now = chrono::Utc::now().timestamp() as u64;
    if let Some(last_attempt) = state.last_websocket_reconnect_attempt {
        let delay = state.calculate_reconnect_delay();
        let elapsed = now.saturating_sub(last_attempt);

        if elapsed < delay {
            let remaining = delay.saturating_sub(elapsed);
            tracing::debug!(
                "[RETRY] Reconnect backoff active: {}s remaining (attempt {})",
                remaining,
                state.websocket_reconnect_attempts
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
    state.last_websocket_reconnect_attempt = Some(now);
    state.websocket_reconnect_attempts += 1;

    tracing::info!(
        "[WEBSOCKET] WebSocket reconnection attempt {} (delay: {}s)",
        state.websocket_reconnect_attempts,
        state.calculate_reconnect_delay()
    );

    state.initialize_weather_service_sync();
    Task::none()
}

pub fn handle_websocket_connected(state: &mut AppState) -> Task<cosmic::Action<Message>> {
    state.websocket_connected = true;
    tracing::info!("WebSocket connection status: Connected");
    Task::none()
}

pub fn handle_websocket_disconnected(state: &mut AppState) -> Task<cosmic::Action<Message>> {
    state.websocket_connected = false;
    tracing::warn!("WebSocket connection status: Disconnected");
    Task::none()
}

pub fn handle_websocket_event_received(
    state: &mut AppState,
    event: crate::weather::WebSocketEvent,
) -> Task<cosmic::Action<Message>> {
    use crate::weather::WebSocketEvent;
    match event {
        WebSocketEvent::ConnectionEstablishedWithHandle(handle) => {
            state.websocket_connected = true;
            state.websocket_task_handle = Some(handle);

            // Update WebSocket status
            state.websocket_status = crate::weather::WebSocketStatus::Connected;
            state.websocket_last_connected = Some(chrono::Utc::now().timestamp() as u64);

            // Reset exponential backoff on successful connection
            state.reset_reconnect_backoff();

            tracing::info!(
                "[WEBSOCKET] WebSocket connection established with message handler AbortHandle stored"
            );

            // Only refresh station data if this is a RECONNECTION (not initial connection)
            // BUT: Don't fetch if we just manually reinitialized WebSocket (e.g., after API key change)
            // because that would create a reconnect loop!
            let is_manual_reinit = state.websocket_just_reconnected;

            if state.selected_station.is_some() && !is_manual_reinit {
                // This is a TRUE reconnection scenario (laptop resume, network restore)
                // Immediately fetch fresh station data to update hub/sensor status
                state.websocket_just_reconnected = true;

                tracing::info!(
                    "[WEBSOCKET] WebSocket reconnected (external event) - fetching fresh station data"
                );
                return Task::batch(vec![
                    state.update(Message::FetchStations),
                    state.update(Message::FetchWeatherData),
                ]);
            } else if is_manual_reinit {
                // Manual reinit (from API key change or station reload) - don't trigger fetch loop
                tracing::info!(
                    "[WEBSOCKET] WebSocket connected after manual reinit - skipping station fetch to avoid loop"
                );
                // Reset flag after consuming it
                state.websocket_just_reconnected = false;
            } else {
                // Initial connection during startup - don't interfere with station loading
                tracing::info!(
                    "[WEBSOCKET] Initial WebSocket connection - waiting for station to be loaded"
                );
            }
        }
        WebSocketEvent::RapidWind {
            speed_mps,
            direction_deg,
        } => {
            // Update WebSocket status and timestamp
            state.websocket_status = crate::weather::WebSocketStatus::Connected;
            state.websocket_last_message = Some(chrono::Utc::now().timestamp() as u64);

            // Update separate state fields (used by panel view)
            state.wind_speed_mps = Some(speed_mps);
            state.wind_direction_degrees = Some(direction_deg);

            // ALSO update current_weather (used by weather view)
            if let Some(ref mut weather) = state.current_weather {
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
                state.alert_history.push(crate::weather::AlertEvent::Wind {
                    timestamp,
                    speed_mps,
                    direction_deg,
                });
                if state.alert_history.len() > 50 {
                    state.alert_history.remove(0);
                }
            }

            // Send high wind notification if threshold exceeded AND per-event enabled AND throttle allows it
            if speed_mps > HIGH_WIND_THRESHOLD_MPS
                && state.config.notifications_enabled
                && state.config.wind_notifications_enabled
            {
                let now = chrono::Utc::now().timestamp() as u64;
                let should_notify = if let Some(last_notif) = state.last_wind_notification_time {
                    let elapsed = now.saturating_sub(last_notif);
                    elapsed >= crate::constants::WIND_NOTIFICATION_COOLDOWN_SECONDS
                } else {
                    true // First notification
                };

                if should_notify {
                    // Format wind speed using user's preferred unit
                    let wind_formatted = crate::weather::conversions::format_wind_speed(
                        speed_mps,
                        state.config.wind_speed_unit,
                    );

                    if let Err(e) = state.notification_manager.send_notification(
                        "[ALERT] High Wind Alert",
                        &format!("Wind gust: {} from {}°", wind_formatted, direction_deg),
                        notify_rust::Timeout::Milliseconds(5000),
                    ) {
                        tracing::warn!("Failed to send wind notification: {}", e);
                    } else {
                        // Update throttle timestamp only if notification succeeded
                        state.last_wind_notification_time = Some(now);
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
            state.websocket_last_message = Some(chrono::Utc::now().timestamp() as u64);

            // Update all obs_st fields in current_weather
            if let Some(ref mut weather) = state.current_weather {
                // Wind data
                weather.wind_lull = wind_lull;
                weather.wind_speed = wind_avg;
                weather.wind_gust = wind_gust;
                weather.wind_direction = wind_direction as u16;
                state.wind_speed_mps = Some(wind_avg);
                state.wind_direction_degrees = Some(wind_direction);
                state.wind_gust_mps = wind_gust;

                // Pressure
                if let Some(pressure) = station_pressure {
                    weather.station_pressure = Some(pressure);
                    weather.pressure = pressure;
                    state.sea_level_pressure = Some(pressure);
                }

                // Temperature and humidity
                if let Some(temp) = air_temperature {
                    weather.temperature = temp;
                }
                if let Some(humidity) = relative_humidity {
                    weather.humidity = humidity;

                    // Recalculate dew point if we have both temp and humidity
                    if let Some(temp) = air_temperature {
                        weather.dew_point = Some(
                            crate::weather::conversions::calculate_dew_point(temp, humidity),
                        );
                    }
                }

                // Light measurements
                weather.brightness = illuminance;
                weather.uv = uv;
                weather.solar_radiation = solar_radiation;
                state.brightness = illuminance;
                state.uv_index = uv;
                state.solar_radiation = solar_radiation;

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
                state.lightning_last_distance = lightning_strike_avg_distance;
                if let Some(count) = lightning_strike_count {
                    state.lightning_count_3h = Some(count);
                }
                if let Some(epoch) = lightning_strike_last_epoch {
                    state.lightning_last_epoch = Some(epoch);
                }

                // Battery and reporting
                weather.battery_voltage = battery;
                weather.report_interval = report_interval;
                state.battery_voltage = battery;

                // Update timestamp to current time
                weather.timestamp = chrono::Utc::now().timestamp() as u64;
                state.last_data_update = Some(weather.timestamp);

                tracing::info!(
                    "[DATA] Complete obs_st data received via WebSocket - all metrics updated"
                );

                // Update WebSocket status - receiving obs_st data means connected
                state.websocket_status = crate::weather::WebSocketStatus::Connected;

                tracing::info!(
                    "[DATA] Complete obs_st data received via WebSocket - WebSocket status: Connected"
                );
            }

            // Clear the reconnection flag
            if state.websocket_just_reconnected {
                state.websocket_just_reconnected = false;
                tracing::info!(
                    "[WEBSOCKET] First obs_st after reconnect - WebSocket receiving data again"
                );
            }
        }
        WebSocketEvent::LightningStrike {
            distance_km,
            energy,
            timestamp,
        } => {
            // Update WebSocket message timestamp
            state.websocket_last_message = Some(chrono::Utc::now().timestamp() as u64);

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
            state.lightning_last_epoch = Some(timestamp as i64);
            state.lightning_last_distance = Some(distance_km);

            // Trigger flash animation
            state.start_lightning_flash();

            // Add to event history (keep last 50 events)
            state
                .alert_history
                .push(crate::weather::AlertEvent::Lightning {
                    timestamp,
                    distance_km: Some(distance_km),
                    strike_count: 1,
                });
            if state.alert_history.len() > 50 {
                state.alert_history.remove(0);
            }

            // Send notification if enabled AND per-event enabled AND throttle allows it
            if state.config.notifications_enabled && state.config.lightning_notifications_enabled {
                let now = chrono::Utc::now().timestamp() as u64;
                let should_notify = if let Some(last_notif) = state.last_lightning_notification_time
                {
                    let elapsed = now.saturating_sub(last_notif);
                    elapsed >= crate::constants::LIGHTNING_NOTIFICATION_COOLDOWN_SECONDS
                } else {
                    true // First notification
                };

                if should_notify {
                    if let Err(e) = state
                        .notification_manager
                        .send_lightning_notification(&event, state.config.distance_unit)
                    {
                        tracing::warn!("Failed to send lightning notification: {}", e);
                    } else {
                        // Update throttle timestamp only if notification succeeded
                        state.last_lightning_notification_time = Some(now);

                        // Log with converted distance for user's unit
                        let converted_distance = crate::weather::conversions::convert_distance(
                            distance_km,
                            state.config.distance_unit,
                        );
                        tracing::info!(
                            "[ALERT] Lightning notification sent (distance: {:.1} {})",
                            converted_distance,
                            state.config.distance_unit
                        );
                    }
                } else {
                    // Log throttled event with converted distance
                    let converted_distance = crate::weather::conversions::convert_distance(
                        distance_km,
                        state.config.distance_unit,
                    );
                    tracing::debug!(
                        "[ALERT] Lightning strike detected ({:.1} {}) but notification throttled (cooldown active)",
                        converted_distance,
                        state.config.distance_unit
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
            state.websocket_last_message = Some(chrono::Utc::now().timestamp() as u64);

            // Create RainStartEvent
            let event = crate::weather::RainStartEvent {
                timestamp,
                device_id,
            };

            // Store the rain start event
            state.last_rain_start = Some(event.clone());

            // Add to event history (keep last 50 events)
            state
                .alert_history
                .push(crate::weather::AlertEvent::Rain { timestamp });
            if state.alert_history.len() > 50 {
                state.alert_history.remove(0);
            }

            // Send rain start notification if enabled AND per-event enabled AND throttle allows it
            if state.config.notifications_enabled && state.config.rain_notifications_enabled {
                let now = chrono::Utc::now().timestamp() as u64;
                let should_notify = if let Some(last_notif) = state.last_rain_notification_time {
                    let elapsed = now.saturating_sub(last_notif);
                    elapsed >= crate::constants::RAIN_NOTIFICATION_COOLDOWN_SECONDS
                } else {
                    true // First notification
                };

                if should_notify {
                    if let Err(e) = state.notification_manager.send_notification(
                        "🌧️ Rain Started",
                        "Precipitation detected at your weather station",
                        notify_rust::Timeout::Milliseconds(5000),
                    ) {
                        tracing::warn!("Failed to send rain notification: {}", e);
                    } else {
                        // Update throttle timestamp only if notification succeeded
                        state.last_rain_notification_time = Some(now);
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
            state.websocket_status = crate::weather::WebSocketStatus::Failed;
            state.websocket_connected = false;

            tracing::warn!("[WEBSOCKET] WebSocket connection failed");
        }
    }
    Task::none()
}

pub fn handle_websocket_lightning_strike(
    state: &mut AppState,
    event: crate::weather::LightningEvent,
) -> Task<cosmic::Action<Message>> {
    // Use existing lightning flash animation
    state.start_lightning_flash();

    // Send notification if enabled
    if state.config.notifications_enabled {
        if let Err(e) = state
            .notification_manager
            .send_lightning_notification(&event, state.config.distance_unit)
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

pub fn handle_websocket_rain_start(
    state: &mut AppState,
    event: crate::weather::RainStartEvent,
) -> Task<cosmic::Action<Message>> {
    // Store the rain start event
    state.last_rain_start = Some(event.clone());

    // Send rain start notification if enabled
    if state.config.notifications_enabled {
        if let Err(e) = state.notification_manager.send_notification(
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

pub fn handle_websocket_rapid_wind(
    state: &mut AppState,
    event: crate::weather::RapidWindEvent,
) -> Task<cosmic::Action<Message>> {
    // Store the rapid wind event
    state.last_rapid_wind = Some(event.clone());

    // Update wind speed and direction for compass display
    state.wind_speed_mps = Some(event.wind_speed_mps);
    state.wind_direction_degrees = Some(event.wind_direction_deg);

    // Calculate wind speed in mph for display
    let wind_mph = event.wind_speed_mps * 2.237;

    // Send high wind notification (>33 mph) if enabled
    if wind_mph > 33.0 && state.config.notifications_enabled {
        if let Err(e) = state.notification_manager.send_notification(
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

pub fn handle_websocket_wind_direction(
    state: &mut AppState,
    degrees: f64,
) -> Task<cosmic::Action<Message>> {
    state.wind_direction_degrees = Some(degrees);
    Task::none()
}
