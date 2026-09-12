// SPDX-License-Identifier: GPL-3.0-or-later

use crate::app::{AppState, Message};
use crate::weather::TemperatureUnit;
use cosmic::Element;
use cosmic::iced::Alignment;
use cosmic::widget::{self, row};

pub(super) struct PanelView<'a> {
    state: &'a AppState,
}

impl<'a> PanelView<'a> {
    pub(super) fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    pub(super) fn render(self) -> Element<'a, Message> {
        render_panel(self.state)
    }
}

fn render_panel(state: &AppState) -> Element<'_, Message> {
    // Get suggested size and padding from panel configuration (needed for both normal and locked views)
    let suggested_size = state.core.applet.suggested_size(true).0;
    let (pad_h, pad_v) = state.core.applet.suggested_padding(true);

    // Icons need to be larger to match text height (1.5x multiplier)
    let icon_size = (suggested_size as f32 * 1.5) as u16;

    // If app is locked, show minimal lockdown view
    if state.is_locked {
        let content = row()
            .push(widget::icon::from_name("changes-prevent-symbolic").size(icon_size))
            .push(state.core.applet.text("--°"))
            .spacing(pad_h)
            .padding([pad_v, pad_h * 2])
            .align_y(Alignment::Center);

        return state
            .core
            .applet
            .autosize_window(cosmic::widget::mouse_area(content).on_press(Message::ToggleWindow))
            .into();
    }

    // Check if we have connection issues (REST API or WebSocket failed)
    let has_connection_issues =
        matches!(state.rest_api_status, crate::weather::RestApiStatus::Failed)
            || matches!(
                state.websocket_status,
                crate::weather::WebSocketStatus::Failed
                    | crate::weather::WebSocketStatus::Disconnected
            );

    // If no weather data AND connection issues, show warning-only view
    if has_connection_issues && state.current_weather.is_none() {
        let status_text = if let Some(station) = &state.selected_station {
            // Calculate time since last successful data
            if let Some(last_update) = station.last_updated {
                let now = chrono::Utc::now().timestamp() as u64;
                let age_seconds = now.saturating_sub(last_update);
                let age_minutes = age_seconds / 60;

                if age_minutes < 60 {
                    format!("{}m", age_minutes)
                } else {
                    format!("{}h", age_minutes / 60)
                }
            } else {
                "offline".to_string()
            }
        } else {
            "offline".to_string()
        };

        let content = row()
            .push(widget::icon::from_name("dialog-warning-symbolic").size(icon_size))
            .push(state.core.applet.text(status_text))
            .spacing(pad_h)
            .padding([pad_v, pad_h * 2])
            .align_y(Alignment::Center);

        return state
            .core
            .applet
            .autosize_window(
                cosmic::widget::mouse_area(content).on_press(Message::ShowStationsView),
            )
            .into();
    }

    // Determine if data is stale (both connections offline)
    let both_offline = matches!(state.rest_api_status, crate::weather::RestApiStatus::Failed)
        && matches!(
            state.websocket_status,
            crate::weather::WebSocketStatus::Failed | crate::weather::WebSocketStatus::Disconnected
        );

    // Normal view - show temperature (safe to cache) but hide wind (unsafe to cache)
    let (temperature_text, weather_icon_value, weather_description) =
        if let Some(weather) = &state.current_weather {
            let temp = match state.config.temperature_unit {
                TemperatureUnit::Celsius => format!("{:.0}°C", weather.temperature),
                TemperatureUnit::Fahrenheit => {
                    let temp_f = weather.temperature * 9.0 / 5.0 + 32.0;
                    format!("{:.0}°F", temp_f)
                }
            };
            (temp, weather.icon.clone(), weather.description.clone())
        } else {
            (
                "--°".to_string(),
                String::from("weather-few-clouds"),
                String::from("Loading"),
            )
        };

    // Use icon_size (1.5x suggested size) to match text height
    let icon_widget = crate::ui::components::weather_icon_widget(
        &weather_icon_value,
        &weather_description,
        icon_size,
    );

    // Build the content row - temperature always shown
    let mut content = row()
        .push(icon_widget)
        .push(state.core.applet.text(temperature_text));

    // Only show wind if we're connected (don't show stale wind data)
    if !both_offline {
        let wind_text = crate::ui::components::format_wind_text(
            state.wind_direction_degrees,
            state.wind_speed_mps,
            state.config.wind_speed_unit,
        );
        let compass_icon = crate::ui::components::compass_icon(
            state.wind_direction_degrees,
            state.wind_speed_mps,
            icon_size,
        );
        content = content
            .push(compass_icon)
            .push(state.core.applet.text(wind_text));
    }

    // Add subtle offline indicator (small dot) instead of warning triangle
    if both_offline && state.current_weather.is_some() {
        content = content.push(
            widget::icon::from_name("network-wireless-offline-symbolic")
                .size((icon_size as f32 * 0.5) as u16), // Tiny indicator
        );
    }

    let content = content
        .spacing(pad_h)
        .padding([pad_v, pad_h * 2])
        .align_y(Alignment::Center);

    // If connection issues, clicking should open Stations tab to show status
    // Otherwise, normal click opens Weather tab
    let click_message = if has_connection_issues {
        Message::ShowStationsView
    } else {
        Message::ToggleWindow
    };

    state
        .core
        .applet
        .autosize_window(cosmic::widget::mouse_area(content).on_press(click_message))
        .into()
}
