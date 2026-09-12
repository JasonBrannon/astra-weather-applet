// SPDX-License-Identifier: GPL-3.0-or-later

mod about;
mod alerts;
mod settings;
mod weather;

use self::about::AboutView;
use self::alerts::AlertsView;
use self::settings::SettingsView;
use self::weather::WeatherView;
use crate::app::messages::AppTab;
use crate::app::{AppState, Message};
use cosmic::Element;
use cosmic::iced::Length;
use cosmic::widget::container;

pub(super) struct TabsView<'a> {
    state: &'a AppState,
    space_s: u16,
    space_m: u16,
}

impl<'a> TabsView<'a> {
    pub(super) fn new(state: &'a AppState, space_s: u16, space_m: u16) -> Self {
        Self {
            state,
            space_s,
            space_m,
        }
    }

    pub(super) fn render(self) -> Element<'a, Message> {
        let content = match self.state.current_tab {
            AppTab::Weather => WeatherView::new(self.state, self.space_s, self.space_m).render(),
            AppTab::Forecast => crate::ui::forecast_view(self.state),
            AppTab::Stations => crate::ui::stations_view(self.state),
            AppTab::Alerts => AlertsView::new(self.state, self.space_s, self.space_m).render(),
            AppTab::Settings => SettingsView::new(self.state, self.space_s, self.space_m).render(),
            AppTab::About => AboutView::new(self.space_s, self.space_m).render(),
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
