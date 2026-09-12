// SPDX-License-Identifier: GPL-3.0-or-later

use crate::app::Message;
use crate::time_utils;
use crate::weather::{AlertEvent, DistanceUnit, WindSpeedUnit};
use cosmic::Element;
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, column, container, row, text};

pub(super) struct AlertHistoryView<'a> {
    events: &'a [AlertEvent],
    distance_unit: DistanceUnit,
    wind_speed_unit: WindSpeedUnit,
    space_s: u16,
}

impl<'a> AlertHistoryView<'a> {
    pub(super) fn new(
        events: &'a [AlertEvent],
        distance_unit: DistanceUnit,
        wind_speed_unit: WindSpeedUnit,
        space_s: u16,
    ) -> Self {
        Self {
            events,
            distance_unit,
            wind_speed_unit,
            space_s,
        }
    }

    pub(super) fn render(self) -> Element<'a, Message> {
        if self.events.is_empty() {
            return column()
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
                .spacing(self.space_s)
                .width(Length::Fill)
                .height(Length::Fill)
                .into();
        }

        let history = self.events.iter().rev().take(10).fold(
            column().spacing(0).width(Length::Fill),
            |history, event| {
                history
                    .push(self.event_row(event))
                    .push(widget::divider::horizontal::light())
            },
        );

        column()
            .push(text("Recent Activity").size(14))
            .push(
                widget::scrollable(
                    container(history)
                        .padding(0)
                        .width(Length::Fill)
                        .class(cosmic::theme::Container::Card),
                )
                .height(Length::Fill),
            )
            .spacing(self.space_s)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn event_row(&self, event: &AlertEvent) -> Element<'static, Message> {
        let (icon, title, details) = self.describe(event);
        container(
            row()
                .push(widget::icon::from_name(icon).size(16))
                .push(
                    column()
                        .push(text(title).size(12))
                        .push(text(details).size(10))
                        .spacing(2)
                        .width(Length::Fill),
                )
                .spacing(12)
                .align_y(Alignment::Center)
                .width(Length::Fill),
        )
        .padding(12)
        .width(Length::Fill)
        .into()
    }

    fn describe(&self, event: &AlertEvent) -> (&'static str, String, String) {
        match event {
            AlertEvent::Lightning {
                timestamp,
                distance_km,
                strike_count,
            } => {
                let distance = distance_km.map_or_else(
                    || "Unknown distance".to_string(),
                    |distance| {
                        let converted = crate::weather::conversions::convert_distance(
                            distance,
                            self.distance_unit,
                        );
                        format!("{converted:.1} {} away", self.distance_unit)
                    },
                );
                (
                    "weather-storm-symbolic",
                    format!(
                        "[ALERT] Lightning Strike{}",
                        if *strike_count > 1 { "s" } else { "" }
                    ),
                    format!("{} • {}", distance, time_utils::format_datetime(*timestamp)),
                )
            }
            AlertEvent::Rain { timestamp } => (
                "weather-showers-symbolic",
                "💧 Rain Started".to_string(),
                time_utils::format_datetime(*timestamp),
            ),
            AlertEvent::Wind {
                timestamp,
                speed_mps,
                direction_deg,
            } => {
                let speed = crate::weather::convert_wind_speed(*speed_mps, self.wind_speed_unit);
                (
                    "weather-windy-symbolic",
                    "[ALERT] High Wind".to_string(),
                    format!(
                        "{speed:.1} {} at {direction_deg}° • {}",
                        self.wind_speed_unit,
                        time_utils::format_datetime(*timestamp)
                    ),
                )
            }
        }
    }
}
