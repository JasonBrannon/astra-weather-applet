// SPDX-License-Identifier: GPL-3.0-or-later

use super::metric::{Metric, section_header};
use crate::app::Message;
use crate::config::Config;
use crate::weather::WeatherData;
use crate::weather::conversions::{convert_distance, format_precipitation};
use cosmic::Element;
use cosmic::iced::Length;
use cosmic::widget::{self, column};

pub(super) struct EnvironmentView<'a> {
    weather: &'a WeatherData,
    config: &'a Config,
    last_updated: Option<u64>,
}

impl<'a> EnvironmentView<'a> {
    pub(super) fn new(
        weather: &'a WeatherData,
        config: &'a Config,
        last_updated: Option<u64>,
    ) -> Self {
        Self {
            weather,
            config,
            last_updated,
        }
    }

    pub(super) fn render(self) -> Element<'static, Message> {
        let accent = cosmic::theme::active().cosmic().accent_color();
        let mut content = column()
            .spacing(8)
            .width(Length::FillPortion(1))
            .push(section_header("PRECIPITATION", "weather-showers-symbolic"))
            .push(widget::divider::horizontal::light());

        if let Some(last_hour) = self.weather.precip_accum_last_1hr {
            content = content.push(
                Metric::new(
                    "Last Hour",
                    format_precipitation(last_hour, self.config.precipitation_unit),
                    "Past 60 minutes",
                )
                .render(accent),
            );
        }
        if let Some(today) = self.weather.local_day_rain_accumulation {
            content = content.push(
                Metric::new(
                    "Today",
                    format_precipitation(today, self.config.precipitation_unit),
                    "Daily total",
                )
                .render(accent),
            );
        }
        if let Some(precip_type) = self.weather.precip_type {
            let precip_type = crate::ui::components::get_precip_type_text(precip_type);
            if precip_type != "None" {
                content = content
                    .push(Metric::new("Type", precip_type, "Precip category").render(accent));
            }
        }
        if self.weather.precip_accum_last_1hr.is_none()
            && self.weather.local_day_rain_accumulation.is_none()
            && self.weather.precip_type.is_none()
        {
            content =
                content.push(Metric::new("Status", "None", "No precipitation").render(accent));
        }

        content = content
            .push(widget::Space::with_height(Length::Fixed(16.0)))
            .push(section_header("SOLAR & LIGHT", "weather-clear-symbolic"))
            .push(widget::divider::horizontal::light());

        if let Some(uv) = self.weather.uv {
            content = content.push(
                Metric::new("UV Index", format!("{uv:.1}"), "UV radiation level").render(accent),
            );
        }
        if let Some(solar) = self.weather.solar_radiation {
            content = content.push(
                Metric::new(
                    "Solar Radiation",
                    format!("{solar} W/m²"),
                    "Incoming energy",
                )
                .render(accent),
            );
        }
        if let Some(brightness) = self.weather.brightness {
            content = content.push(
                Metric::new("Illuminance", format!("{brightness} lux"), "Light level")
                    .render(accent),
            );
        }
        if self.weather.uv.is_none()
            && self.weather.solar_radiation.is_none()
            && self.weather.brightness.is_none()
        {
            content =
                content.push(Metric::new("Status", "No Data", "Sensors offline").render(accent));
        }

        content = content
            .push(widget::Space::with_height(Length::Fixed(16.0)))
            .push(section_header("LIGHTNING", "weather-storm-symbolic"))
            .push(widget::divider::horizontal::light());

        if let Some(last_epoch) = self.weather.lightning_strike_last_epoch {
            content = content.push(
                Metric::new(
                    "Last Strike",
                    relative_age_signed(chrono::Utc::now().timestamp() - last_epoch),
                    "Most recent",
                )
                .render(accent),
            );
        }
        if let Some(distance) = self.weather.lightning_avg_distance {
            let distance = convert_distance(distance, self.config.distance_unit);
            content = content.push(
                Metric::new(
                    "Distance",
                    format!("{distance:.1} {}", self.config.distance_unit),
                    "Average distance",
                )
                .render(accent),
            );
        }
        if let Some(count) = self.weather.lightning_strike_count {
            content = content.push(
                Metric::new("Strikes (3h)", count.to_string(), "Recent count").render(accent),
            );
        }
        if self.weather.lightning_strike_last_epoch.is_none()
            && self.weather.lightning_avg_distance.is_none()
            && self.weather.lightning_strike_count.is_none()
        {
            content = content
                .push(Metric::new("Status", "No Activity", "No strikes detected").render(accent));
        }

        content = content
            .push(widget::Space::with_height(Length::Fixed(16.0)))
            .push(section_header(
                "STATION",
                "network-wireless-signal-excellent-symbolic",
            ))
            .push(widget::divider::horizontal::light());

        if let Some(voltage) = self.weather.battery_voltage {
            content = content.push(
                Metric::new(
                    "Battery",
                    format!("{voltage:.2}V"),
                    crate::ui::components::get_battery_status(voltage),
                )
                .render(accent),
            );
        }
        if let Some(last_updated) = self.last_updated {
            let age = (chrono::Utc::now().timestamp() as u64).saturating_sub(last_updated);
            content = content
                .push(Metric::new("Last Update", relative_age(age), "Data age").render(accent));
        }

        content.into()
    }
}

fn relative_age_signed(age: i64) -> String {
    relative_age(age.max(0) as u64)
}

fn relative_age(age: u64) -> String {
    if age < 60 {
        format!("{age}s ago")
    } else if age < 3_600 {
        format!("{}m ago", age / 60)
    } else if age < 86_400 {
        format!("{}h ago", age / 3_600)
    } else {
        format!("{}d ago", age / 86_400)
    }
}

#[cfg(test)]
mod tests {
    use super::relative_age;

    #[test]
    fn relative_age_uses_expected_boundaries() {
        assert_eq!(relative_age(59), "59s ago");
        assert_eq!(relative_age(60), "1m ago");
        assert_eq!(relative_age(3_600), "1h ago");
        assert_eq!(relative_age(86_400), "1d ago");
    }
}
