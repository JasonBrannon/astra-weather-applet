// SPDX-License-Identifier: GPL-3.0-or-later

use crate::app::{AppState, Message};
use crate::fl;
use crate::weather::{StationHealth, TemperatureUnit, WeatherData, WeatherStation};
use cosmic::Element;
use cosmic::cosmic_theme;
use cosmic::widget::{
    self, button, column, container, divider, horizontal_space, row, scrollable, text,
};

pub fn stations_view(state: &AppState) -> Element<'_, Message> {
    // Get COSMIC theme spacing for density compliance
    let cosmic_theme::Spacing {
        space_xs,
        space_s,
        space_m,
        ..
    } = cosmic::theme::active().cosmic().spacing;

    // Static header section (not scrollable)
    let mut header_content = column()
        .spacing(space_m)
        .width(cosmic::iced::Length::Fill)
        .align_x(cosmic::iced::Alignment::Center);

    // Show station selection prompt if needed
    if state.needs_station_selection {
        header_content = header_content
            .push(station_selection_prompt(state, space_s))
            .push(divider::horizontal::default());
    }

    // Show connection status banner ONLY if there's a selected station AND there are issues
    if state.selected_station.is_some() {
        let has_connection_issues =
            matches!(state.rest_api_status, crate::weather::RestApiStatus::Failed)
                || matches!(
                    state.websocket_status,
                    crate::weather::WebSocketStatus::Failed
                        | crate::weather::WebSocketStatus::Disconnected
                );

        if has_connection_issues {
            header_content = header_content
                .push(connection_status_banner(state, space_s))
                .push(divider::horizontal::default());
        }
    }

    header_content = header_content
        .push(stations_header(state, space_xs))
        .push(divider::horizontal::default());

    // Build the view with static header + scrollable stations list
    column()
        .push(
            container(header_content)
                .padding(space_m)
                .width(cosmic::iced::Length::Fill),
        )
        .push(scrollable(
            container(stations_list(
                &state.available_stations,
                &state.station_health,
                state.config.temperature_unit,
                state.needs_station_selection,
                state.config.selected_station_id.as_deref(),
                state.battery_voltage,
                &state.current_weather,
                state.rest_api_status,
                state.websocket_status,
                space_xs,
                space_s,
                space_m,
            ))
            .padding([0, space_m, space_m, space_m]) // No top padding, keeps side and bottom
            .width(cosmic::iced::Length::Fill),
        ))
        .width(cosmic::iced::Length::Fill)
        .into()
}

fn station_selection_prompt(state: &AppState, space_s: u16) -> Element<'_, Message> {
    let mut content = column()
        .push(
            row()
                .push(widget::icon::from_name("dialog-information-symbolic").size(20))
                .push(text(fl!("stations-selection-required")).size(18))
                .spacing(space_s)
                .align_y(cosmic::iced::Alignment::Center),
        )
        .spacing(space_s);

    if state.fetching_stations {
        content = content.push(text(fl!("stations-selection-loading")).size(14));
    } else if state.available_stations.is_empty() {
        content = content
            .push(text(fl!("stations-selection-no-stations")).size(14))
            .push(button::suggested(fl!("stations-fetch-button")).on_press(Message::FetchStations));
    }

    container(content)
        .padding(space_s)
        .class(cosmic::theme::Container::Card)
        .into()
}

fn connection_status_banner(state: &AppState, space_s: u16) -> Element<'_, Message> {
    // Get COSMIC colors
    let accent = cosmic::theme::active().cosmic().accent_color();
    let warning_color = cosmic::iced::Color::from_rgb(0.95, 0.7, 0.0); // Amber
    let error_color = cosmic::iced::Color::from_rgb(0.9, 0.2, 0.2); // Red

    // Format REST API status (compact) with color
    let (rest_api_text, rest_api_color) = match state.rest_api_status {
        crate::weather::RestApiStatus::NotAttempted => ("Not attempted", warning_color),
        crate::weather::RestApiStatus::Fetching => ("Fetching...", warning_color),
        crate::weather::RestApiStatus::Connected => ("OK", accent.into()),
        crate::weather::RestApiStatus::Failed => ("Failed", error_color),
    };

    // Format WebSocket status (compact) with color
    let (websocket_text, websocket_color) = match state.websocket_status {
        crate::weather::WebSocketStatus::Disconnected => ("Disconnected", error_color),
        crate::weather::WebSocketStatus::Connecting => ("Connecting...", warning_color),
        crate::weather::WebSocketStatus::Connected => ("OK", accent.into()),
        crate::weather::WebSocketStatus::ConnectedIdle => ("Idle", warning_color),
        crate::weather::WebSocketStatus::Failed => ("Failed", error_color),
    };

    // Single-line compact layout with colored status text
    let content = row()
        .push(widget::icon::from_name("dialog-warning-symbolic").size(16))
        .push(text("Connection:").size(14))
        .push(text("API").size(12))
        .push(
            text(rest_api_text)
                .size(13)
                .class(cosmic::theme::Text::Color(rest_api_color)),
        )
        .push(text("•").size(12))
        .push(text("WebSocket").size(12))
        .push(
            text(websocket_text)
                .size(13)
                .class(cosmic::theme::Text::Color(websocket_color)),
        )
        .spacing(space_s)
        .align_y(cosmic::iced::Alignment::Center);

    container(content)
        .padding(space_s)
        .class(cosmic::theme::Container::Card)
        .into()
}

fn stations_header(state: &AppState, space_xs: u16) -> Element<'_, Message> {
    // Get accent color for values
    let accent = cosmic::theme::active().cosmic().accent_color();

    // Active count: 1 if we have a selected station, 0 otherwise
    // Don't rely on API's is_active field which may be stale
    let active_count = if state.selected_station.is_some() {
        1
    } else {
        0
    };
    let total_count = state.available_stations.len();

    // Format last update time using unified helper function
    let last_update_text = match state.last_data_update {
        Some(timestamp) => format_timestamp(timestamp),
        None => fl!("stations-last-updated-never"),
    };

    // Overall connection status based on dual independent statuses
    let (ws_icon, ws_text, ws_color): (&str, String, cosmic::iced::Color) = {
        // Determine overall status from REST API + WebSocket
        let api_ok = matches!(
            state.rest_api_status,
            crate::weather::RestApiStatus::Connected
        );
        let ws_ok = matches!(
            state.websocket_status,
            crate::weather::WebSocketStatus::Connected
        );

        // Get COSMIC accent color for success states
        let accent = cosmic::theme::active().cosmic().accent_color();
        let warning_color = cosmic::iced::Color::from_rgb(0.95, 0.7, 0.0); // Amber/warning
        let error_color = cosmic::iced::Color::from_rgb(0.9, 0.2, 0.2); // Red/error

        match (api_ok, ws_ok) {
            (true, true) => (
                "network-wireless-signal-excellent-symbolic",
                "Connected".to_string(),
                accent.into(), // Success = accent color
            ),
            (true, false) => (
                "network-wireless-signal-ok-symbolic",
                "API only".to_string(),
                warning_color, // Partial = warning amber
            ),
            (false, true) => (
                "network-wireless-signal-weak-symbolic",
                "WS only".to_string(),
                warning_color, // Partial = warning amber
            ),
            (false, false) => (
                "network-wireless-offline-symbolic",
                "Disconnected".to_string(),
                error_color, // Failure = error red
            ),
        }
    };

    let ws_indicator = row()
        .push(widget::icon::from_name(ws_icon).size(14))
        .push(
            text(ws_text)
                .size(13)
                .class(cosmic::theme::Text::Color(ws_color)),
        )
        .spacing(space_xs)
        .align_y(cosmic::iced::Alignment::Center);

    // Simplified header - only show title and essential info
    column()
        .push(
            row()
                .push(text(fl!("stations-title")).size(20))
                .push(horizontal_space())
                .push(crate::ui::components::branding_icon())
                .align_y(cosmic::iced::Alignment::Center),
        )
        .push(
            row()
                .push(text(format!("{} {}", active_count, fl!("stations-active-short"))).size(13))
                .push(text("•").size(13))
                .push(text(format!("{} {}", total_count, fl!("stations-total-short"))).size(13))
                .push(text("•").size(13))
                .push(ws_indicator)
                .push(horizontal_space())
                .push(
                    text(last_update_text)
                        .size(13)
                        .class(cosmic::theme::Text::Color(accent.into())),
                )
                .spacing(space_xs)
                .align_y(cosmic::iced::Alignment::Center),
        )
        .spacing(space_xs)
        .into()
}

fn stations_list<'a>(
    stations: &'a [WeatherStation],
    _health_data: &'a [StationHealth],
    _unit: TemperatureUnit,
    needs_station_selection: bool,
    selected_station_id: Option<&'a str>,
    battery_voltage: Option<f64>,
    current_weather: &'a Option<WeatherData>,
    rest_api_status: crate::weather::RestApiStatus,
    websocket_status: crate::weather::WebSocketStatus,
    space_xs: u16,
    space_s: u16,
    space_m: u16,
) -> Element<'a, Message> {
    // Get accent color for values
    let accent = cosmic::theme::active().cosmic().accent_color();

    if stations.is_empty() {
        return column()
            .push(text(fl!("stations-no-stations-available")).size(18))
            .push(text(fl!("stations-no-stations-configured")).size(14))
            .push(
                button::standard(fl!("stations-discover-button")).on_press(Message::FetchStations),
            )
            .spacing(space_s)
            .into();
    }

    let station_items: Vec<Element<Message>> = stations
        .iter()
        .map(|station| {
            // Check if this is the selected station (for battery display and connection status)
            let is_selected = selected_station_id == Some(&station.id as &str);

            // Use simple station icon (not connection status - that's shown in column 2)
            let icon_name = "weather-few-clouds-symbolic";

            let location_text = match (&station.location.city, &station.location.country) {
                (Some(city), Some(country)) => format!("{}, {}", city, country),
                (Some(city), None) => city.clone(),
                _ => format!(
                    "{:.4}, {:.4}",
                    station.location.latitude, station.location.longitude
                ),
            };

            let last_updated_text = match station.last_updated {
                Some(timestamp) => format_timestamp(timestamp),
                None => fl!("stations-last-updated-never"),
            };

            // HEADER ROW: Station name + Status icon + Active badge (integrated)
            let header = if is_selected {
                // Active station - show station name + Active badge integrated
                row()
                    .push(widget::icon::from_name(icon_name).size(24))
                    .push(text(&station.name).size(20))
                    .push(horizontal_space())
                    .push(
                        container(
                            row()
                                .push(widget::icon::from_name("object-select-symbolic").size(16))
                                .push(
                                    text("Active")
                                        .size(14)
                                        .class(cosmic::theme::Text::Color(accent.into())),
                                )
                                .spacing(space_xs)
                                .align_y(cosmic::iced::Alignment::Center),
                        )
                        .padding([space_xs, space_s])
                        .class(cosmic::theme::Container::ContextDrawer),
                    )
                    .spacing(space_s)
                    .align_y(cosmic::iced::Alignment::Center)
                    .width(cosmic::iced::Length::Fill)
            } else {
                // Inactive station - just show name and status icon
                row()
                    .push(widget::icon::from_name(icon_name).size(24))
                    .push(text(&station.name).size(20))
                    .spacing(space_s)
                    .align_y(cosmic::iced::Alignment::Center)
                    .width(cosmic::iced::Length::Fill)
            };

            // INFO GRID: 3 columns for better space utilization
            let info_row = row()
                .push(
                    // Column 1: Location + Last Update
                    column()
                        .push(
                            row()
                                .push(text("Location").size(11))
                                .push(
                                    text(location_text)
                                        .size(13)
                                        .class(cosmic::theme::Text::Color(accent.into())),
                                )
                                .spacing(space_xs),
                        )
                        .push(
                            row()
                                .push(text("Updated").size(11))
                                .push(
                                    text(last_updated_text)
                                        .size(12)
                                        .class(cosmic::theme::Text::Color(accent.into())),
                                )
                                .spacing(space_xs),
                        )
                        .spacing(space_xs)
                        .width(cosmic::iced::Length::FillPortion(2)),
                )
                .push(
                    // Column 2: Connection Status (REST API + WebSocket) - only for selected station
                    if is_selected {
                        column()
                            .push({
                                // REST API status
                                let rest_api_icon = match rest_api_status {
                                    crate::weather::RestApiStatus::Connected => {
                                        "network-transmit-receive-symbolic"
                                    }
                                    crate::weather::RestApiStatus::Fetching => {
                                        "content-loading-symbolic"
                                    }
                                    crate::weather::RestApiStatus::Failed => {
                                        "network-error-symbolic"
                                    }
                                    crate::weather::RestApiStatus::NotAttempted => {
                                        "network-offline-symbolic"
                                    }
                                };
                                let rest_api_label = match rest_api_status {
                                    crate::weather::RestApiStatus::Connected => "API: OK",
                                    crate::weather::RestApiStatus::Fetching => "API: ...",
                                    crate::weather::RestApiStatus::Failed => "API: Failed",
                                    crate::weather::RestApiStatus::NotAttempted => "API: N/A",
                                };
                                row()
                                    .push(widget::icon::from_name(rest_api_icon).size(14))
                                    .push(text(rest_api_label).size(11))
                                    .spacing(space_xs)
                                    .align_y(cosmic::iced::Alignment::Center)
                            })
                            .push({
                                // WebSocket status
                                let ws_icon = match websocket_status {
                                    crate::weather::WebSocketStatus::Connected => {
                                        "network-wireless-signal-good-symbolic"
                                    }
                                    crate::weather::WebSocketStatus::Connecting => {
                                        "content-loading-symbolic"
                                    }
                                    crate::weather::WebSocketStatus::Disconnected => {
                                        "network-wireless-offline-symbolic"
                                    }
                                    crate::weather::WebSocketStatus::Failed => {
                                        "network-wireless-signal-none-symbolic"
                                    }
                                    crate::weather::WebSocketStatus::ConnectedIdle => {
                                        "network-wireless-signal-weak-symbolic"
                                    }
                                };
                                let ws_label = match websocket_status {
                                    crate::weather::WebSocketStatus::Connected => "WS: OK",
                                    crate::weather::WebSocketStatus::Connecting => "WS: ...",
                                    crate::weather::WebSocketStatus::Disconnected => "WS: Off",
                                    crate::weather::WebSocketStatus::Failed => "WS: Failed",
                                    crate::weather::WebSocketStatus::ConnectedIdle => "WS: Idle",
                                };
                                row()
                                    .push(widget::icon::from_name(ws_icon).size(14))
                                    .push(text(ws_label).size(11))
                                    .spacing(space_xs)
                                    .align_y(cosmic::iced::Alignment::Center)
                            })
                            .spacing(space_xs)
                            .width(cosmic::iced::Length::FillPortion(2))
                    } else {
                        // For non-selected stations, show empty column or placeholder
                        column().width(cosmic::iced::Length::FillPortion(2))
                    },
                )
                .push({
                    // Column 3: Battery (if selected station) or Select button
                    let col3: Element<Message> = if is_selected {
                        let battery_element: Element<Message> = if let Some(voltage) =
                            battery_voltage
                        {
                            let battery_status = crate::ui::components::get_battery_status(voltage);
                            row()
                                .push(text("Battery").size(11))
                                .push(
                                    text(format!("{:.2}V", voltage))
                                        .size(13)
                                        .class(cosmic::theme::Text::Color(accent.into())),
                                )
                                .push(text(format!("({})", battery_status)).size(11))
                                .spacing(space_xs)
                                .into()
                        } else if let Some(weather) = current_weather {
                            if let Some(voltage) = weather.battery_voltage {
                                let battery_status =
                                    crate::ui::components::get_battery_status(voltage);
                                row()
                                    .push(text("Battery").size(11))
                                    .push(
                                        text(format!("{:.2}V", voltage))
                                            .size(13)
                                            .class(cosmic::theme::Text::Color(accent.into())),
                                    )
                                    .push(text(format!("({})", battery_status)).size(11))
                                    .spacing(space_xs)
                                    .into()
                            } else {
                                row().into()
                            }
                        } else {
                            row().into()
                        };

                        column()
                            .push(battery_element)
                            .spacing(space_xs)
                            .width(cosmic::iced::Length::FillPortion(1))
                            .into()
                    } else {
                        // Not selected - show action button
                        column()
                            .push(horizontal_space())
                            .push({
                                if needs_station_selection {
                                    button::suggested(fl!("stations-set-default"))
                                        .on_press(Message::SetDefaultStation(station.clone()))
                                } else {
                                    button::standard(fl!("stations-select-button"))
                                        .on_press(Message::SelectStation(station.clone()))
                                }
                            })
                            .spacing(space_xs)
                            .align_x(cosmic::iced::Alignment::End)
                            .width(cosmic::iced::Length::FillPortion(1))
                            .into()
                    };
                    col3
                })
                .spacing(space_m)
                .width(cosmic::iced::Length::Fill);

            // Assemble the complete station card
            let station_card = column()
                .push(header)
                .push(divider::horizontal::light())
                .push(info_row)
                .spacing(space_s)
                .width(cosmic::iced::Length::Fill);

            container(station_card)
                .padding(space_m)
                .class(cosmic::theme::Container::Card)
                .into()
        })
        .collect();

    column().extend(station_items).spacing(space_s).into()
}

fn format_timestamp(timestamp: u64) -> String {
    use chrono::{DateTime, TimeZone, Utc};
    let dt: DateTime<Utc> = Utc.timestamp_opt(timestamp as i64, 0).unwrap();
    let now = Utc::now();
    let duration = now.signed_duration_since(dt);

    if duration.num_days() > 0 {
        fl!("stations-time-days", value = duration.num_days())
    } else if duration.num_hours() > 0 {
        fl!("stations-time-hours", value = duration.num_hours())
    } else if duration.num_minutes() > 0 {
        fl!("stations-time-minutes", value = duration.num_minutes())
    } else {
        fl!("stations-time-just-now")
    }
}
