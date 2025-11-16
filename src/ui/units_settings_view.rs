// SPDX-License-Identifier: GPL-3.0-or-later

use crate::app::Message;
use crate::config::Config;
use crate::weather::{
    DistanceUnit, PrecipitationUnit, PressureUnit, TemperatureUnit, WindSpeedUnit,
};
use cosmic::iced::{Alignment, Length};
use cosmic::prelude::*;
use cosmic::widget::{container, row, text};

/// Compact unit row for Settings view (COSMIC style)
/// Label on left, current value + arrow on right
pub fn compact_unit_row<'a>(
    setting_name: &'static str,
    config: &'a Config,
) -> Element<'a, Message> {
    use crate::app::messages::UnitSelectorType;

    let (current_value, selector_type) = match setting_name {
        "Temperature" => (
            match config.temperature_unit {
                TemperatureUnit::Celsius => "Celsius (°C)",
                TemperatureUnit::Fahrenheit => "Fahrenheit (°F)",
            },
            UnitSelectorType::Temperature,
        ),
        "Wind Speed" => (
            match config.wind_speed_unit {
                WindSpeedUnit::MetersPerSecond => "m/s",
                WindSpeedUnit::MilesPerHour => "mph",
                WindSpeedUnit::KilometersPerHour => "km/h",
                WindSpeedUnit::Knots => "knots",
                WindSpeedUnit::Beaufort => "Beaufort",
                WindSpeedUnit::FeetPerMinute => "ft/min",
            },
            UnitSelectorType::WindSpeed,
        ),
        "Pressure" => (
            match config.pressure_unit {
                PressureUnit::Millibar => "mb",
                PressureUnit::Hectopascal => "hPa",
                PressureUnit::InchesOfMercury => "inHg",
                PressureUnit::MillimetersOfMercury => "mmHg",
            },
            UnitSelectorType::Pressure,
        ),
        "Precipitation" => (
            match config.precipitation_unit {
                PrecipitationUnit::Millimeters => "mm",
                PrecipitationUnit::Centimeters => "cm",
                PrecipitationUnit::Inches => "inches",
            },
            UnitSelectorType::Precipitation,
        ),
        "Distance" => (
            match config.distance_unit {
                DistanceUnit::Kilometers => "km",
                DistanceUnit::Miles => "miles",
            },
            UnitSelectorType::Distance,
        ),
        _ => ("Unknown", UnitSelectorType::Temperature),
    };

    cosmic::widget::mouse_area(
        container(
            row()
                .push(text(setting_name).size(14).width(Length::Fill))
                .push(text(current_value).size(14))
                .push(cosmic::widget::icon::from_name("go-next-symbolic").size(16))
                .align_y(Alignment::Center)
                .spacing(8)
                .width(Length::Fill),
        )
        .padding(12)
        .width(Length::Fill),
    )
    .on_press(Message::ShowUnitSelector(selector_type))
    .into()
}
