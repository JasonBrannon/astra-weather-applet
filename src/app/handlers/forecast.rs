// SPDX-License-Identifier: GPL-3.0-or-later

//! Forecast data handlers

use crate::app::{AppState, Message};
use cosmic::Task;

pub fn handle_better_forecast_received(
    state: &mut AppState,
    result: Result<crate::weather::BetterForecastResponse, String>,
) -> Task<cosmic::Action<Message>> {
    match result {
        Ok(better_forecast) => {
            // Convert BetterForecastResponse to ExtendedForecast
            // NOTE: All temperatures are in Celsius (SI units from API)
            // UI layer will convert to user's preferred units
            // Icon values from API are preserved and mapped in UI layer
            let location = state
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
                    let date = if let (Some(month), Some(day)) = (d.month_num, d.day_num) {
                        format!("{:02}/{:02}", month, day)
                    } else {
                        // Fallback: format from timestamp
                        let dt = chrono::DateTime::from_timestamp(d.day_start_local as i64, 0)
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

            state.extended_forecast = Some(extended_forecast);
            state.last_forecast_update = Some(chrono::Utc::now().timestamp() as u64);

            // Update current weather icon from Better Forecast current_conditions
            if let Some(ref mut weather) = state.current_weather {
                weather.icon = better_forecast.current_conditions.icon.clone();
                weather.description = better_forecast.current_conditions.conditions.clone();
                tracing::debug!(
                    "[ICON] Updated current weather icon: '{}' ({})",
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

pub fn handle_refresh_forecast(state: &mut AppState) -> Task<cosmic::Action<Message>> {
    // Fetch real forecast data from Tempest API in SI units
    // UI will convert to user's preferred units
    let station_manager = state.station_manager.clone();
    let api_key = state.config.get_api_key();
    let station = state.selected_station.clone();

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
