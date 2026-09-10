// SPDX-License-Identifier: GPL-3.0-or-later

use super::messages::AppTab;
use crate::config::Config;
use crate::notifications::NotificationManager;
use crate::weather::{
    AlertEvent, ExtendedForecast, LightningEvent, StationHealth, StationManager, WeatherData,
    WeatherStation,
};

pub struct AppState {
    pub core: cosmic::Core,
    pub config: Config,
    pub popup: Option<cosmic::iced::window::Id>,
    pub current_weather: Option<WeatherData>,
    pub selected_station: Option<WeatherStation>,
    pub lightning_flash_active: bool,
    pub lightning_flash_timer: u64,
    pub notification_manager: NotificationManager,
    pub current_tab: AppTab,
    pub extended_forecast: Option<ExtendedForecast>,
    pub available_stations: Vec<WeatherStation>,
    pub station_health: Vec<StationHealth>,
    pub station_manager: StationManager,
    pub is_locked: bool,
    pub api_key_input: String,
    pub show_api_key_input: bool,
    pub location_search_query: String,
    pub location_search_results: Vec<crate::weather::Location>,
    pub needs_station_selection: bool,
    pub fetching_stations: bool,
    pub websocket_connected: bool,
    pub last_data_update: Option<u64>,
    pub wind_direction_degrees: Option<f64>,
    pub wind_speed_mps: Option<f64>,
    pub wind_gust_mps: Option<f64>,
    pub sea_level_pressure: Option<f64>,
    pub lightning_last_epoch: Option<i64>,
    pub lightning_last_distance: Option<f64>,
    pub lightning_count_3h: Option<u32>,
    pub rain_amount_1h: Option<f64>,
    pub uv_index: Option<f64>,
    pub solar_radiation: Option<f64>,
    pub brightness: Option<f64>,
    pub battery_voltage: Option<f64>,
    pub show_hourly_forecast: bool,
    pub forecast_loading: bool,
    pub last_forecast_update: Option<u64>,
    pub hourly_forecast_scroll_offset: f32,
    pub last_rain_start: Option<crate::weather::RainStartEvent>,
    pub last_rapid_wind: Option<crate::weather::RapidWindEvent>,
    pub websocket_task_handle: Option<std::sync::Arc<tokio::task::AbortHandle>>,
    pub websocket_event_receiver: Option<
        std::sync::Arc<
            tokio::sync::Mutex<
                tokio::sync::mpsc::UnboundedReceiver<crate::weather::WebSocketEvent>,
            >,
        >,
    >,
    // Connection monitoring fields - Independent status indicators
    pub rest_api_status: crate::weather::RestApiStatus, // REST API connection status
    pub rest_api_last_success: Option<u64>,             // Timestamp of last successful API call
    pub rest_api_last_error: Option<String>,            // Last error message from API
    pub websocket_status: crate::weather::WebSocketStatus, // WebSocket connection status
    pub websocket_last_connected: Option<u64>,          // Timestamp when WebSocket last connected
    pub websocket_last_message: Option<u64>,            // Timestamp of last WebSocket message
    pub network_available: bool,                        // OS-level network availability
    pub last_api_attempt_time: Option<u64>,
    pub last_station_fetch_failed: bool,
    pub station_fetch_retry_count: u32,
    pub websocket_reconnect_attempts: u32, // For exponential backoff
    pub last_websocket_reconnect_attempt: Option<u64>, // For backoff timing
    pub websocket_just_reconnected: bool,  // True after reconnect until first obs_st received
    pub active_unit_selector: Option<super::messages::UnitSelectorType>,
    // Notification throttle - prevent notification spam
    pub last_lightning_notification_time: Option<u64>,
    pub last_rain_notification_time: Option<u64>,
    pub last_wind_notification_time: Option<u64>,
    // Event history for alerts view (last 50 events)
    pub alert_history: Vec<AlertEvent>,
    // Time format state - changing this forces UI re-render with new time format
    pub time_format_update_counter: u32,
}

impl AppState {
    pub fn new(core: cosmic::Core, config: Config) -> Self {
        let is_locked = !config.has_api_key();
        let has_station = config.selected_station_id.is_some();

        tracing::warn!(
            "🔧 AppState::new() - selected_station_id from config: {:?}",
            config.selected_station_id
        );

        Self {
            core,
            config,
            popup: None,
            // Don't set mock data - let the fetch flow populate real data
            current_weather: None,
            selected_station: None,
            lightning_flash_active: false,
            lightning_flash_timer: 0,
            notification_manager: NotificationManager::new(),
            // Default tab: Settings if locked, Stations if no station selected, otherwise Forecast
            current_tab: if is_locked {
                AppTab::Settings
            } else if !has_station {
                AppTab::Stations
            } else {
                AppTab::Forecast
            },
            extended_forecast: None,
            available_stations: Vec::new(),
            station_health: Vec::new(),
            station_manager: StationManager::new(),
            is_locked,
            api_key_input: String::new(),
            show_api_key_input: false,
            location_search_query: String::new(),
            location_search_results: Vec::new(),
            needs_station_selection: false,
            fetching_stations: false,
            websocket_connected: false,
            last_data_update: None,
            wind_direction_degrees: None,
            wind_speed_mps: None,
            wind_gust_mps: None,
            sea_level_pressure: None,
            lightning_last_epoch: None,
            lightning_last_distance: None,
            lightning_count_3h: None,
            rain_amount_1h: None,
            uv_index: None,
            solar_radiation: None,
            brightness: None,
            battery_voltage: None,
            show_hourly_forecast: true,
            forecast_loading: false,
            last_forecast_update: None,
            hourly_forecast_scroll_offset: 0.0,
            last_rain_start: None,
            last_rapid_wind: None,
            websocket_task_handle: None,
            websocket_event_receiver: None,
            rest_api_status: crate::weather::RestApiStatus::NotAttempted,
            rest_api_last_success: None,
            rest_api_last_error: None,
            websocket_status: crate::weather::WebSocketStatus::Disconnected,
            websocket_last_connected: None,
            websocket_last_message: None,
            network_available: true, // Assume network is available at startup
            last_api_attempt_time: None,
            last_station_fetch_failed: false,
            station_fetch_retry_count: 0,
            websocket_reconnect_attempts: 0,
            last_websocket_reconnect_attempt: None,
            websocket_just_reconnected: false,
            active_unit_selector: None,
            last_lightning_notification_time: None,
            last_rain_notification_time: None,
            last_wind_notification_time: None,
            alert_history: if crate::constants::DEV_MODE {
                Self::create_mock_alert_history()
            } else {
                Vec::new()
            },
            time_format_update_counter: 0,
        }
    }

    /// Create mock alert history for UI/UX testing (DEV_MODE only)
    fn create_mock_alert_history() -> Vec<AlertEvent> {
        let now = chrono::Utc::now().timestamp() as u64;
        vec![
            // Most recent events first (will be reversed in UI)
            AlertEvent::Lightning {
                timestamp: now - 120, // 2 minutes ago
                distance_km: Some(3.2),
                strike_count: 1,
            },
            AlertEvent::Lightning {
                timestamp: now - 180, // 3 minutes ago
                distance_km: Some(5.7),
                strike_count: 2,
            },
            AlertEvent::Wind {
                timestamp: now - 360, // 6 minutes ago
                speed_mps: 18.5,
                direction_deg: 245.0,
            },
            AlertEvent::Lightning {
                timestamp: now - 540, // 9 minutes ago
                distance_km: Some(12.3),
                strike_count: 1,
            },
            AlertEvent::Rain {
                timestamp: now - 720, // 12 minutes ago
            },
            AlertEvent::Wind {
                timestamp: now - 900, // 15 minutes ago
                speed_mps: 16.2,
                direction_deg: 280.0,
            },
            AlertEvent::Lightning {
                timestamp: now - 1200, // 20 minutes ago
                distance_km: Some(8.9),
                strike_count: 3,
            },
            AlertEvent::Wind {
                timestamp: now - 1500, // 25 minutes ago
                speed_mps: 19.8,
                direction_deg: 270.0,
            },
            AlertEvent::Lightning {
                timestamp: now - 1800, // 30 minutes ago
                distance_km: Some(15.6),
                strike_count: 1,
            },
            AlertEvent::Rain {
                timestamp: now - 2100, // 35 minutes ago
            },
            AlertEvent::Lightning {
                timestamp: now - 2400, // 40 minutes ago
                distance_km: Some(21.4),
                strike_count: 2,
            },
            AlertEvent::Wind {
                timestamp: now - 2700, // 45 minutes ago
                speed_mps: 17.3,
                direction_deg: 255.0,
            },
        ]
    }

    /// Station Connection Manager - Determine unified connection state
    /// Prioritizes WebSocket (fast), uses REST API as fallback (slow)
    ///
    /// Priority:
    /// 1. Network availability (if no network → OFFLINE)
    /// 2. WebSocket status (if connected with recent data → CONNECTED)
    /// 3. REST API status (if reachable but WebSocket down → PARTIAL)
    /// 4. Connection attempt status (if trying → CONNECTING)
    /// 5. Default → DISCONNECTED
    // Removed: determine_connection_state()
    // Connection state is now tracked independently for REST API and WebSocket

    /// Calculate WebSocket reconnect delay with exponential backoff
    /// Returns delay in seconds: 5s → 10s → 20s → 40s → 80s → 120s (max)
    pub fn calculate_reconnect_delay(&self) -> u64 {
        let base_delay = 5; // Start with 5 seconds
        let max_delay = 120; // Cap at 2 minutes

        // Exponential: 5 * 2^attempts
        let delay = base_delay * 2_u64.pow(self.websocket_reconnect_attempts.min(5));
        let capped_delay = delay.min(max_delay);

        tracing::info!(
            "Reconnect delay calculated: {}s (attempt {}, base={}s, max={}s)",
            capped_delay,
            self.websocket_reconnect_attempts + 1,
            base_delay,
            max_delay
        );

        capped_delay
    }

    /// Reset reconnect backoff counter (call on successful connection)
    pub fn reset_reconnect_backoff(&mut self) {
        if self.websocket_reconnect_attempts > 0 {
            tracing::info!(
                "Resetting reconnect backoff (was at attempt {})",
                self.websocket_reconnect_attempts
            );
            self.websocket_reconnect_attempts = 0;
            self.last_websocket_reconnect_attempt = None;
        }
    }

    /// Check if network is available using multiple detection methods
    /// Returns true if any method indicates network availability
    pub async fn check_network_availability() -> bool {
        // Method 1: Try DNS lookup for a reliable host
        match tokio::net::lookup_host("1.1.1.1:53").await {
            Ok(mut addrs) => {
                if addrs.next().is_some() {
                    tracing::debug!("Network check: DNS lookup succeeded (1.1.1.1)");
                    return true;
                }
            }
            Err(e) => {
                tracing::debug!("Network check: DNS lookup failed - {}", e);
            }
        }

        // Method 2: Try Google's public DNS as fallback
        match tokio::net::lookup_host("8.8.8.8:53").await {
            Ok(mut addrs) => {
                if addrs.next().is_some() {
                    tracing::debug!("Network check: DNS lookup succeeded (8.8.8.8)");
                    return true;
                }
            }
            Err(e) => {
                tracing::debug!("Network check: Google DNS lookup failed - {}", e);
            }
        }

        tracing::warn!("Network check: All methods failed - network appears unavailable");
        false
    }

    pub fn start_lightning_flash(&mut self) {
        tracing::debug!("Starting lightning flash animation");
        self.lightning_flash_active = true;
        self.lightning_flash_timer = chrono::Utc::now().timestamp() as u64;
    }

    pub fn update_lightning_flash(&mut self) {
        if self.lightning_flash_active {
            let current_time = chrono::Utc::now().timestamp() as u64;
            // Flash for 3 seconds, then reset
            if current_time - self.lightning_flash_timer > 3 {
                tracing::debug!("Lightning flash animation completed, stopping flash");
                self.lightning_flash_active = false;
            }
        }
    }

    /// Get the last lightning event from state data
    pub fn get_last_lightning_event(&self) -> Option<LightningEvent> {
        if let (Some(epoch), Some(distance), Some(count)) = (
            self.lightning_last_epoch,
            self.lightning_last_distance,
            self.lightning_count_3h,
        ) {
            Some(LightningEvent {
                timestamp: epoch as u64,
                distance_km: Some(distance),
                intensity: crate::weather::LightningIntensity::Medium,
                strike_count: count,
                strike_type: crate::weather::LightningStrikeType::CloudToGround,
            })
        } else {
            None
        }
    }

    /// Unlock the app after a valid API key is provided
    pub fn unlock(&mut self) {
        if self.config.has_api_key() {
            self.is_locked = false;

            // Check if we need to select a default station
            if self.config.selected_station_id.is_none() {
                self.needs_station_selection = true;
                self.current_tab = AppTab::Stations;
                tracing::info!("App unlocked - API key validated, station selection needed");
            } else {
                // Station ID exists in config
                // We'll fetch the actual station data via FetchStations message
                // For now, just set the tab and let the update handler fetch data
                self.current_tab = AppTab::Forecast;
                tracing::info!(
                    "App unlocked - API key validated, station ID in config: {:?}",
                    self.config.selected_station_id
                );
            }

            self.api_key_input.clear();
            self.show_api_key_input = false;
        }
    }

    /// Initialize WebSocket connection for the selected station
    /// This method sets up real-time data streaming for wind, lightning, and observations
    pub fn initialize_weather_service_sync(&mut self) {
        tracing::info!("[WEBSOCKET] Initializing WebSocket connection for selected station");

        // Clean up old WebSocket connection before creating a new one
        if let Some(old_handle) = self.websocket_task_handle.take() {
            tracing::info!("[WEBSOCKET] Aborting old WebSocket task before switching stations");
            old_handle.abort();
        }

        // Drop old receiver to close the channel
        if self.websocket_event_receiver.take().is_some() {
            tracing::info!("[WEBSOCKET] Closed old WebSocket event receiver");
        }

        // Mark as disconnected until new connection is established
        self.websocket_connected = false;

        // Only initialize if we have a selected station
        if let Some(station) = &self.selected_station {
            // Initialize WebSocket connection if we have API key and devices
            if let Some(api_key) = self.config.get_api_key() {
                // IMPORTANT: WebSocket uses device_id, not station_id
                // Find the first Tempest device (type "ST" or "SK")
                if let Some(device) = station
                    .devices
                    .iter()
                    .find(|d| d.device_type == "ST" || d.device_type == "SK")
                {
                    let device_id = device.device_id;
                    let api_key_clone = api_key.clone();

                    tracing::info!(
                        "🔌 Found Tempest device {} (type: {}) for WebSocket connection",
                        device_id,
                        device.device_type
                    );

                    // Create channel for WebSocket events
                    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
                    self.websocket_event_receiver =
                        Some(std::sync::Arc::new(tokio::sync::Mutex::new(rx)));

                    // Clone sender for connection status notification
                    let status_sender = tx.clone();

                    // Spawn WebSocket connection task
                    tokio::spawn(async move {
                        let mut ws = crate::weather::TempestWebSocket::new_with_sender(tx)
                            .with_api_key(api_key_clone)
                            .with_station(device_id);

                        match ws.connect().await {
                            Ok(message_handler_handle) => {
                                tracing::info!(
                                    "✅ WebSocket connected successfully for device {}",
                                    device_id
                                );

                                // Send connection established event with the abort handle
                                let _ = status_sender.send(
                                    crate::weather::WebSocketEvent::ConnectionEstablishedWithHandle(
                                        std::sync::Arc::new(message_handler_handle),
                                    ),
                                );
                            }
                            Err(e) => {
                                tracing::error!("[WEBSOCKET] Failed to connect WebSocket: {}", e);
                                // Send failure event
                                let _ = status_sender
                                    .send(crate::weather::WebSocketEvent::ConnectionFailed);
                            }
                        }
                    });
                } else {
                    tracing::warn!(
                        "Station '{}' has no Tempest devices (ST/SK), cannot connect WebSocket",
                        station.name
                    );
                }
            } else {
                tracing::warn!("No API key available, cannot initialize WebSocket");
            }
        } else {
            tracing::warn!("No station selected, cannot initialize WebSocket");
        }
    }

    /// Lock the app when API key is removed or invalid
    pub fn lock(&mut self) {
        // Clean up WebSocket connection when API key is removed
        if let Some(old_handle) = self.websocket_task_handle.take() {
            tracing::info!("[WEBSOCKET] Aborting WebSocket task (API key removed)");
            old_handle.abort();
        }

        // Drop receiver to close the channel
        if self.websocket_event_receiver.take().is_some() {
            tracing::info!("[WEBSOCKET] Closed WebSocket event receiver (API key removed)");
        }

        self.websocket_connected = false;
        self.is_locked = true;
        self.current_weather = None;
        self.selected_station = None;
        self.current_tab = AppTab::Settings;
        self.api_key_input.clear();
        self.show_api_key_input = false;
        tracing::info!("App locked - no valid API key");
    }
}
