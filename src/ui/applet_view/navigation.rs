// SPDX-License-Identifier: GPL-3.0-or-later

use crate::app::messages::AppTab;
use crate::app::{AppState, Message};
use crate::fl;
use crate::weather::TemperatureUnit;
use cosmic::Element;
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{button, column, container, horizontal_space, row, text};

pub(super) struct NavigationView<'a> {
    state: &'a AppState,
    space_s: u16,
}

impl<'a> NavigationView<'a> {
    pub(super) fn new(state: &'a AppState, space_s: u16) -> Self {
        Self { state, space_s }
    }

    pub(super) fn render(self) -> Element<'a, Message> {
        if self.state.needs_station_selection {
            return container(
                row()
                    .push(button::suggested(fl!("tab-stations")))
                    .spacing(self.space_s),
            )
            .width(Length::Fill)
            .into();
        }

        let has_station = self.state.selected_station.is_some();
        let mut tabs = self
            .tab_specs()
            .into_iter()
            .fold(row().spacing(self.space_s), |tabs, tab| {
                tabs.push(tab.render(self.state.current_tab, has_station))
            });
        tabs = tabs.push(horizontal_space());

        if let Some(summary) = self.weather_summary() {
            tabs = tabs
                .push(summary.icon)
                .push(
                    column()
                        .push(text(summary.temperature).size(24))
                        .push(text(summary.description).size(12))
                        .spacing(2)
                        .align_x(cosmic::iced::alignment::Horizontal::Center),
                )
                .spacing(self.space_s)
                .align_y(Alignment::Center);
        }

        tabs.into()
    }

    fn tab_specs(&self) -> [TabSpec; 6] {
        [
            TabSpec::new(AppTab::Forecast, fl!("tab-forecast"), true),
            TabSpec::new(AppTab::Weather, fl!("tab-weather"), true),
            TabSpec::new(AppTab::Stations, fl!("tab-stations"), false),
            TabSpec::new(AppTab::Alerts, fl!("tab-alerts"), true),
            TabSpec::new(AppTab::Settings, fl!("tab-settings"), false),
            TabSpec::new(AppTab::About, fl!("tab-about"), false),
        ]
    }

    fn weather_summary(&self) -> Option<WeatherSummary> {
        let weather = self.state.current_weather.as_ref()?;
        let temperature = match self.state.config.temperature_unit {
            TemperatureUnit::Celsius => format_temperature(weather.temperature, "°C"),
            TemperatureUnit::Fahrenheit => {
                format_temperature(weather.temperature * 9.0 / 5.0 + 32.0, "°F")
            }
        };

        Some(WeatherSummary {
            icon: crate::ui::components::weather_icon_widget(
                &weather.icon,
                &weather.description,
                48,
            ),
            temperature,
            description: weather.description.clone(),
        })
    }
}

struct TabSpec {
    tab: AppTab,
    label: String,
    requires_station: bool,
}

impl TabSpec {
    fn new(tab: AppTab, label: String, requires_station: bool) -> Self {
        Self {
            tab,
            label,
            requires_station,
        }
    }

    fn render(self, current: AppTab, has_station: bool) -> Element<'static, Message> {
        if self.requires_station && !has_station {
            return text(self.label).size(14).into();
        }

        if current == self.tab {
            button::suggested(self.label)
                .on_press(Message::SelectTab(self.tab))
                .into()
        } else {
            button::text(self.label)
                .on_press(Message::SelectTab(self.tab))
                .into()
        }
    }
}

struct WeatherSummary {
    icon: Element<'static, Message>,
    temperature: String,
    description: String,
}

fn format_temperature(temp: f64, unit: &str) -> String {
    if temp.fract() == 0.0 {
        format!("{temp:.0}{unit}")
    } else {
        format!("{temp:.1}{unit}")
    }
}

#[cfg(test)]
mod tests {
    use super::format_temperature;

    #[test]
    fn formats_whole_temperature_without_decimal() {
        assert_eq!(format_temperature(45.0, "°F"), "45°F");
    }

    #[test]
    fn formats_fractional_temperature_with_one_decimal() {
        assert_eq!(format_temperature(12.34, "°C"), "12.3°C");
    }
}
