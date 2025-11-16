// SPDX-License-Identifier: GPL-3.0-or-later

//! UI navigation and interaction handlers

use crate::app::messages::{AppTab, UnitSelectorType};
use crate::app::{AppState, Message};
use cosmic::Task;

pub fn handle_select_tab(state: &mut AppState, tab: AppTab) -> Task<cosmic::Action<Message>> {
    state.current_tab = tab;

    // Auto-close unit selector when switching tabs
    if state.active_unit_selector.is_some() {
        state.active_unit_selector = None;
    }

    Task::none()
}

pub fn handle_toggle_forecast_view(state: &mut AppState) -> Task<cosmic::Action<Message>> {
    state.show_hourly_forecast = !state.show_hourly_forecast;
    state.forecast_loading = true;
    // Reset scroll position when switching views
    state.hourly_forecast_scroll_offset = 0.0;

    // Simulate brief loading animation, then clear loading state
    Task::perform(
        async {
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        },
        |_| cosmic::Action::App(Message::ForecastLoadComplete),
    )
}

pub fn handle_forecast_load_complete(state: &mut AppState) -> Task<cosmic::Action<Message>> {
    state.forecast_loading = false;
    Task::none()
}

pub fn handle_hourly_forecast_scrolled(
    state: &mut AppState,
    offset: f32,
) -> Task<cosmic::Action<Message>> {
    state.hourly_forecast_scroll_offset = offset;
    Task::none()
}

pub fn handle_show_unit_selector(
    state: &mut AppState,
    selector_type: UnitSelectorType,
) -> Task<cosmic::Action<Message>> {
    state.active_unit_selector = Some(selector_type);
    Task::none()
}

pub fn handle_close_unit_selector(state: &mut AppState) -> Task<cosmic::Action<Message>> {
    state.active_unit_selector = None;
    Task::none()
}

pub fn handle_open_url(url: String) -> Task<cosmic::Action<Message>> {
    if let Err(e) = open::that(&url) {
        tracing::error!("[UI] Failed to open URL {}: {}", url, e);
    }
    Task::none()
}
