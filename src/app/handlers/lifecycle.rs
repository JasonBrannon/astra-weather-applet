// SPDX-License-Identifier: GPL-3.0-or-later

//! Lifecycle and timer handlers
//!
//! This module handles periodic timers, window management, and WebSocket polling.

use crate::app::{AppState, Message};
use cosmic::Task;

pub fn handle_tick(state: &mut AppState) -> Task<cosmic::Action<Message>> {
    // Weather data refresh - this happens at the configured interval
    // Also refresh time format detection in case system settings changed
    crate::time_utils::refresh_time_format();

    // Only fetch if we have a station selected
    if state.selected_station.is_some() {
        tracing::debug!("Tick: Refreshing weather data");
        state.update(Message::FetchWeatherData)
    } else {
        Task::none()
    }
}

pub fn handle_forecast_tick(state: &mut AppState) -> Task<cosmic::Action<Message>> {
    // Forecast refresh - happens hourly
    // Also refresh time format detection in case system settings changed
    crate::time_utils::refresh_time_format();

    // Only refresh if we have current weather data
    if state.current_weather.is_some() {
        tracing::debug!("ForecastTick: Refreshing forecast data");
        state.update(Message::RefreshForecast)
    } else {
        Task::none()
    }
}

pub fn handle_websocket_poll(state: &mut AppState) -> Task<cosmic::Action<Message>> {
    // Poll WebSocket event receiver
    if let Some(ref mut receiver) = state.websocket_event_receiver {
        match receiver.try_recv() {
            Ok(event) => {
                return state.update(Message::WebSocketEventReceived(event));
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                // No events available, continue
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                // Only log and update state once when first disconnected
                if state.websocket_connected {
                    tracing::warn!(
                        "WebSocket event channel disconnected - will attempt reconnect in 5 seconds"
                    );
                    state.websocket_connected = false;
                }
                // Clear the receiver to stop polling
                state.websocket_event_receiver = None;

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

pub fn handle_lightning_check(_state: &mut AppState) -> Task<cosmic::Action<Message>> {
    // DEPRECATED: WebSocket now provides real-time lightning events
    // This handler kept for backwards compatibility but does nothing
    // Lightning flash animation is updated via UpdateLightningFlash message
    Task::none()
}

pub fn handle_update_lightning_flash(state: &mut AppState) -> Task<cosmic::Action<Message>> {
    state.update_lightning_flash();
    Task::none()
}

pub fn handle_toggle_window(state: &mut AppState) -> Task<cosmic::Action<Message>> {
    if let Some(id) = state.popup.take() {
        cosmic::iced::platform_specific::shell::commands::popup::destroy_popup(id)
    } else {
        // Check for time format changes when opening popup
        let format_changed = crate::time_utils::refresh_time_format();
        if format_changed {
            tracing::info!("[TIME] Time format changed on popup open - forcing UI update");
            state.time_format_update_counter = state.time_format_update_counter.wrapping_add(1);
        }

        // Force Stations tab if station selection is required
        if state.needs_station_selection && state.current_tab != super::super::messages::AppTab::Stations {
            tracing::info!(
                "[UI] Forcing Stations tab because station selection is required"
            );
            state.current_tab = super::super::messages::AppTab::Stations;
        }

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
    }
}

pub fn handle_popup_closed(
    state: &mut AppState,
    id: cosmic::iced::window::Id,
) -> Task<cosmic::Action<Message>> {
    if state.popup == Some(id) {
        state.popup = None;
    }
    Task::none()
}
