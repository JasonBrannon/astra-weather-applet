// SPDX-License-Identifier: GPL-3.0-or-later

use crate::app::{AppState, Message};
use crate::fl;
use crate::time_utils;
use cosmic::Element;
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, column, horizontal_space, row, text};

mod card;
mod history;

use self::card::AlertCard;
use self::history::AlertHistoryView;

pub(super) struct AlertsView<'a> {
    state: &'a AppState,
    space_s: u16,
    space_m: u16,
}

impl<'a> AlertsView<'a> {
    pub(super) fn new(state: &'a AppState, space_s: u16, space_m: u16) -> Self {
        Self {
            state,
            space_s,
            space_m,
        }
    }

    pub(super) fn render(self) -> Element<'a, Message> {
        // Compact header with global toggle
        let header = row()
            .push(text(fl!("alerts-title")).size(16))
            .push(horizontal_space())
            .push(widget::icon::from_name("notification-symbolic").size(16))
            .push(text("Desktop Notifications").size(12))
            .push(
                widget::toggler(self.state.config.notifications_enabled)
                    .on_toggle(Message::SetNotifications),
            )
            .spacing(self.space_s)
            .align_y(Alignment::Center)
            .width(Length::Fill);

        let cards_row = row()
            .push(self.lightning_card().render())
            .push(self.rain_card().render())
            .push(self.wind_card().render())
            .spacing(self.space_m)
            .width(Length::Fill);

        let history_section = AlertHistoryView::new(
            &self.state.alert_history,
            self.state.config.distance_unit,
            self.state.config.wind_speed_unit,
            self.space_s,
        )
        .render();

        column()
            .push(header)
            .push(cards_row)
            .push(history_section)
            .spacing(self.space_s)
            .width(Length::Fill)
            .into()
    }

    fn lightning_card(&self) -> AlertCard {
        let (status, details) = if let Some(event) = self.state.get_last_lightning_event() {
            let distance = event.distance_km.map_or_else(
                || "Unknown".to_string(),
                |distance| {
                    let converted = crate::weather::conversions::convert_distance(
                        distance,
                        self.state.config.distance_unit,
                    );
                    format!("{converted:.1} {}", self.state.config.distance_unit)
                },
            );
            (
                "Lightning Detected".to_string(),
                format!(
                    "{} strikes • {} • {}",
                    event.strike_count,
                    distance,
                    time_utils::format_time(event.timestamp)
                ),
            )
        } else {
            (
                fl!("alerts-lightning-none"),
                "No recent activity".to_string(),
            )
        };

        self.card(
            fl!("alerts-lightning-title"),
            status,
            details,
            self.state.config.lightning_notifications_enabled,
            Message::SetLightningNotifications,
        )
    }

    fn rain_card(&self) -> AlertCard {
        let (status, details) = self.state.last_rain_start.as_ref().map_or_else(
            || (fl!("alerts-rain-none"), "No recent activity".to_string()),
            |event| {
                (
                    "Rain Detected".to_string(),
                    format!("Started: {}", time_utils::format_datetime(event.timestamp)),
                )
            },
        );

        self.card(
            fl!("alerts-rain-title"),
            status,
            details,
            self.state.config.rain_notifications_enabled,
            Message::SetRainNotifications,
        )
    }

    fn wind_card(&self) -> AlertCard {
        let (status, details) = self.state.last_rapid_wind.as_ref().map_or_else(
            || (fl!("alerts-wind-none"), "No recent activity".to_string()),
            |event| {
                let speed = crate::weather::convert_wind_speed(
                    event.wind_speed_mps,
                    self.state.config.wind_speed_unit,
                );
                let status = if event.wind_speed_mps * 2.237 > 33.0 {
                    fl!("alerts-wind-high")
                } else {
                    fl!("alerts-wind-recent")
                };
                (
                    status,
                    format!(
                        "{speed:.1} {} • {}° • {}",
                        self.state.config.wind_speed_unit,
                        event.wind_direction_deg,
                        time_utils::format_time(event.timestamp)
                    ),
                )
            },
        );

        self.card(
            fl!("alerts-wind-title"),
            status,
            details,
            self.state.config.wind_notifications_enabled,
            Message::SetWindNotifications,
        )
    }

    fn card(
        &self,
        title: String,
        status: String,
        details: String,
        event_enabled: bool,
        on_toggle: fn(bool) -> Message,
    ) -> AlertCard {
        let notifications_enabled = self.state.config.notifications_enabled;
        AlertCard::new(
            title,
            status,
            details,
            event_enabled && notifications_enabled,
            notifications_enabled.then_some(on_toggle),
        )
    }
}
