// SPDX-License-Identifier: GPL-3.0-or-later

mod atmosphere;
mod environment;
mod metric;

use self::atmosphere::AtmosphereView;
use self::environment::EnvironmentView;
use crate::app::{AppState, Message};
use crate::fl;
use cosmic::Element;
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{column, container, row, text};

pub(super) struct WeatherView<'a> {
    state: &'a AppState,
    space_s: u16,
    space_m: u16,
}

impl<'a> WeatherView<'a> {
    pub(super) fn new(state: &'a AppState, space_s: u16, space_m: u16) -> Self {
        Self {
            state,
            space_s,
            space_m,
        }
    }

    pub(super) fn render(self) -> Element<'a, Message> {
        let Some(weather) = &self.state.current_weather else {
            return column()
                .push(text(fl!("weather-loading")))
                .align_x(Alignment::Center)
                .into();
        };

        let dashboard = row()
            .push(AtmosphereView::new(weather, &self.state.config).render())
            .push(
                EnvironmentView::new(
                    weather,
                    &self.state.config,
                    self.state
                        .selected_station
                        .as_ref()
                        .and_then(|station| station.last_updated),
                )
                .render(),
            )
            .spacing(self.space_m)
            .width(Length::Fill)
            .height(Length::Fill);

        column()
            .push(
                container(dashboard)
                    .padding(self.space_s)
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .spacing(self.space_m)
            .width(Length::Fill)
            .align_x(Alignment::Center)
            .into()
    }
}
