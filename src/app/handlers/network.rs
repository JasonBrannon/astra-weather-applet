// SPDX-License-Identifier: GPL-3.0-or-later

//! Network health check and availability handlers

use crate::app::{AppState, Message};
use cosmic::Task;

pub fn handle_periodic_health_check(_state: &mut AppState) -> Task<cosmic::Action<Message>> {
    // === STATION CONNECTION MANAGER - Health Check ===

    // Update network availability status
    let network_check = crate::app::state::AppState::check_network_availability();
    Task::perform(network_check, |available| {
        cosmic::Action::App(Message::NetworkAvailabilityChecked(available))
    })

    // Note: The rest of PeriodicHealthCheck logic will be handled in NetworkAvailabilityChecked
}

pub fn handle_network_availability_checked(
    state: &mut AppState,
    available: bool,
) -> Task<cosmic::Action<Message>> {
    let now = chrono::Utc::now().timestamp() as u64;
    let old_network_state = state.network_available;

    state.network_available = available;

    // Detect network state changes
    if old_network_state != available {
        if available {
            tracing::info!("[NETWORK] Network restored - triggering reconnection attempts");
            // Reset backoff when network comes back
            state.websocket_reconnect_attempts = 0;

            // Immediately try to restore station if we have one configured
            if state.selected_station.is_none() && state.config.selected_station_id.is_some() {
                tracing::info!(
                    "[NETWORK] Network restored - attempting to reload configured station"
                );
                return state.update(Message::FetchStations);
            }

            // If we have a station but WebSocket is disconnected, try reconnecting
            if state.selected_station.is_some() && !state.websocket_connected {
                tracing::info!("[NETWORK] Network restored - attempting WebSocket reconnection");
                return state.update(Message::WebSocketReconnect);
            }
        } else {
            tracing::warn!("[NETWORK] Network lost - marking all connections as OFFLINE");
            // Don't trigger reconnects when network is gone
        }
    }

    // 2. If we have no station but a configured station_id and last fetch failed, retry
    if state.selected_station.is_none()
        && state.config.selected_station_id.is_some()
        && state.last_station_fetch_failed
        && !state.fetching_stations
        && available
    // Only retry if network is available
    {
        tracing::warn!(
            "[NETWORK] No station loaded but configured station exists - retrying FetchStations"
        );
        return state.update(Message::FetchStations);
    }

    // Simple WebSocket timeout check - no complex DeviceState logic
    // WebSocket should send data every ~3 seconds (rapid_wind) or ~60s (obs_st)
    if state.websocket_connected {
        if let Some(last_ws_msg) = state.websocket_last_message {
            let ws_silence = now.saturating_sub(last_ws_msg);

            // Disconnect after 120 seconds of silence
            if ws_silence > 120 {
                tracing::warn!(
                    "[NETWORK] WebSocket timeout: no data for {} seconds - disconnecting",
                    ws_silence
                );

                // Update WebSocket status
                state.websocket_status = crate::weather::WebSocketStatus::Disconnected;
                state.websocket_connected = false;

                // Close WebSocket task
                if let Some(handle) = state.websocket_task_handle.take() {
                    handle.abort();
                }

                // Close receiver channel
                state.websocket_event_receiver.take();

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
