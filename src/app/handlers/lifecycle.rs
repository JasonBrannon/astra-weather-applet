// SPDX-License-Identifier: GPL-3.0-or-later

//! Lifecycle and timer handlers
//!
//! This module handles periodic timers, window management, and WebSocket lifecycle events.

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

pub fn handle_websocket_channel_disconnected(
    state: &mut AppState,
) -> Task<cosmic::Action<Message>> {
    if state.websocket_connected {
        tracing::warn!(
            "WebSocket event channel disconnected - will attempt reconnect in 5 seconds"
        );
        state.websocket_connected = false;
    }
    state.websocket_event_receiver = None;

    Task::perform(
        async {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        },
        |_| cosmic::Action::App(Message::WebSocketReconnect),
    )
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
