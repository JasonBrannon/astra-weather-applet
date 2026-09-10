// SPDX-License-Identifier: GPL-3.0-or-later

use crate::app::messages::AppTab;
use crate::app::{AppState, Message};
use crate::fl;
use crate::time_utils;
use crate::weather::TemperatureUnit;
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, button, column, container, horizontal_space, row, text, text_input};
use cosmic::{Element, cosmic_theme, theme};

impl AppState {
    pub fn view(&self) -> Element<'_, Message> {
        // Get suggested size and padding from panel configuration (needed for both normal and locked views)
        let suggested_size = self.core.applet.suggested_size(true).0;
        let (pad_h, pad_v) = self.core.applet.suggested_padding(true);

        // Icons need to be larger to match text height (1.5x multiplier)
        let icon_size = (suggested_size as f32 * 1.5) as u16;

        // If app is locked, show minimal lockdown view
        if self.is_locked {
            let content = row()
                .push(widget::icon::from_name("changes-prevent-symbolic").size(icon_size))
                .push(self.core.applet.text("--°"))
                .spacing(pad_h)
                .padding([pad_v, pad_h * 2])
                .align_y(Alignment::Center);

            return self
                .core
                .applet
                .autosize_window(
                    cosmic::widget::mouse_area(content).on_press(Message::ToggleWindow),
                )
                .into();
        }

        // Check if we have connection issues (REST API or WebSocket failed)
        let has_connection_issues =
            matches!(self.rest_api_status, crate::weather::RestApiStatus::Failed)
                || matches!(
                    self.websocket_status,
                    crate::weather::WebSocketStatus::Failed
                        | crate::weather::WebSocketStatus::Disconnected
                );

        // If no weather data AND connection issues, show warning-only view
        if has_connection_issues && self.current_weather.is_none() {
            let status_text = if let Some(station) = &self.selected_station {
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
                .push(self.core.applet.text(status_text))
                .spacing(pad_h)
                .padding([pad_v, pad_h * 2])
                .align_y(Alignment::Center);

            return self
                .core
                .applet
                .autosize_window(
                    cosmic::widget::mouse_area(content).on_press(Message::ShowStationsView),
                )
                .into();
        }

        // Determine if data is stale (both connections offline)
        let both_offline = matches!(self.rest_api_status, crate::weather::RestApiStatus::Failed)
            && matches!(
                self.websocket_status,
                crate::weather::WebSocketStatus::Failed
                    | crate::weather::WebSocketStatus::Disconnected
            );

        // Normal view - show temperature (safe to cache) but hide wind (unsafe to cache)
        let (temperature_text, weather_icon_value, weather_description) =
            if let Some(weather) = &self.current_weather {
                let temp = match self.config.temperature_unit {
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
            .push(self.core.applet.text(temperature_text));

        // Only show wind if we're connected (don't show stale wind data)
        if !both_offline {
            let wind_text = crate::ui::components::format_wind_text(
                self.wind_direction_degrees,
                self.wind_speed_mps,
                self.config.wind_speed_unit,
            );
            let compass_icon = crate::ui::components::compass_icon(
                self.wind_direction_degrees,
                self.wind_speed_mps,
                icon_size,
            );
            content = content
                .push(compass_icon)
                .push(self.core.applet.text(wind_text));
        }

        // Add subtle offline indicator (small dot) instead of warning triangle
        if both_offline && self.current_weather.is_some() {
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

        self.core
            .applet
            .autosize_window(cosmic::widget::mouse_area(content).on_press(click_message))
            .into()
    }

    pub fn view_window(&self, id: cosmic::iced::window::Id) -> Element<'_, Message> {
        // Only show content for our popup window
        if self.popup != Some(id) {
            return text("").into();
        }

        let cosmic_theme::Spacing {
            space_s, space_m, ..
        } = theme::active().cosmic().spacing;

        // Build content based on locked state
        let content = if self.is_locked {
            crate::ui::lockdown_view(self)
        } else {
            // Force Stations tab if station selection is required and we're not already on it
            // This handles the case where popup opens with wrong tab selected
            let should_force_stations =
                self.needs_station_selection && self.current_tab != AppTab::Stations;

            if should_force_stations {
                tracing::debug!("Forcing Stations tab because station selection is required");
            }

            column()
                .push(self.tab_navigation_view(space_s))
                .push(self.tab_content_view(space_s, space_m))
                .spacing(space_m)
                .padding(space_m)
                .into()
        };

        // Use popup_container() like official COSMIC applets (applies correct theme)
        self.core
            .applet
            .popup_container(content)
            .limits(
                cosmic::iced::Limits::NONE
                    .min_width(900.0)
                    .max_width(900.0)
                    .max_height(550.0),
            )
            .into()
    }

    fn get_wind_arrow(degrees: u16) -> &'static str {
        match degrees {
            0..=22 | 338..=360 => "↑",
            23..=67 => "↗",
            68..=112 => "→",
            113..=157 => "↘",
            158..=202 => "↓",
            203..=247 => "↙",
            248..=292 => "←",
            293..=337 => "↖",
            _ => "·",
        }
    }

    /// Get cardinal direction from degrees
    // Use shared 16-point compass function from components module for consistency
    fn get_cardinal_direction(degrees: u16) -> &'static str {
        crate::ui::components::get_cardinal_direction(degrees as f64)
    }

    /// Format temperature without .0 decimal (e.g., "45°F" instead of "45.0°F")
    fn format_temperature_clean(temp: f64, unit: &str) -> String {
        if temp.fract() == 0.0 {
            // Whole number - no decimal
            format!("{:.0}{}", temp, unit)
        } else {
            // Has decimal - show one decimal place
            format!("{:.1}{}", temp, unit)
        }
    }

    fn tab_navigation_view(&self, space_s: u16) -> Element<'_, Message> {
        let has_station = self.selected_station.is_some();

        // If station selection is required, only show Stations tab
        if self.needs_station_selection {
            let tabs = row()
                .push(button::suggested(fl!("tab-stations")))
                .spacing(space_s);

            return container(tabs).width(Length::Fill).into();
        }

        // Use COSMIC-standard button group for tab navigation with proper hierarchy
        let mut tabs = row().spacing(space_s);

        // Forecast tab - requires station (first tab)
        if has_station {
            tabs = tabs.push(if self.current_tab == AppTab::Forecast {
                button::suggested(fl!("tab-forecast"))
                    .on_press(Message::SelectTab(AppTab::Forecast))
            } else {
                button::text(fl!("tab-forecast")).on_press(Message::SelectTab(AppTab::Forecast))
            });
        } else {
            tabs = tabs.push(text(fl!("tab-forecast")).size(14));
        }

        // Weather tab - requires station (second tab)
        if has_station {
            tabs = tabs.push(if self.current_tab == AppTab::Weather {
                button::suggested(fl!("tab-weather")).on_press(Message::SelectTab(AppTab::Weather))
            } else {
                button::text(fl!("tab-weather")).on_press(Message::SelectTab(AppTab::Weather))
            });
        } else {
            tabs = tabs.push(text(fl!("tab-weather")).size(14));
        }

        // Stations tab - always available
        tabs = tabs.push(if self.current_tab == AppTab::Stations {
            button::suggested(fl!("tab-stations")).on_press(Message::SelectTab(AppTab::Stations))
        } else {
            button::text(fl!("tab-stations")).on_press(Message::SelectTab(AppTab::Stations))
        });

        // Alerts tab - requires station (for lightning monitoring)
        if has_station {
            tabs = tabs.push(if self.current_tab == AppTab::Alerts {
                button::suggested(fl!("tab-alerts")).on_press(Message::SelectTab(AppTab::Alerts))
            } else {
                button::text(fl!("tab-alerts")).on_press(Message::SelectTab(AppTab::Alerts))
            });
        } else {
            tabs = tabs.push(text(fl!("tab-alerts")).size(14));
        }

        // Settings tab - always available
        tabs = tabs.push(if self.current_tab == AppTab::Settings {
            button::suggested(fl!("tab-settings")).on_press(Message::SelectTab(AppTab::Settings))
        } else {
            button::text(fl!("tab-settings")).on_press(Message::SelectTab(AppTab::Settings))
        });

        // About tab - always available
        tabs = tabs.push(if self.current_tab == AppTab::About {
            button::suggested(fl!("tab-about")).on_press(Message::SelectTab(AppTab::About))
        } else {
            button::text(fl!("tab-about")).on_press(Message::SelectTab(AppTab::About))
        });

        // Add horizontal space to push weather status to the right
        tabs = tabs.push(horizontal_space());

        // Add weather status to top-right (icon, temperature, description)
        if let Some(weather) = &self.current_weather {
            let temp_display = match self.config.temperature_unit {
                TemperatureUnit::Celsius => {
                    Self::format_temperature_clean(weather.temperature, "°C")
                }
                TemperatureUnit::Fahrenheit => {
                    let temp_f = weather.temperature * 9.0 / 5.0 + 32.0;
                    Self::format_temperature_clean(temp_f, "°F")
                }
            };

            let icon_widget = crate::ui::components::weather_icon_widget(
                &weather.icon,
                &weather.description,
                48, // Larger icon (was 32)
            );

            tabs = tabs
                .push(icon_widget)
                .push(
                    column()
                        .push(text(temp_display).size(24)) // Larger temperature (was 18)
                        .push(text(&weather.description).size(12)) // Larger description (was 10)
                        .spacing(2)
                        .align_x(cosmic::iced::alignment::Horizontal::Center),
                )
                .spacing(space_s)
                .align_y(Alignment::Center);
        }

        tabs.into()
    }

    fn tab_content_view(&self, space_s: u16, space_m: u16) -> Element<'_, Message> {
        let content = match self.current_tab {
            AppTab::Weather => self.weather_tab_view(space_s, space_m),
            AppTab::Forecast => crate::ui::forecast_view(self),
            AppTab::Stations => crate::ui::stations_view(self),
            AppTab::Alerts => self.alerts_tab_view(space_s, space_m),
            AppTab::Settings => self.settings_tab_view(space_s, space_m),
            AppTab::About => self.about_tab_view(space_s, space_m),
        };

        // Wrap in container with fixed height to prevent jumping between tabs
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn weather_tab_view(&self, space_s: u16, space_m: u16) -> Element<'_, Message> {
        let content = column()
            .spacing(space_m)
            .width(Length::Fill)
            .align_x(Alignment::Center)
            .push(if let Some(weather) = &self.current_weather {
                self.clean_weather_details_view(weather, space_s, space_m)
            } else {
                column().push(text(fl!("weather-loading"))).into()
            });

        // No scrollable wrapper - all content should fit
        content.into()
    }

    fn alerts_tab_view(&self, space_s: u16, space_m: u16) -> Element<'_, Message> {
        let accent = cosmic::theme::active().cosmic().accent_color();

        // Compact header with global toggle
        let header = row()
            .push(text(fl!("alerts-title")).size(16))
            .push(horizontal_space())
            .push(widget::icon::from_name("notification-symbolic").size(16))
            .push(text("Desktop Notifications").size(12))
            .push(
                widget::toggler(self.config.notifications_enabled)
                    .on_toggle(Message::SetNotifications),
            )
            .spacing(space_s)
            .align_y(Alignment::Center)
            .width(Length::Fill);

        // Lightning Card with notification toggle
        let lightning_card = {
            let (_icon, status_text, details) =
                if let Some(lightning_event) = self.get_last_lightning_event() {
                    let distance_text = if let Some(distance_km) = lightning_event.distance_km {
                        // Convert to user's preferred distance unit
                        let converted = crate::weather::conversions::convert_distance(
                            distance_km,
                            self.config.distance_unit,
                        );
                        format!("{:.1} {}", converted, self.config.distance_unit)
                    } else {
                        "Unknown".to_string()
                    };
                    let time_text = time_utils::format_time(lightning_event.timestamp);

                    (
                        "weather-storm-symbolic",
                        "Lightning Detected".to_string(),
                        format!(
                            "{} strikes • {} • {}",
                            lightning_event.strike_count, distance_text, time_text
                        ),
                    )
                } else {
                    (
                        "weather-clear-symbolic",
                        fl!("alerts-lightning-none"),
                        "No recent activity".to_string(),
                    )
                };

            widget::container(
                column()
                    .push(
                        row()
                            .push(text(fl!("alerts-lightning-title")).size(14))
                            .push(horizontal_space())
                            .push(
                                widget::toggler(
                                    self.config.lightning_notifications_enabled
                                        && self.config.notifications_enabled,
                                )
                                .on_toggle_maybe(if self.config.notifications_enabled {
                                    Some(Message::SetLightningNotifications)
                                } else {
                                    None
                                })
                                .width(Length::Shrink),
                            )
                            .spacing(8)
                            .align_y(Alignment::Center)
                            .width(Length::Fill),
                    )
                    .push(
                        text(status_text)
                            .size(12)
                            .class(cosmic::theme::Text::Color(accent.into())),
                    )
                    .push(text(details).size(11))
                    .spacing(4)
                    .width(Length::Fill),
            )
            .padding(12)
            .width(Length::FillPortion(1))
            .class(cosmic::theme::Container::Card)
        };

        // Rain Card with notification toggle
        let rain_card = {
            let (_icon, status_text, details) = if let Some(rain_event) = &self.last_rain_start {
                let time_text = time_utils::format_datetime(rain_event.timestamp);
                (
                    "weather-showers-symbolic",
                    "Rain Detected".to_string(),
                    format!("Started: {}", time_text),
                )
            } else {
                (
                    "weather-clear-symbolic",
                    fl!("alerts-rain-none"),
                    "No recent activity".to_string(),
                )
            };

            widget::container(
                column()
                    .push(
                        row()
                            .push(text(fl!("alerts-rain-title")).size(14))
                            .push(horizontal_space())
                            .push(
                                widget::toggler(
                                    self.config.rain_notifications_enabled
                                        && self.config.notifications_enabled,
                                )
                                .on_toggle_maybe(if self.config.notifications_enabled {
                                    Some(Message::SetRainNotifications)
                                } else {
                                    None
                                })
                                .width(Length::Shrink),
                            )
                            .spacing(8)
                            .align_y(Alignment::Center)
                            .width(Length::Fill),
                    )
                    .push(
                        text(status_text)
                            .size(12)
                            .class(cosmic::theme::Text::Color(accent.into())),
                    )
                    .push(text(details).size(11))
                    .spacing(4)
                    .width(Length::Fill),
            )
            .padding(12)
            .width(Length::FillPortion(1))
            .class(cosmic::theme::Container::Card)
        };

        // Wind Card with notification toggle
        let wind_card = {
            let (_icon, status_text, details) = if let Some(wind_event) = &self.last_rapid_wind {
                let wind_speed_converted = crate::weather::convert_wind_speed(
                    wind_event.wind_speed_mps,
                    self.config.wind_speed_unit,
                );
                let wind_mph = wind_event.wind_speed_mps * 2.237;
                let is_high_wind = wind_mph > 33.0;
                let time_text = time_utils::format_time(wind_event.timestamp);

                let status = if is_high_wind {
                    fl!("alerts-wind-high")
                } else {
                    fl!("alerts-wind-recent")
                };

                let detail_text = format!(
                    "{:.1} {} • {}° • {}",
                    wind_speed_converted,
                    self.config.wind_speed_unit,
                    wind_event.wind_direction_deg,
                    time_text
                );

                ("weather-windy-symbolic", status, detail_text)
            } else {
                (
                    "weather-clear-symbolic",
                    fl!("alerts-wind-none"),
                    "No recent activity".to_string(),
                )
            };

            widget::container(
                column()
                    .push(
                        row()
                            .push(text(fl!("alerts-wind-title")).size(14))
                            .push(horizontal_space())
                            .push(
                                widget::toggler(
                                    self.config.wind_notifications_enabled
                                        && self.config.notifications_enabled,
                                )
                                .on_toggle_maybe(if self.config.notifications_enabled {
                                    Some(Message::SetWindNotifications)
                                } else {
                                    None
                                })
                                .width(Length::Shrink),
                            )
                            .spacing(8)
                            .align_y(Alignment::Center)
                            .width(Length::Fill),
                    )
                    .push(
                        text(status_text)
                            .size(12)
                            .class(cosmic::theme::Text::Color(accent.into())),
                    )
                    .push(text(details).size(11))
                    .spacing(4)
                    .width(Length::Fill),
            )
            .padding(12)
            .width(Length::FillPortion(1))
            .class(cosmic::theme::Container::Card)
        };

        // Layout: Header + 3 alert cards in a row (3 columns)
        let cards_row = row()
            .push(lightning_card)
            .push(rain_card)
            .push(wind_card)
            .spacing(space_m)
            .width(Length::Fill);

        // Event History Section
        let history_section = if !self.alert_history.is_empty() {
            let mut history_list = column().spacing(0).width(Length::Fill);

            // Show last 10 events (most recent first)
            for event in self.alert_history.iter().rev().take(10) {
                let (icon, event_text, details) = match event {
                    crate::weather::AlertEvent::Lightning {
                        timestamp,
                        distance_km,
                        strike_count,
                    } => {
                        let time = time_utils::format_datetime(*timestamp);
                        let distance_text = if let Some(dist_km) = distance_km {
                            let converted = crate::weather::conversions::convert_distance(
                                *dist_km,
                                self.config.distance_unit,
                            );
                            format!("{:.1} {} away", converted, self.config.distance_unit)
                        } else {
                            "Unknown distance".to_string()
                        };
                        (
                            "weather-storm-symbolic",
                            format!(
                                "[ALERT] Lightning Strike{}",
                                if *strike_count > 1 { "s" } else { "" }
                            ),
                            format!("{} • {}", distance_text, time),
                        )
                    }
                    crate::weather::AlertEvent::Rain { timestamp } => {
                        let time = time_utils::format_datetime(*timestamp);
                        (
                            "weather-showers-symbolic",
                            "💧 Rain Started".to_string(),
                            time,
                        )
                    }
                    crate::weather::AlertEvent::Wind {
                        timestamp,
                        speed_mps,
                        direction_deg,
                    } => {
                        let time = time_utils::format_datetime(*timestamp);
                        let speed_converted = crate::weather::convert_wind_speed(
                            *speed_mps,
                            self.config.wind_speed_unit,
                        );
                        (
                            "weather-windy-symbolic",
                            "[ALERT] High Wind".to_string(),
                            format!(
                                "{:.1} {} at {}° • {}",
                                speed_converted, self.config.wind_speed_unit, direction_deg, time
                            ),
                        )
                    }
                };

                history_list = history_list.push(
                    container(
                        row()
                            .push(widget::icon::from_name(icon).size(16))
                            .push(
                                column()
                                    .push(text(event_text).size(12))
                                    .push(text(details).size(10))
                                    .spacing(2)
                                    .width(Length::Fill),
                            )
                            .spacing(12)
                            .align_y(Alignment::Center)
                            .width(Length::Fill),
                    )
                    .padding(12)
                    .width(Length::Fill),
                );

                history_list = history_list.push(widget::divider::horizontal::light());
            }

            column()
                .push(text("Recent Activity").size(14))
                .push(
                    widget::scrollable(
                        container(history_list)
                            .padding(0)
                            .width(Length::Fill)
                            .class(cosmic::theme::Container::Card),
                    )
                    .height(Length::Fill),
                )
                .spacing(space_s)
                .width(Length::Fill)
                .height(Length::Fill)
        } else {
            column()
                .push(text("Recent Activity").size(14))
                .push(
                    container(text("No recent events").size(12))
                        .padding(24)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .center_x(Length::Fill)
                        .center_y(Length::Fill)
                        .class(cosmic::theme::Container::Card),
                )
                .spacing(space_s)
                .width(Length::Fill)
                .height(Length::Fill)
        };

        column()
            .push(header)
            .push(cards_row)
            .push(history_section)
            .spacing(space_s)
            .width(Length::Fill)
            .into()
    }

    fn settings_tab_view(&self, space_s: u16, space_m: u16) -> Element<'_, Message> {
        // API Key Management Section - inline implementation
        let api_key_section = if self.show_api_key_input {
            // Show input form to change API key
            column()
                .push(text(fl!("settings-api-key-title")).size(16))
                .push(
                    text_input(fl!("settings-api-key-placeholder"), &self.api_key_input)
                        .on_input(Message::UpdateApiKeyInput)
                        .password()
                        .width(Length::Fill),
                )
                .push(
                    row()
                        .push(button::text(fl!("common-cancel")).on_press(Message::HideApiKeyInput))
                        .push(horizontal_space())
                        .push(
                            button::suggested(fl!("settings-api-key-save")).on_press_maybe(
                                if self.api_key_input.trim().len()
                                    >= crate::constants::MIN_API_KEY_LENGTH
                                {
                                    Some(Message::SetApiKey(self.api_key_input.clone()))
                                } else {
                                    None
                                },
                            ),
                        )
                        .spacing(space_s)
                        .width(Length::Fill),
                )
                .spacing(space_s)
        } else {
            // Show current status with change/remove buttons
            column()
                .push(text(fl!("settings-api-key-title")).size(16))
                .push(
                    row()
                        .push(
                            text(if !self.is_locked {
                                fl!("settings-api-key-configured")
                            } else {
                                fl!("settings-api-key-not-configured")
                            })
                            .size(14),
                        )
                        .push(horizontal_space())
                        .push(if !self.is_locked {
                            button::destructive(fl!("settings-api-key-remove"))
                                .on_press(Message::RemoveApiKey)
                        } else {
                            button::standard(fl!("settings-api-key-add-button"))
                                .on_press(Message::ShowApiKeyInput)
                        })
                        .push(if !self.is_locked {
                            button::standard(fl!("settings-api-key-change"))
                                .on_press(Message::ShowApiKeyInput)
                        } else {
                            button::text("")
                        })
                        .spacing(space_s)
                        .align_y(Alignment::Center)
                        .width(Length::Fill),
                )
                .spacing(space_s)
        };

        let settings_content = widget::scrollable(
            container(
                column()
                    // API Key Management Section (with proper show/hide handling)
                    .push(api_key_section)
                    .push(widget::divider::horizontal::default())
                    // Units Section
                    .push(text(fl!("settings-units-title")).size(16))
                    .push(
                        container(
                            column()
                                .push(crate::ui::compact_unit_row("Temperature", &self.config))
                                .push(widget::divider::horizontal::light())
                                .push(crate::ui::compact_unit_row("Wind Speed", &self.config))
                                .push(widget::divider::horizontal::light())
                                .push(crate::ui::compact_unit_row("Pressure", &self.config))
                                .push(widget::divider::horizontal::light())
                                .push(crate::ui::compact_unit_row("Precipitation", &self.config))
                                .push(widget::divider::horizontal::light())
                                .push(crate::ui::compact_unit_row("Distance", &self.config))
                                .spacing(0)
                                .width(Length::Fill),
                        )
                        .padding(12)
                        .width(Length::Fill)
                        .class(cosmic::theme::Container::Card),
                    )
                    .spacing(space_s)
                    .width(Length::Fill),
            )
            .padding(space_m)
            .width(Length::Fill),
        );

        // If a unit selector is active, show flyout overlaid on top of settings
        if let Some(selector_type) = self.active_unit_selector {
            cosmic::iced::widget::stack![
                settings_content,
                // Right-aligned flyout panel
                row()
                    .push(widget::horizontal_space())
                    .push(self.unit_selector_modal(selector_type, space_s, space_m))
                    .width(Length::Fill),
            ]
            .into()
        } else {
            settings_content.into()
        }
    }

    fn about_tab_view(&self, space_s: u16, space_m: u16) -> Element<'_, Message> {
        let about_content = widget::scrollable(
            container(
                column()
                    // App icon and name
                    .push(
                        column()
                            .push(widget::icon::from_name("com.slagmine.astra-symbolic").size(64))
                            .push(text("Astra Weather Applet").size(24))
                            .push(text(fl!("about-tagline")).size(14))
                            .spacing(space_s)
                            .align_x(cosmic::iced::alignment::Horizontal::Center)
                            .width(Length::Fill),
                    )
                    .push(widget::divider::horizontal::default())
                    // Version info
                    .push(
                        row()
                            .push(text(fl!("about-version")).size(14))
                            .push(horizontal_space())
                            .push(text(env!("CARGO_PKG_VERSION")).size(14))
                            .spacing(space_s)
                            .width(Length::Fill),
                    )
                    .push(widget::divider::horizontal::default())
                    // Description
                    .push(text(fl!("about-description")).size(14))
                    .push(widget::divider::horizontal::default())
                    // Repository link
                    .push(
                        row()
                            .push(text(fl!("about-repository")).size(14))
                            .push(horizontal_space())
                            .push(button::link(env!("CARGO_PKG_REPOSITORY")).on_press(
                                Message::OpenUrl(env!("CARGO_PKG_REPOSITORY").to_string()),
                            ))
                            .spacing(space_s)
                            .width(Length::Fill),
                    )
                    .push(widget::divider::horizontal::default())
                    // License
                    .push(
                        row()
                            .push(text(fl!("about-license")).size(14))
                            .push(horizontal_space())
                            .push(text(env!("CARGO_PKG_LICENSE")).size(14))
                            .spacing(space_s)
                            .width(Length::Fill),
                    )
                    .spacing(space_m)
                    .width(Length::Fill),
            )
            .padding(space_m)
            .width(Length::Fill),
        );

        about_content.into()
    }

    fn unit_selector_modal(
        &self,
        selector_type: crate::app::messages::UnitSelectorType,
        _space_s: u16,
        space_m: u16,
    ) -> Element<'_, Message> {
        use crate::app::messages::UnitSelectorType;

        let title = match selector_type {
            UnitSelectorType::Temperature => "Temperature Unit",
            UnitSelectorType::WindSpeed => "Wind Speed Unit",
            UnitSelectorType::Pressure => "Pressure Unit",
            UnitSelectorType::Precipitation => "Precipitation Unit",
            UnitSelectorType::Distance => "Distance Unit",
        };

        let mut options = column().spacing(0);

        match selector_type {
            UnitSelectorType::Temperature => {
                options = options
                    .push(self.unit_option_button(
                        "Celsius (°C)",
                        self.config.temperature_unit == crate::weather::TemperatureUnit::Celsius,
                        Message::ChangeTemperatureUnit(crate::weather::TemperatureUnit::Celsius),
                    ))
                    .push(widget::divider::horizontal::light())
                    .push(self.unit_option_button(
                        "Fahrenheit (°F)",
                        self.config.temperature_unit == crate::weather::TemperatureUnit::Fahrenheit,
                        Message::ChangeTemperatureUnit(crate::weather::TemperatureUnit::Fahrenheit),
                    ));
            }
            UnitSelectorType::WindSpeed => {
                use crate::weather::WindSpeedUnit;
                options = options
                    .push(self.unit_option_button(
                        "m/s",
                        self.config.wind_speed_unit == WindSpeedUnit::MetersPerSecond,
                        Message::SetWindSpeedUnit(WindSpeedUnit::MetersPerSecond),
                    ))
                    .push(widget::divider::horizontal::light())
                    .push(self.unit_option_button(
                        "mph",
                        self.config.wind_speed_unit == WindSpeedUnit::MilesPerHour,
                        Message::SetWindSpeedUnit(WindSpeedUnit::MilesPerHour),
                    ))
                    .push(widget::divider::horizontal::light())
                    .push(self.unit_option_button(
                        "km/h",
                        self.config.wind_speed_unit == WindSpeedUnit::KilometersPerHour,
                        Message::SetWindSpeedUnit(WindSpeedUnit::KilometersPerHour),
                    ))
                    .push(widget::divider::horizontal::light())
                    .push(self.unit_option_button(
                        "knots",
                        self.config.wind_speed_unit == WindSpeedUnit::Knots,
                        Message::SetWindSpeedUnit(WindSpeedUnit::Knots),
                    ))
                    .push(widget::divider::horizontal::light())
                    .push(self.unit_option_button(
                        "Beaufort",
                        self.config.wind_speed_unit == WindSpeedUnit::Beaufort,
                        Message::SetWindSpeedUnit(WindSpeedUnit::Beaufort),
                    ))
                    .push(widget::divider::horizontal::light())
                    .push(self.unit_option_button(
                        "ft/min",
                        self.config.wind_speed_unit == WindSpeedUnit::FeetPerMinute,
                        Message::SetWindSpeedUnit(WindSpeedUnit::FeetPerMinute),
                    ));
            }
            UnitSelectorType::Pressure => {
                use crate::weather::PressureUnit;
                options = options
                    .push(self.unit_option_button(
                        "mb",
                        self.config.pressure_unit == PressureUnit::Millibar,
                        Message::SetPressureUnit(PressureUnit::Millibar),
                    ))
                    .push(widget::divider::horizontal::light())
                    .push(self.unit_option_button(
                        "hPa",
                        self.config.pressure_unit == PressureUnit::Hectopascal,
                        Message::SetPressureUnit(PressureUnit::Hectopascal),
                    ))
                    .push(widget::divider::horizontal::light())
                    .push(self.unit_option_button(
                        "inHg",
                        self.config.pressure_unit == PressureUnit::InchesOfMercury,
                        Message::SetPressureUnit(PressureUnit::InchesOfMercury),
                    ))
                    .push(widget::divider::horizontal::light())
                    .push(self.unit_option_button(
                        "mmHg",
                        self.config.pressure_unit == PressureUnit::MillimetersOfMercury,
                        Message::SetPressureUnit(PressureUnit::MillimetersOfMercury),
                    ));
            }
            UnitSelectorType::Precipitation => {
                use crate::weather::PrecipitationUnit;
                options = options
                    .push(self.unit_option_button(
                        "mm",
                        self.config.precipitation_unit == PrecipitationUnit::Millimeters,
                        Message::SetPrecipitationUnit(PrecipitationUnit::Millimeters),
                    ))
                    .push(widget::divider::horizontal::light())
                    .push(self.unit_option_button(
                        "cm",
                        self.config.precipitation_unit == PrecipitationUnit::Centimeters,
                        Message::SetPrecipitationUnit(PrecipitationUnit::Centimeters),
                    ))
                    .push(widget::divider::horizontal::light())
                    .push(self.unit_option_button(
                        "inches",
                        self.config.precipitation_unit == PrecipitationUnit::Inches,
                        Message::SetPrecipitationUnit(PrecipitationUnit::Inches),
                    ));
            }
            UnitSelectorType::Distance => {
                use crate::weather::DistanceUnit;
                options = options
                    .push(self.unit_option_button(
                        "km",
                        self.config.distance_unit == DistanceUnit::Kilometers,
                        Message::SetDistanceUnit(DistanceUnit::Kilometers),
                    ))
                    .push(widget::divider::horizontal::light())
                    .push(self.unit_option_button(
                        "miles",
                        self.config.distance_unit == DistanceUnit::Miles,
                        Message::SetDistanceUnit(DistanceUnit::Miles),
                    ));
            }
        }

        container(
            column()
                .push(
                    container(
                        row()
                            .push(text(title).size(20))
                            .push(widget::horizontal_space())
                            .push(
                                button::text(fl!("common-close"))
                                    .on_press(Message::CloseUnitSelector),
                            )
                            .align_y(Alignment::Center)
                            .width(Length::Fill),
                    )
                    .padding(space_m),
                )
                .push(widget::divider::horizontal::default())
                .push(widget::scrollable(
                    container(options).padding(space_m).width(Length::Fill),
                ))
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .class(cosmic::theme::Container::Card) // Use Card class for elevated appearance
        .width(Length::Fixed(400.0))
        .height(Length::Fill)
        .into()
    }

    fn unit_option_button<'a>(
        &self,
        label: &'static str,
        is_selected: bool,
        on_press: Message,
    ) -> Element<'a, Message> {
        button::custom(
            row()
                .push(text(label).size(14).width(Length::Fill))
                .push_maybe(if is_selected {
                    Some(widget::icon::from_name("object-select-symbolic").size(16))
                } else {
                    None
                })
                .align_y(Alignment::Center)
                .spacing(8)
                .width(Length::Fill),
        )
        .on_press(on_press)
        .padding(12)
        .width(Length::Fill)
        .class(if is_selected {
            cosmic::theme::Button::Suggested
        } else {
            cosmic::theme::Button::Text
        })
        .into()
    }

    // Command & Control Dashboard Helpers

    /// Create a single metric row with label, value, and description (readable fonts)
    fn metric_row<'a>(
        label: &'static str,
        value: String,
        description: &str,
        accent: cosmic::cosmic_theme::palette::Alpha<cosmic::cosmic_theme::palette::rgb::Rgb, f32>,
    ) -> Element<'a, Message> {
        row()
            .push(
                column()
                    .push(text(label).size(13))
                    .push(text(description.to_string()).size(11))
                    .spacing(2)
                    .width(Length::Fixed(140.0)),
            )
            .push(
                text(value)
                    .size(18)
                    .class(cosmic::theme::Text::Color(accent.into()))
                    .wrapping(cosmic::iced_core::text::Wrapping::None),
            )
            .spacing(12)
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .into()
    }

    /// Create a section header with icon and title
    fn section_header<'a>(title: &'static str, icon: &'static str) -> Element<'a, Message> {
        row()
            .push(widget::icon::from_name(icon).size(14))
            .push(text(title).size(13))
            .spacing(6)
            .align_y(Alignment::Center)
            .into()
    }

    fn clean_weather_details_view<'a>(
        &self,
        weather: &'a crate::weather::WeatherData,
        space_s: u16,
        space_m: u16,
    ) -> Element<'a, Message> {
        use crate::weather::conversions::*;

        // Get accent color for values
        let accent = theme::active().cosmic().accent_color();

        // === LEFT COLUMN ===
        let mut left_col = column().spacing(8).width(Length::FillPortion(1));

        // ATMOSPHERIC CONDITIONS
        left_col = left_col
            .push(Self::section_header(
                "ATMOSPHERIC",
                "weather-overcast-symbolic",
            ))
            .push(widget::divider::horizontal::light());

        if let Some(feels_like) = weather.feels_like {
            let feels_like_display = format_temperature(feels_like, self.config.temperature_unit);
            let actual_temp_display =
                format_temperature(weather.temperature, self.config.temperature_unit);

            tracing::debug!(
                "[DEBUG] Display: Actual Temperature: {}, Feels Like: {} (Unit: {:?})",
                actual_temp_display,
                feels_like_display,
                self.config.temperature_unit
            );

            left_col = left_col.push(Self::metric_row(
                "Feels Like",
                feels_like_display,
                "Apparent temp",
                accent,
            ));
        }
        left_col = left_col.push(Self::metric_row(
            "Humidity",
            format!("{}%", weather.humidity),
            "Moisture in air",
            accent,
        ));
        if let Some(dew_point) = weather.dew_point {
            left_col = left_col.push(Self::metric_row(
                "Dew Point",
                format_temperature(dew_point, self.config.temperature_unit),
                "Condensation point",
                accent,
            ));
        }
        left_col = left_col.push(Self::metric_row(
            "Pressure",
            format_pressure(weather.pressure, self.config.pressure_unit),
            "Barometric pressure",
            accent,
        ));

        // WIND CONDITIONS
        left_col = left_col
            .push(widget::Space::with_height(Length::Fixed(16.0)))
            .push(Self::section_header("WIND", "weather-windy-symbolic"))
            .push(widget::divider::horizontal::light());

        let is_calm = weather.wind_speed < 0.5;
        let wind_display = if is_calm {
            "Calm".to_string()
        } else {
            let wind_arrow = Self::get_wind_arrow(weather.wind_direction);
            let wind_cardinal = Self::get_cardinal_direction(weather.wind_direction);
            format!(
                "{} {} {}",
                wind_cardinal,
                wind_arrow,
                format_wind_speed(weather.wind_speed, self.config.wind_speed_unit)
            )
        };
        left_col = left_col.push(Self::metric_row(
            "Current",
            wind_display,
            "Speed & direction",
            accent,
        ));
        if let Some(lull) = weather.wind_lull {
            left_col = left_col.push(Self::metric_row(
                "Lull",
                format_wind_speed(lull, self.config.wind_speed_unit),
                "Minimum speed",
                accent,
            ));
        }
        if let Some(gust) = weather.wind_gust {
            left_col = left_col.push(Self::metric_row(
                "Gust",
                format_wind_speed(gust, self.config.wind_speed_unit),
                "Maximum speed",
                accent,
            ));
        }

        // === RIGHT COLUMN ===
        let mut right_col = column().spacing(8).width(Length::FillPortion(1));

        // PRECIPITATION
        right_col = right_col
            .push(Self::section_header(
                "PRECIPITATION",
                "weather-showers-symbolic",
            ))
            .push(widget::divider::horizontal::light());

        if let Some(precip_1hr) = weather.precip_accum_last_1hr {
            right_col = right_col.push(Self::metric_row(
                "Last Hour",
                format_precipitation(precip_1hr, self.config.precipitation_unit),
                "Past 60 minutes",
                accent,
            ));
        }
        if let Some(rain_today) = weather.local_day_rain_accumulation {
            right_col = right_col.push(Self::metric_row(
                "Today",
                format_precipitation(rain_today, self.config.precipitation_unit),
                "Daily total",
                accent,
            ));
        }
        if let Some(precip_type) = weather.precip_type {
            let precip_text = crate::ui::components::get_precip_type_text(precip_type);
            if precip_text != "None" {
                right_col = right_col.push(Self::metric_row(
                    "Type",
                    precip_text.to_string(),
                    "Precip category",
                    accent,
                ));
            }
        }
        // Always show at least one precipitation row
        if weather.precip_accum_last_1hr.is_none()
            && weather.local_day_rain_accumulation.is_none()
            && weather.precip_type.is_none()
        {
            right_col = right_col.push(Self::metric_row(
                "Status",
                "None".to_string(),
                "No precipitation",
                accent,
            ));
        }

        // SOLAR & LIGHT
        right_col = right_col
            .push(widget::Space::with_height(Length::Fixed(16.0)))
            .push(Self::section_header(
                "SOLAR & LIGHT",
                "weather-clear-symbolic",
            ))
            .push(widget::divider::horizontal::light());

        if let Some(uv) = weather.uv {
            right_col = right_col.push(Self::metric_row(
                "UV Index",
                format!("{:.1}", uv),
                "UV radiation level",
                accent,
            ));
        }
        if let Some(solar) = weather.solar_radiation {
            right_col = right_col.push(Self::metric_row(
                "Solar Radiation",
                format!("{} W/m²", solar),
                "Incoming energy",
                accent,
            ));
        }
        if let Some(brightness) = weather.brightness {
            right_col = right_col.push(Self::metric_row(
                "Illuminance",
                format!("{} lux", brightness),
                "Light level",
                accent,
            ));
        }
        // Show placeholder if no solar data
        if weather.uv.is_none() && weather.solar_radiation.is_none() && weather.brightness.is_none()
        {
            right_col = right_col.push(Self::metric_row(
                "Status",
                "No Data".to_string(),
                "Sensors offline",
                accent,
            ));
        }

        // LIGHTNING ACTIVITY
        right_col = right_col
            .push(widget::Space::with_height(Length::Fixed(16.0)))
            .push(Self::section_header("LIGHTNING", "weather-storm-symbolic"))
            .push(widget::divider::horizontal::light());

        if let Some(last_epoch) = weather.lightning_strike_last_epoch {
            let now = chrono::Utc::now().timestamp();
            let diff = now - last_epoch;
            let last_str = if diff < 60 {
                format!("{}s ago", diff)
            } else if diff < 3600 {
                format!("{}m ago", diff / 60)
            } else if diff < 86400 {
                format!("{}h ago", diff / 3600)
            } else {
                format!("{}d ago", diff / 86400)
            };
            right_col = right_col.push(Self::metric_row(
                "Last Strike",
                last_str,
                "Most recent",
                accent,
            ));
        }
        if let Some(distance) = weather.lightning_avg_distance {
            let converted = convert_distance(distance, self.config.distance_unit);
            right_col = right_col.push(Self::metric_row(
                "Distance",
                format!("{:.1} {}", converted, self.config.distance_unit),
                "Average distance",
                accent,
            ));
        }
        if let Some(count) = weather.lightning_strike_count {
            right_col = right_col.push(Self::metric_row(
                "Strikes (3h)",
                format!("{}", count),
                "Recent count",
                accent,
            ));
        }
        // Show placeholder if no lightning
        if weather.lightning_strike_last_epoch.is_none()
            && weather.lightning_avg_distance.is_none()
            && weather.lightning_strike_count.is_none()
        {
            right_col = right_col.push(Self::metric_row(
                "Status",
                "No Activity".to_string(),
                "No strikes detected",
                accent,
            ));
        }

        // STATION HEALTH
        right_col = right_col
            .push(widget::Space::with_height(Length::Fixed(16.0)))
            .push(Self::section_header(
                "STATION",
                "network-wireless-signal-excellent-symbolic",
            ))
            .push(widget::divider::horizontal::light());

        if let Some(voltage) = weather.battery_voltage {
            let status = crate::ui::components::get_battery_status(voltage);
            right_col = right_col.push(Self::metric_row(
                "Battery",
                format!("{:.2}V", voltage),
                &status.to_string(),
                accent,
            ));
        }
        if let Some(last_updated) = self.selected_station.as_ref().and_then(|s| s.last_updated) {
            let now = chrono::Utc::now().timestamp() as u64;
            let age = now.saturating_sub(last_updated);
            let age_str = if age < 60 {
                format!("{}s ago", age)
            } else if age < 3600 {
                format!("{}m ago", age / 60)
            } else {
                format!("{}h ago", age / 3600)
            };
            right_col =
                right_col.push(Self::metric_row("Last Update", age_str, "Data age", accent));
        }

        // === 2-COLUMN LAYOUT (Header is in right column top) ===
        let dashboard = row()
            .push(left_col)
            .push(right_col)
            .spacing(space_m)
            .width(Length::Fill)
            .height(Length::Fill);

        container(dashboard)
            .padding(space_s)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
