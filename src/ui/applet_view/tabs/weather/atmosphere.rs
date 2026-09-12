// SPDX-License-Identifier: GPL-3.0-or-later

use super::metric::{Metric, section_header};
use crate::app::Message;
use crate::config::Config;
use crate::weather::WeatherData;
use crate::weather::conversions::{format_pressure, format_temperature, format_wind_speed};
use cosmic::Element;
use cosmic::iced::Length;
use cosmic::widget::{self, column};

pub(super) struct AtmosphereView<'a> {
    weather: &'a WeatherData,
    config: &'a Config,
}

impl<'a> AtmosphereView<'a> {
    pub(super) fn new(weather: &'a WeatherData, config: &'a Config) -> Self {
        Self { weather, config }
    }

    pub(super) fn render(self) -> Element<'static, Message> {
        let accent = cosmic::theme::active().cosmic().accent_color();
        let mut content = column()
            .spacing(8)
            .width(Length::FillPortion(1))
            .push(section_header("ATMOSPHERIC", "weather-overcast-symbolic"))
            .push(widget::divider::horizontal::light());

        if let Some(feels_like) = self.weather.feels_like {
            let feels_like = format_temperature(feels_like, self.config.temperature_unit);
            tracing::debug!(
                "[DEBUG] Display: Actual Temperature: {}, Feels Like: {} (Unit: {:?})",
                format_temperature(self.weather.temperature, self.config.temperature_unit),
                feels_like,
                self.config.temperature_unit
            );
            content =
                content.push(Metric::new("Feels Like", feels_like, "Apparent temp").render(accent));
        }

        content = content.push(
            Metric::new(
                "Humidity",
                format!("{}%", self.weather.humidity),
                "Moisture in air",
            )
            .render(accent),
        );

        if let Some(dew_point) = self.weather.dew_point {
            content = content.push(
                Metric::new(
                    "Dew Point",
                    format_temperature(dew_point, self.config.temperature_unit),
                    "Condensation point",
                )
                .render(accent),
            );
        }

        content = content
            .push(
                Metric::new(
                    "Pressure",
                    format_pressure(self.weather.pressure, self.config.pressure_unit),
                    "Barometric pressure",
                )
                .render(accent),
            )
            .push(widget::Space::with_height(Length::Fixed(16.0)))
            .push(section_header("WIND", "weather-windy-symbolic"))
            .push(widget::divider::horizontal::light())
            .push(Metric::new("Current", self.current_wind(), "Speed & direction").render(accent));

        if let Some(lull) = self.weather.wind_lull {
            content = content.push(
                Metric::new(
                    "Lull",
                    format_wind_speed(lull, self.config.wind_speed_unit),
                    "Minimum speed",
                )
                .render(accent),
            );
        }

        if let Some(gust) = self.weather.wind_gust {
            content = content.push(
                Metric::new(
                    "Gust",
                    format_wind_speed(gust, self.config.wind_speed_unit),
                    "Maximum speed",
                )
                .render(accent),
            );
        }

        content.into()
    }

    fn current_wind(&self) -> String {
        if self.weather.wind_speed < 0.5 {
            return "Calm".to_string();
        }

        format!(
            "{} {} {}",
            crate::ui::components::get_cardinal_direction(self.weather.wind_direction as f64),
            wind_arrow(self.weather.wind_direction),
            format_wind_speed(self.weather.wind_speed, self.config.wind_speed_unit)
        )
    }
}

fn wind_arrow(degrees: u16) -> &'static str {
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

#[cfg(test)]
mod tests {
    use super::wind_arrow;

    #[test]
    fn wind_arrow_changes_at_sector_boundaries() {
        assert_eq!(wind_arrow(22), "↑");
        assert_eq!(wind_arrow(23), "↗");
        assert_eq!(wind_arrow(337), "↖");
        assert_eq!(wind_arrow(338), "↑");
    }
}
