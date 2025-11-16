// SPDX-License-Identifier: GPL-3.0-or-later

//! WebSocket implementation for Tempest Weather Station real-time data
//!
//! # Overview
//! This module handles WebSocket connections to Tempest weather stations for
//! real-time data streams including:
//! - Lightning strike events
//! - Rapid wind updates
//! - Precipitation events
//! - Air quality measurements
//!
//! # Important Requirements
//! - WebSocket connection MUST only be established AFTER a station is selected
//! - API key MUST be validated before attempting connection
//! - Connection should gracefully handle disconnects and reconnects
//! - Events should be queued if connection is temporarily lost
//!
//! # Tempest WebSocket API
//! Documentation: https://weatherflow.github.io/Tempest/api/ws.html
//!
//! ## Connection
//! - URL: `wss://ws.weatherflow.com/swd/data`
//! - Authentication: API key via query parameter or message
//!
//! ## Message Types
//! ### Listen Start (Client → Server)
//! ```json
//! {
//!   "type": "listen_start",
//!   "device_id": 12345,
//!   "id": "request-id"
//! }
//! ```
//!
//! ### Lightning Strike Event (Server → Client)
//! ```json
//! {
//!   "type": "evt_strike",
//!   "serial_number": "SK-00001234",
//!   "hub_sn": "HB-00001234",
//!   "evt": [1493164835, 27, 3587]  // [epoch, distance_km, energy]
//! }
//! ```
//!
//! ### Rapid Wind Event (Server → Client)
//! ```json
//! {
//!   "type": "rapid_wind",
//!   "serial_number": "SK-00001234",
//!   "hub_sn": "HB-00001234",
//!   "ob": [1493322445, 2.3, 128]  // [epoch, wind_speed_mps, direction_deg]
//! }
//! ```

use crate::weather::types::*;
use serde::{Deserialize, Serialize};

/// WebSocket connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebSocketState {
    /// Not connected
    Disconnected,
    /// Attempting to connect
    Connecting,
    /// Connected and listening
    Connected,
    /// Connection failed
    Failed,
}

/// WebSocket message types from server
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum TempestWebSocketMessage {
    /// Connection acknowledged (no message field in actual response)
    #[serde(rename = "connection_opened")]
    ConnectionOpened,

    /// Acknowledgment of subscription
    #[serde(rename = "ack")]
    Ack { id: String },

    /// Lightning strike event
    /// evt[0] = Time Epoch (seconds)
    /// evt[1] = Distance (km)
    /// evt[2] = Energy
    #[serde(rename = "evt_strike")]
    LightningStrike { evt: Vec<i64> },

    /// Rain start event
    #[serde(rename = "evt_precip")]
    RainStart { device_id: i64 },

    /// Rapid wind update
    /// ob[0] = Time Epoch (seconds)
    /// ob[1] = Wind Speed (m/s)
    /// ob[2] = Wind Direction (degrees)
    #[serde(rename = "rapid_wind")]
    RapidWind { device_id: i64, ob: Vec<f64> },

    /// Observation data
    #[serde(rename = "obs_st")]
    Observation {
        device_id: i64,
        obs: Vec<Vec<serde_json::Value>>,
        #[serde(default)]
        summary: Option<ObsSummary>,
    },
}

/// Summary data included in obs_st messages
#[derive(Debug, Clone, Deserialize)]
pub struct ObsSummary {
    pub strike_last_epoch: Option<i64>,
    pub strike_last_dist: Option<f64>,
    pub strike_count_3h: Option<u32>,
}

/// WebSocket client message types
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum TempestWebSocketCommand {
    /// Start listening to a device
    #[serde(rename = "listen_start")]
    ListenStart { device_id: i64, id: String },

    /// Listen to rapid wind events
    #[serde(rename = "listen_rapid_start")]
    ListenRapidStart { device_id: i64, id: String },

    /// Stop listening to a device
    #[allow(dead_code)]
    #[serde(rename = "listen_stop")]
    ListenStop { device_id: i64, id: String },

    /// Stop listening to rapid wind events
    #[allow(dead_code)]
    #[serde(rename = "listen_rapid_stop")]
    ListenRapidStop { device_id: i64, id: String },
}

/// WebSocket manager for Tempest connections
pub struct TempestWebSocket {
    state: WebSocketState,
    station_id: Option<i64>,
    api_key: Option<String>,
    event_sender: Option<tokio::sync::mpsc::UnboundedSender<WebSocketEvent>>,
}

/// Events that can be sent from WebSocket to the application
#[derive(Debug, Clone)]
pub enum WebSocketEvent {
    ConnectionEstablishedWithHandle(std::sync::Arc<tokio::task::AbortHandle>),
    ConnectionFailed,
    RapidWind {
        speed_mps: f64,
        direction_deg: f64,
    },
    LightningStrike {
        distance_km: f64,
        energy: f64,
        timestamp: u64,
    },
    RainStart {
        device_id: i64,
        timestamp: u64,
    },
    ObsStation {
        wind_lull: Option<f64>,
        wind_avg: f64,
        wind_gust: Option<f64>,
        wind_direction: f64,
        station_pressure: Option<f64>,
        air_temperature: Option<f64>,
        relative_humidity: Option<u8>,
        illuminance: Option<f64>,
        uv: Option<f64>,
        solar_radiation: Option<f64>,
        precip_accumulated: Option<f64>,
        precip_type: Option<u8>,
        lightning_strike_avg_distance: Option<f64>,
        lightning_strike_count: Option<u32>,
        lightning_strike_last_epoch: Option<i64>,
        battery: Option<f64>,
        report_interval: Option<u16>,
        local_day_rain_accumulation: Option<f64>,
        nc_rain_accumulation: Option<f64>,
        local_day_nc_rain_accumulation: Option<f64>,
    },
}

impl TempestWebSocket {
    /// Create a new WebSocket manager
    ///
    /// Note: Connection is NOT established until `connect()` is called
    pub fn new() -> Self {
        Self {
            state: WebSocketState::Disconnected,
            station_id: None,
            api_key: None,
            event_sender: None,
        }
    }

    /// Create a new WebSocket manager with an event channel
    pub fn new_with_sender(sender: tokio::sync::mpsc::UnboundedSender<WebSocketEvent>) -> Self {
        Self {
            state: WebSocketState::Disconnected,
            station_id: None,
            api_key: None,
            event_sender: Some(sender),
        }
    }

    /// Set the API key for authentication
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    /// Set the station ID to listen to
    ///
    /// # Important
    /// This MUST be called before connecting
    pub fn with_station(mut self, station_id: i64) -> Self {
        self.station_id = Some(station_id);
        self
    }

    /// Connect to the Tempest WebSocket server
    ///
    /// # Returns
    /// - `Ok(AbortHandle)` if connection initiated successfully - handle to abort the message handler task
    /// - `Err` if missing required configuration (API key or station)
    ///
    /// # Requirements
    /// - API key must be set via `with_api_key()`
    /// - Station ID must be set via `with_station()`
    pub async fn connect(&mut self) -> WeatherResult<tokio::task::AbortHandle> {
        use futures_util::{SinkExt, StreamExt};
        use tokio_tungstenite::connect_async;

        if self.api_key.is_none() {
            return Err(WeatherError::ApiError("API key required".to_string()));
        }

        if self.station_id.is_none() {
            return Err(WeatherError::ApiError("Station ID required".to_string()));
        }

        let api_key = self.api_key.as_ref().unwrap();
        let station_id = self.station_id.unwrap();

        // Build WebSocket URL with token
        let ws_url = format!("wss://ws.weatherflow.com/swd/data?token={}", api_key);

        self.state = WebSocketState::Connecting;
        tracing::info!("Connecting to Tempest WebSocket for station {}", station_id);

        // Attempt connection
        match connect_async(&ws_url).await {
            Ok((ws_stream, _)) => {
                self.state = WebSocketState::Connected;
                tracing::info!("WebSocket connected successfully");

                let (mut write, mut read) = ws_stream.split();

                // Send listen_start command for station observations
                let listen_start = TempestWebSocketCommand::ListenStart {
                    device_id: station_id,
                    id: format!("listen-{}", station_id),
                };

                if let Ok(msg_json) = serde_json::to_string(&listen_start) {
                    tracing::info!("[WEBSOCKET] Sending listen_start command: {}", msg_json);
                    if let Err(e) = write
                        .send(tokio_tungstenite::tungstenite::Message::Text(
                            msg_json.into(),
                        ))
                        .await
                    {
                        tracing::error!("[WEBSOCKET] Failed to send listen_start: {}", e);
                        self.state = WebSocketState::Failed;
                        return Err(WeatherError::NetworkError(format!(
                            "Failed to send listen command: {}",
                            e
                        )));
                    }
                    // Flush to ensure message is sent before the next command
                    if let Err(e) = write.flush().await {
                        tracing::error!("[WEBSOCKET] Failed to flush listen_start: {}", e);
                    }
                    tracing::info!("[WEBSOCKET] Sent listen_start for station {}", station_id);
                }

                // Send listen_rapid_start for rapid wind updates
                let listen_rapid = TempestWebSocketCommand::ListenRapidStart {
                    device_id: station_id,
                    id: format!("rapid-{}", station_id),
                };

                if let Ok(msg_json) = serde_json::to_string(&listen_rapid) {
                    tracing::info!(
                        "[WEBSOCKET] Sending listen_rapid_start command: {}",
                        msg_json
                    );
                    if let Err(e) = write
                        .send(tokio_tungstenite::tungstenite::Message::Text(
                            msg_json.into(),
                        ))
                        .await
                    {
                        tracing::error!("[WEBSOCKET] Failed to send listen_rapid_start: {}", e);
                    } else {
                        // Flush to ensure message is sent
                        if let Err(e) = write.flush().await {
                            tracing::error!(
                                "[WEBSOCKET] Failed to flush listen_rapid_start: {}",
                                e
                            );
                        }
                        tracing::info!(
                            "[WEBSOCKET] Sent listen_rapid_start for station {} - expecting rapid_wind or obs_st events",
                            station_id
                        );
                    }
                }

                // Clone the event sender for the background task
                let event_sender = self.event_sender.clone();

                // Spawn background task to handle incoming WebSocket messages
                let message_handler_task = tokio::spawn(async move {
                    tracing::info!(
                        "WebSocket message handler started for station {}",
                        station_id
                    );

                    let mut message_count = 0;
                    let start_time = std::time::Instant::now();
                    let _last_stats_time = std::time::Instant::now();

                    // Log status every 30 seconds
                    let mut status_interval =
                        tokio::time::interval(std::time::Duration::from_secs(30));

                    loop {
                        tokio::select! {
                            msg_result = read.next() => {
                                match msg_result {
                                    Some(msg_result) => {
                                        message_count += 1;
                                        match msg_result {
                            Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                                // Log ALL incoming WebSocket messages (text is Utf8Bytes in 0.28+)
                                let text_str = text.as_str();
                                tracing::debug!("[WEBSOCKET] Message received: {}", text_str);

                                // Parse JSON message
                                match serde_json::from_str::<TempestWebSocketMessage>(text_str) {
                                    Ok(parsed_msg) => {
                                        match parsed_msg {
                                            TempestWebSocketMessage::LightningStrike {
                                                evt,
                                            } => {
                                                if evt.len() >= 3 {
                                                    let distance_km = evt[1] as f64;
                                                    let energy = evt[2] as f64;

                                                    tracing::warn!(
                                                        "⚡ Lightning strike detected! Distance: {} km, Energy: {}",
                                                        distance_km,
                                                        energy
                                                    );

                                                    // Send lightning strike event through channel
                                                    if let Some(ref sender) = event_sender {
                                                        let _ = sender.send(
                                                            WebSocketEvent::LightningStrike {
                                                                distance_km,
                                                                energy,
                                                                timestamp: chrono::Utc::now().timestamp() as u64,
                                                            },
                                                        );
                                                        tracing::debug!("[EVENT] Lightning strike event sent to UI");
                                                    }
                                                }
                                            }
                                            TempestWebSocketMessage::RainStart { device_id } => {
                                                tracing::info!(
                                                    "🌧️  Rain started at station {}",
                                                    device_id
                                                );

                                                // Send rain start event through channel
                                                if let Some(ref sender) = event_sender {
                                                    let _ = sender.send(
                                                        WebSocketEvent::RainStart {
                                                            device_id,
                                                            timestamp: chrono::Utc::now().timestamp() as u64,
                                                        },
                                                    );
                                                    tracing::debug!("[EVENT] Rain start event sent to UI");
                                                }
                                            }
                                            TempestWebSocketMessage::RapidWind {
                                                device_id,
                                                ob,
                                            } => {
                                                if ob.len() >= 3 {
                                                    let wind_speed = ob[1];
                                                    let wind_dir = ob[2];

                                                    tracing::info!(
                                                        "💨 RAPID_WIND: {:.1} m/s from {:.0}° (device: {})",
                                                        wind_speed,
                                                        wind_dir,
                                                        device_id
                                                    );

                                                    // Send wind event through channel
                                                    if let Some(ref sender) = event_sender {
                                                        let _ = sender.send(
                                                            WebSocketEvent::RapidWind {
                                                                speed_mps: wind_speed,
                                                                direction_deg: wind_dir,
                                                            },
                                                        );
                                                        tracing::debug!("[EVENT] Wind event sent to UI");
                                                    }

                                                    // Alert on high wind speeds (>15 m/s = ~33 mph)
                                                    if wind_speed > 15.0 {
                                                        tracing::warn!(
                                                            "💨 High wind gust: {:.1} m/s ({:.0}°)",
                                                            wind_speed,
                                                            wind_dir
                                                        );
                                                    }
                                                }
                                            }
                                            TempestWebSocketMessage::ConnectionOpened => {
                                                tracing::info!(
                                                    "✅ WebSocket connection_opened event received"
                                                );
                                            }
                                            TempestWebSocketMessage::Ack { id } => {
                                                tracing::info!(
                                                    "✅ Subscription acknowledged: {}",
                                                    id
                                                );
                                            }
                                            TempestWebSocketMessage::Observation {
                                                device_id,
                                                obs,
                                                summary,
                                            } => {
                                                // obs_st observation format (Tempest):
                                                // [0] = timestamp (epoch seconds)
                                                // [1] = wind lull (m/s)
                                                // [2] = wind avg (m/s)
                                                // [3] = wind gust (m/s)
                                                // [4] = wind direction (degrees)
                                                // [5] = wind sample interval (seconds)
                                                // [6] = station pressure (MB)
                                                // [7] = air temperature (C)
                                                // [8] = relative humidity (%)
                                                // [9] = illuminance (lux)
                                                // [10] = uv (index)
                                                // [11] = solar radiation (W/m^2)
                                                // [12] = precip accumulated (mm)
                                                // [13] = precip type (0=none, 1=rain, 2=hail, 3=rain+hail)
                                                // [14] = lightning avg distance (km)
                                                // [15] = lightning strike count
                                                // [16] = battery (volts)
                                                // [17] = report interval (minutes)
                                                // [18] = local day rain accumulation (mm)
                                                // [19] = NC rain accumulation (mm)
                                                // [20] = local day NC rain accumulation (mm)
                                                // [21] = precip analysis type (0=none, 1=rain_check, 2=rain_check_with_user_display)

                                                if let Some(observation) = obs.first() {
                                                    if observation.len() >= 19 {
                                                        // Extract all obs_st fields
                                                        let wind_lull = observation[1].as_f64();
                                                        let wind_avg = observation[2].as_f64();
                                                        let wind_gust = observation[3].as_f64();
                                                        let wind_direction = observation[4].as_f64();
                                                        let station_pressure = observation[6].as_f64();
                                                        let air_temperature = observation[7].as_f64();
                                                        let relative_humidity = observation[8].as_u64().map(|v| v as u8);
                                                        let illuminance = observation[9].as_f64();
                                                        let uv = observation[10].as_f64();
                                                        let solar_radiation = observation[11].as_f64();
                                                        let precip_accumulated = observation[12].as_f64();
                                                        let precip_type = observation[13].as_u64().map(|v| v as u8);
                                                        let lightning_strike_avg_distance = observation[14].as_f64();
                                                        let lightning_strike_count = observation[15].as_u64().map(|v| v as u32);
                                                        let battery = observation[16].as_f64();
                                                        let report_interval = observation[17].as_u64().map(|v| v as u16);
                                                        let local_day_rain_accumulation = observation[18].as_f64();
                                                        // NC (noise-corrected) rain fields (indices 19-20)
                                                        let nc_rain_accumulation = if observation.len() > 19 {
                                                            observation[19].as_f64()
                                                        } else {
                                                            None
                                                        };
                                                        let local_day_nc_rain_accumulation = if observation.len() > 20 {
                                                            observation[20].as_f64()
                                                        } else {
                                                            None
                                                        };

                                                        // Extract lightning data from summary (more accurate than obs array)
                                                        let (lightning_last_epoch, lightning_last_dist, lightning_count_3h) = if let Some(ref sum) = summary {
                                                            (sum.strike_last_epoch, sum.strike_last_dist, sum.strike_count_3h.map(|v| v))
                                                        } else {
                                                            (None, lightning_strike_avg_distance, lightning_strike_count)
                                                        };

                                                        if let (Some(wind_avg), Some(wind_dir)) = (wind_avg, wind_direction) {
                                                            tracing::info!(
                                                                "📊 OBS_ST: temp={:.1}°C humidity={}% wind={:.1}m/s dir={:.0}° pressure={:.1}mb (device: {})",
                                                                air_temperature.unwrap_or(0.0),
                                                                relative_humidity.unwrap_or(0),
                                                                wind_avg,
                                                                wind_dir,
                                                                station_pressure.unwrap_or(0.0),
                                                                device_id
                                                            );

                                                            // Send complete observation data
                                                            if let Some(ref sender) = event_sender {
                                                                let _ = sender.send(
                                                                    WebSocketEvent::ObsStation {
                                                                        wind_lull,
                                                                        wind_avg,
                                                                        wind_gust,
                                                                        wind_direction: wind_dir,
                                                                        station_pressure,
                                                                        air_temperature,
                                                                        relative_humidity,
                                                                        illuminance,
                                                                        uv,
                                                                        solar_radiation,
                                                                        precip_accumulated,
                                                                        precip_type,
                                                                        lightning_strike_avg_distance: lightning_last_dist,
                                                                        lightning_strike_count: lightning_count_3h,
                                                                        lightning_strike_last_epoch: lightning_last_epoch,
                                                                        battery,
                                                                        report_interval,
                                                                        local_day_rain_accumulation,
                                                                        nc_rain_accumulation,
                                                                        local_day_nc_rain_accumulation,
                                                                    },
                                                                );
                                                                tracing::debug!("[EVENT] Complete obs_st data sent to UI");
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        tracing::info!(
                                            "⚠️  Failed to parse WebSocket message: {}. Raw: {}",
                                            e,
                                            text
                                        );
                                    }
                                }
                            }
                            Ok(tokio_tungstenite::tungstenite::Message::Close(_)) => {
                                tracing::warn!("WebSocket connection closed by server");
                                break;
                            }
                            Err(e) => {
                                tracing::error!("WebSocket error: {}", e);
                                break;
                            }
                            _ => {}
                                        }
                                    }
                                    None => {
                                        tracing::warn!("WebSocket stream ended");
                                        break;
                                    }
                                }
                            }
                            _ = status_interval.tick() => {
                                let elapsed = start_time.elapsed().as_secs();
                                tracing::info!(
                                    "⏱️  WebSocket status: {} messages in {}s (waiting for obs_st...)",
                                    message_count, elapsed
                                );
                            }
                        }
                    }

                    tracing::info!(
                        "WebSocket message handler stopped for station {} ({} messages total)",
                        station_id,
                        message_count
                    );
                });

                tracing::info!(
                    "WebSocket connection established and listening for station {}",
                    station_id
                );

                // Return the abort handle so the caller can clean up this task
                Ok(message_handler_task.abort_handle())
            }
            Err(e) => {
                self.state = WebSocketState::Failed;
                tracing::error!("WebSocket connection failed: {}", e);
                Err(WeatherError::NetworkError(format!(
                    "WebSocket connection failed: {}",
                    e
                )))
            }
        }
    }
}

impl Default for TempestWebSocket {
    fn default() -> Self {
        Self::new()
    }
}
