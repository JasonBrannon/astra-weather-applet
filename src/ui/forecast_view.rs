// SPDX-License-Identifier: GPL-3.0-or-later

use crate::app::{AppState, Message};
use crate::fl;
use crate::time_utils;
use crate::weather::{DailyForecast, HourlyForecast, TemperatureUnit};
use chrono::{Local, TimeZone};
use cosmic::iced::Length;
use cosmic::widget::{column, container, row, scrollable, text};
use cosmic::{Element, cosmic_theme, iced, theme, widget};

pub fn forecast_view(state: &AppState) -> Element<'_, Message> {
    // Get COSMIC theme spacing for density-aware sizing
    let cosmic_theme::Spacing {
        space_s,
        space_m,
        space_l,
        ..
    } = theme::active().cosmic().spacing;

    let mut content = column().spacing(space_m).padding(space_m);

    // Get COSMIC theme accent color for date label
    let accent = cosmic::theme::active().cosmic().accent_color();

    // Radio buttons for hourly/daily view selection
    let radio_row = row()
        .push(widget::radio(
            text(fl!("forecast-toggle-hourly")),
            true,
            Some(state.show_hourly_forecast),
            |_| Message::ToggleForecastView,
        ))
        .push(widget::radio(
            text(fl!("forecast-toggle-daily")),
            false,
            Some(state.show_hourly_forecast),
            |_| Message::ToggleForecastView,
        ))
        .spacing(space_m)
        .align_y(cosmic::iced::Alignment::Center);

    content = content.push(
        row()
            .push(radio_row)
            .push(widget::Space::with_width(Length::Fill))
            .align_y(cosmic::iced::Alignment::Center)
            .spacing(space_s),
    );

    // Grid-based forecast view with loading animation
    if state.forecast_loading {
        // Loading animation with fixed icon size
        content = content.push(
            container(
                column()
                    .push(widget::icon::from_name("folder-download-symbolic").size(64))
                    .push(text::body(fl!("forecast-loading")))
                    .spacing(space_m)
                    .align_x(cosmic::iced::Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
        );
    } else if let Some(forecast) = &state.extended_forecast {
        let forecast_grid = if state.show_hourly_forecast {
            hourly_forecast_grid(
                &forecast.hourly,
                state.config.temperature_unit,
                space_s,
                space_m,
                space_l,
            )
        } else {
            daily_forecast_grid(
                &forecast.daily,
                state.config.temperature_unit,
                space_s,
                space_m,
                space_l,
            )
        };

        let mut scroll_view = scrollable(forecast_grid)
            .direction(iced::widget::scrollable::Direction::Horizontal(
                iced::widget::scrollable::Scrollbar::default(),
            ))
            .height(Length::Fill)
            .width(Length::Fill);

        // Add scroll event handler for hourly forecast to update day label
        if state.show_hourly_forecast {
            scroll_view = scroll_view.on_scroll(|viewport| {
                Message::HourlyForecastScrolled(viewport.absolute_offset().x)
            });
        }

        content = content.push(scroll_view);

        // Add day label below the scroll area for hourly forecast
        if state.show_hourly_forecast {
            // Fixed card width + spacing (matching hourly_forecast_grid)
            let card_width_with_spacing = 138.0; // 130px card + 8px spacing
            if let Some(day_label) = calculate_visible_day(
                &forecast.hourly,
                state.hourly_forecast_scroll_offset,
                card_width_with_spacing,
            ) {
                content = content.push(
                    container(
                        text::body(day_label)
                            .width(Length::Fill)
                            .align_x(cosmic::iced::alignment::Horizontal::Center)
                            .class(cosmic::theme::Text::Color(accent.into())), // Apply COSMIC accent color
                    )
                    .padding(space_s)
                    .width(Length::Fill)
                    .class(cosmic::theme::Container::Card),
                );
            }
        }
    } else {
        // Empty forecast state with fixed icon size
        content = content.push(
            container(
                column()
                    .push(widget::icon::from_name("folder-download-symbolic").size(64))
                    .push(text::body(fl!("forecast-loading-message")))
                    .spacing(space_m)
                    .align_x(cosmic::iced::Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
        );
    }

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn hourly_forecast_grid(
    hourly: &[HourlyForecast],
    unit: TemperatureUnit,
    _space_s: u16,
    space_m: u16,
    _space_l: u16,
) -> Element<'_, Message> {
    // Single horizontal row with all hourly forecast cards (168 hours = 7 days)
    // Card dimensions mostly fixed, but height adjusts slightly for text scaling
    let card_spacing = 8; // Fixed small spacing
    let card_padding = 10; // Fixed padding
    let card_width = 130; // Fixed width
    // Height adjusts with text size: base + space_m gives room for scaled text
    // 210 base + 16 default = 226, + 20 spacious = 230, + 12 compact = 222
    let card_height = 210 + space_m;
    let weather_icon_size = 56; // Fixed icon size
    let small_icon_size = 11; // Fixed small icon size

    let mut row_items = row()
        .spacing(card_spacing)
        .align_y(cosmic::iced::Alignment::Center);

    for hour in hourly.iter().take(168) {
        // Format temperature as whole number with degree symbol only
        let temp_value = crate::weather::convert_temperature(hour.temperature, unit);
        let temp_str = format!("{:.0}°", temp_value);

        let time_str = format_hour_timestamp(hour.timestamp);
        let icon_widget = crate::ui::components::weather_icon_widget(
            &hour.icon,
            &hour.description,
            weather_icon_size,
        );

        let card = container(
            column()
                .push(row().push(text::body(time_str)).width(Length::Fill))
                .push(icon_widget)
                .push(row().push(text::body(temp_str)).width(Length::Fill))
                .push(
                    row()
                        .push(text::caption(&hour.description))
                        .width(Length::Fill),
                )
                .push(
                    row()
                        .push(crate::ui::components::precipitation_icon_widget(
                            small_icon_size,
                        ))
                        .push(text::caption(format!(
                            "{}% Precip",
                            hour.precipitation_probability
                        )))
                        .spacing(4) // Fixed tight spacing for icon+text
                        .align_y(cosmic::iced::Alignment::Center)
                        .width(Length::Fill),
                )
                .push(
                    row()
                        .push(crate::ui::components::humidity_icon_widget(small_icon_size))
                        .push(text::caption(format!("{}% Humidity", hour.humidity)))
                        .spacing(4) // Fixed tight spacing for icon+text
                        .align_y(cosmic::iced::Alignment::Center)
                        .width(Length::Fill),
                )
                .spacing(5) // Fixed internal spacing between card elements
                .align_x(cosmic::iced::Alignment::Center),
        )
        .padding(card_padding)
        .width(card_width)
        .height(card_height)
        .class(cosmic::theme::Container::Card);

        row_items = row_items.push(card);
    }

    row_items.into()
}

fn daily_forecast_grid(
    daily: &[DailyForecast],
    unit: TemperatureUnit,
    _space_s: u16,
    space_m: u16,
    _space_l: u16,
) -> Element<'_, Message> {
    // Single horizontal row with all daily forecast cards (10 days)
    // Card dimensions mostly fixed, but height adjusts slightly for text scaling (matching hourly)
    let card_spacing = 8; // Fixed small spacing
    let card_padding = 10; // Fixed padding
    let card_width = 130; // Fixed width - matches hourly
    // Height adjusts with text size to prevent truncation in Spacious mode
    let card_height = 210 + space_m;
    let weather_icon_size = 56; // Fixed icon size - matches hourly
    let small_icon_size = 11; // Fixed small icon size

    let mut row_items = row()
        .spacing(card_spacing)
        .align_y(cosmic::iced::Alignment::Center);

    // Get theme accent color for arrows
    let accent = cosmic::theme::active().cosmic().accent_color();

    for day in daily.iter().take(10) {
        // Format temperatures as whole numbers with degree symbol and arrows
        let high_value = crate::weather::convert_temperature(day.temperature_high, unit);
        let low_value = crate::weather::convert_temperature(day.temperature_low, unit);

        let icon_widget = crate::ui::components::weather_icon_widget(
            &day.icon,
            &day.description,
            weather_icon_size,
        );

        // Create temperature row with colored arrows (Low then High)
        let temp_row = row()
            .push(text::body("↓").class(cosmic::theme::Text::Color(accent.into())))
            .push(text::body(format!("{:.0}°", low_value)))
            .push(text::body(" "))
            .push(text::body("↑").class(cosmic::theme::Text::Color(accent.into())))
            .push(text::body(format!("{:.0}°", high_value)))
            .spacing(2)
            .width(Length::Fill);

        let card = container(
            column()
                .push(row().push(text::body(&day.date)).width(Length::Fill))
                .push(icon_widget)
                .push(temp_row)
                .push(
                    row()
                        .push(text::caption(&day.description))
                        .width(Length::Fill),
                )
                .push(
                    row()
                        .push(crate::ui::components::precipitation_icon_widget(
                            small_icon_size,
                        ))
                        .push(text::caption(format!(
                            "{}% Precip",
                            day.precipitation_probability
                        )))
                        .spacing(4) // Fixed tight spacing for icon+text
                        .align_y(cosmic::iced::Alignment::Center)
                        .width(Length::Fill),
                )
                .push(
                    row()
                        .push(crate::ui::components::humidity_icon_widget(small_icon_size))
                        .push(text::caption(format!("{}% Humidity", day.humidity)))
                        .spacing(4) // Fixed tight spacing for icon+text
                        .align_y(cosmic::iced::Alignment::Center)
                        .width(Length::Fill),
                )
                .spacing(5) // Fixed internal spacing between card elements
                .align_x(cosmic::iced::Alignment::Center),
        )
        .padding(card_padding)
        .width(card_width)
        .height(card_height)
        .class(cosmic::theme::Container::Card);

        row_items = row_items.push(card);
    }

    row_items.into()
}

// NOTE: Removed duplicate format_temperature - use crate::weather::format_temperature instead
// Forecast data is stored in Celsius (SI units), UI converts to user preferences

fn format_hour_timestamp(timestamp: u64) -> String {
    time_utils::format_hour_minute(timestamp)
}

// Calculate which day is currently visible based on scroll offset
fn calculate_visible_day(
    hourly: &[HourlyForecast],
    scroll_offset: f32,
    card_width_with_spacing: f32,
) -> Option<String> {
    if hourly.is_empty() {
        return None;
    }

    // Calculate which card index is at the scroll position (approximate center of view)
    let visible_card_index = (scroll_offset / card_width_with_spacing) as usize;

    // Clamp to valid range
    let index = visible_card_index.min(hourly.len().saturating_sub(1));

    // Get the timestamp of the visible card
    let timestamp = hourly[index].timestamp;

    // Convert timestamp to local datetime and format as day name
    Local
        .timestamp_opt(timestamp as i64, 0)
        .single()
        .map(|datetime| datetime.format("%A, %B %-d, %Y").to_string())
}
