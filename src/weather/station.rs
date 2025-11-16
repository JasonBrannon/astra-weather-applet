// SPDX-License-Identifier: GPL-3.0-or-later

use crate::weather::types::*;
use crate::weather::{TempestObservation, TempestObservationResponse, TempestStationsResponse};

#[derive(Clone)]
pub struct StationManager {
    client: reqwest::Client,
}

impl StationManager {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Get both weather data and detailed observation for sensor metrics
    /// Get detailed station weather observations
    /// NOTE: Always requests data in SI units (Celsius, m/s, mb, mm)
    /// UI layer is responsible for converting to user's preferred units
    pub async fn get_station_weather_detailed(
        &self,
        station: &WeatherStation,
        api_key: &str,
    ) -> WeatherResult<(WeatherData, TempestObservation)> {
        let url = format!(
            "https://swd.weatherflow.com/swd/rest/observations/station/{}",
            station.id
        );

        tracing::info!(
            "[API] Fetching observations API (SI units: °C, m/s, mb, mm) for station {}",
            station.id
        );

        let response = self
            .client
            .get(&url)
            .query(&[
                ("token", api_key),
                ("units_temp", "c"),
                ("units_wind", "mps"),
                ("units_pressure", "mb"),
                ("units_precip", "mm"),
                ("units_distance", "km"),
            ])
            .send()
            .await
            .map_err(|e| WeatherError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(WeatherError::ApiError(format!(
                "Failed to fetch observations: HTTP {}",
                response.status()
            )));
        }

        let obs_response: TempestObservationResponse = response
            .json()
            .await
            .map_err(|e| WeatherError::ParseError(e.to_string()))?;

        if obs_response.status.status_code != 0 {
            return Err(WeatherError::ApiError(format!(
                "API error: {}",
                obs_response.status.status_message
            )));
        }

        let obs = obs_response
            .obs
            .first()
            .ok_or_else(|| WeatherError::ApiError("No observations available".to_string()))?
            .clone();

        // Convert to WeatherData (always returns SI units)
        let weather_data = self.observation_to_weather_data(&obs, &obs_response.station_units)?;

        Ok((weather_data, obs))
    }

    /// Convert TempestObservation to WeatherData
    /// NOTE: Returns all values in SI units (Celsius, m/s, mb, mm)
    /// UI layer is responsible for converting to user's preferred units
    fn observation_to_weather_data(
        &self,
        obs: &TempestObservation,
        _station_units: &Option<StationUnits>,
    ) -> WeatherResult<WeatherData> {
        let temp_value = obs
            .air_temperature
            .ok_or_else(|| WeatherError::ApiError("Temperature data missing".to_string()))?;

        // API always returns SI units (Celsius) - store as-is, UI will convert
        let temperature = temp_value; // Already in Celsius

        // Extract other weather data with defaults (all in SI units from API)
        let humidity = obs.relative_humidity.unwrap_or(0.0) as u8;
        let pressure = obs
            .barometric_pressure
            .or(obs.sea_level_pressure)
            .unwrap_or(1013.25); // mb
        let wind_speed = obs.wind_avg.unwrap_or(0.0); // m/s
        let wind_direction = obs.wind_direction.unwrap_or(0);
        let description = Self::generate_weather_description(obs);

        // Calculate dew point from temperature and humidity
        let dew_point = Some(crate::weather::conversions::calculate_dew_point(
            temperature,
            humidity,
        ));

        // Debug logging for feels_like investigation
        if let Some(fl) = obs.feels_like {
            tracing::debug!(
                "[DATA] Temperature: {:.1}°C, Feels Like: {:.1}°C, Humidity: {}%, Wind: {:.1} m/s",
                temperature,
                fl,
                humidity,
                wind_speed
            );
        }

        Ok(WeatherData {
            temperature,
            humidity,
            pressure,
            wind_speed,
            wind_direction,
            wind_gust: obs.wind_gust,
            description,
            icon: "partly-cloudy-day".to_string(), // Placeholder until Better Forecast updates it
            timestamp: obs.timestamp as u64,
            uv: obs.uv,
            solar_radiation: obs.solar_radiation,
            brightness: obs.brightness,
            feels_like: obs.feels_like,
            precip: obs.precip,
            precip_accum_last_1hr: obs.precip_accum_last_1hr,
            // obs_st WebSocket fields
            wind_lull: obs.wind_lull,
            station_pressure: obs.station_pressure,
            sea_level_pressure: obs.sea_level_pressure,
            dew_point,
            precip_type: None, // Not in REST API, will come from WebSocket
            lightning_avg_distance: obs.lightning_strike_last_distance,
            lightning_strike_count: obs.lightning_strike_count,
            lightning_strike_last_epoch: obs.lightning_strike_last_epoch,
            battery_voltage: None, // Not in REST API, will come from WebSocket
            report_interval: None, // Not in REST API, will come from WebSocket
            local_day_rain_accumulation: None, // Not in REST API, will come from WebSocket
            nc_rain_accumulation: None, // Not in REST API, will come from WebSocket
            local_day_nc_rain_accumulation: None, // Not in REST API, will come from WebSocket
        })
    }

    fn generate_weather_description(obs: &TempestObservation) -> String {
        // Simple weather description based on available data
        let mut conditions = Vec::new();

        if let Some(precip) = obs.precip {
            if precip > 0.0 {
                conditions.push("Rain");
            }
        }

        if let Some(wind) = obs.wind_avg {
            if wind > 10.0 {
                conditions.push("Windy");
            }
        }

        if conditions.is_empty() {
            "Clear".to_string()
        } else {
            conditions.join(", ")
        }
    }

    pub async fn get_all_stations(&self, api_key: &str) -> WeatherResult<Vec<WeatherStation>> {
        let url = "https://swd.weatherflow.com/swd/rest/stations";

        let response = self
            .client
            .get(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await
            .map_err(|e| WeatherError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(WeatherError::ApiError(format!(
                "Failed to fetch stations: HTTP {}",
                response.status()
            )));
        }

        let stations_response: TempestStationsResponse = response
            .json()
            .await
            .map_err(|e| WeatherError::ParseError(e.to_string()))?;

        let stations = stations_response
            .stations
            .into_iter()
            .map(|s| {
                // Extract Hub device (HB)
                let hub_device = s.devices.iter().find(|d| d.device_type == "HB").cloned();

                // Extract Sensor device (ST or SK)
                let sensor_device = s
                    .devices
                    .iter()
                    .find(|d| d.device_type == "ST" || d.device_type == "SK")
                    .cloned();

                // Determine if station is active based on observation age
                let is_active = if let Some(last_obs) = s.last_observation {
                    let now = chrono::Utc::now().timestamp();
                    let age_seconds = now - last_obs;
                    age_seconds < 7200 // Active if observation within 2 hours
                } else {
                    false // No observations = inactive
                };

                if let Some(ref hub) = hub_device {
                    tracing::debug!(
                        "Station '{}': Hub {} found, Sensor {:?}",
                        s.name,
                        hub.serial_number,
                        sensor_device.as_ref().map(|d| &d.serial_number)
                    );
                } else {
                    tracing::warn!("Station '{}': No Hub device found!", s.name);
                }

                WeatherStation {
                    id: s.station_id.to_string(),
                    name: s.name,
                    location: Location {
                        latitude: s.latitude,
                        longitude: s.longitude,
                        city: None,
                        country: None,
                    },
                    is_active,
                    last_updated: s.last_observation.map(|ts| ts as u64),
                    devices: s.devices,
                }
            })
            .collect();

        Ok(stations)
    }

    /// Get better_forecast data from Tempest API
    /// Returns hourly and daily forecast for the specified station
    ///
    /// NOTE: Always requests data in SI units (Celsius, m/s, mb, mm, km)
    /// UI layer is responsible for converting to user's preferred units
    pub async fn get_better_forecast(
        &self,
        station: &WeatherStation,
        api_key: &str,
    ) -> WeatherResult<BetterForecastResponse> {
        let base_url = "https://swd.weatherflow.com/swd/rest/better_forecast";

        // ALWAYS request SI units - UI will convert based on user preferences
        let units_temp = "c"; // Celsius
        let units_wind = "mps"; // meters per second
        let units_pressure = "mb"; // millibars
        let units_precip = "mm"; // millimeters
        let units_distance = "km"; // kilometers

        tracing::info!(
            "[API] Fetching forecast API (SI units: °C, m/s, mb, mm, km) for station {}",
            station.id
        );

        let response = self
            .client
            .get(base_url)
            .query(&[
                ("station_id", station.id.as_str()),
                ("token", api_key),
                ("units_temp", units_temp),
                ("units_wind", units_wind),
                ("units_pressure", units_pressure),
                ("units_precip", units_precip),
                ("units_distance", units_distance),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("[API] Forecast API error {}: {}", status, error_text);
            return Err(WeatherError::ApiError(format!(
                "Forecast API returned {}: {}",
                status, error_text
            )));
        }

        let forecast_response: BetterForecastResponse = response.json().await?;

        tracing::info!(
            "[API] Received forecast: {} hourly, {} daily (SI units: °C, m/s, mb, mm, km)",
            forecast_response.forecast.hourly.len(),
            forecast_response.forecast.daily.len()
        );

        // Log first few hourly icons to verify API data
        for (i, h) in forecast_response.forecast.hourly.iter().take(5).enumerate() {
            tracing::debug!(
                "[DATA] API Hourly[{}]: icon='{}', conditions='{}', local_hour={:?}",
                i,
                h.icon,
                h.conditions,
                h.local_hour
            );
        }

        // Log first few daily icons to verify API data
        for (i, d) in forecast_response.forecast.daily.iter().take(5).enumerate() {
            tracing::debug!(
                "[DATA] API Daily[{}]: icon='{}', conditions='{}', date={:02}/{:02}",
                i,
                d.icon,
                d.conditions,
                d.month_num.unwrap_or(0),
                d.day_num.unwrap_or(0)
            );
        }

        Ok(forecast_response)
    }
}
