// SPDX-License-Identifier: GPL-3.0-or-later

use crate::app::Message;
use crate::app::messages::UnitSelectorType;
use crate::config::Config;
use crate::fl;
use crate::weather::{
    DistanceUnit, PrecipitationUnit, PressureUnit, TemperatureUnit, WindSpeedUnit,
};
use cosmic::Element;
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, button, column, container, row, text};

pub(super) struct UnitSelectorView<'a> {
    config: &'a Config,
    selector_type: UnitSelectorType,
    space_m: u16,
}

struct UnitOption {
    label: &'static str,
    selected: bool,
    message: Message,
}

impl<'a> UnitSelectorView<'a> {
    pub(super) fn new(config: &'a Config, selector_type: UnitSelectorType, space_m: u16) -> Self {
        Self {
            config,
            selector_type,
            space_m,
        }
    }

    pub(super) fn render(self) -> Element<'a, Message> {
        let options = self.options().into_iter().enumerate().fold(
            column().spacing(0),
            |options, (index, option)| {
                let options = if index == 0 {
                    options
                } else {
                    options.push(widget::divider::horizontal::light())
                };
                options.push(Self::option_button(option))
            },
        );

        container(
            column()
                .push(
                    container(
                        row()
                            .push(text(self.title()).size(20))
                            .push(widget::horizontal_space())
                            .push(
                                button::text(fl!("common-close"))
                                    .on_press(Message::CloseUnitSelector),
                            )
                            .align_y(Alignment::Center)
                            .width(Length::Fill),
                    )
                    .padding(self.space_m),
                )
                .push(widget::divider::horizontal::default())
                .push(widget::scrollable(
                    container(options).padding(self.space_m).width(Length::Fill),
                ))
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .class(cosmic::theme::Container::Card)
        .width(Length::Fixed(400.0))
        .height(Length::Fill)
        .into()
    }

    fn title(&self) -> &'static str {
        match self.selector_type {
            UnitSelectorType::Temperature => "Temperature Unit",
            UnitSelectorType::WindSpeed => "Wind Speed Unit",
            UnitSelectorType::Pressure => "Pressure Unit",
            UnitSelectorType::Precipitation => "Precipitation Unit",
            UnitSelectorType::Distance => "Distance Unit",
        }
    }

    fn options(&self) -> Vec<UnitOption> {
        match self.selector_type {
            UnitSelectorType::Temperature => vec![
                UnitOption::new(
                    "Celsius (°C)",
                    self.config.temperature_unit == TemperatureUnit::Celsius,
                    Message::ChangeTemperatureUnit(TemperatureUnit::Celsius),
                ),
                UnitOption::new(
                    "Fahrenheit (°F)",
                    self.config.temperature_unit == TemperatureUnit::Fahrenheit,
                    Message::ChangeTemperatureUnit(TemperatureUnit::Fahrenheit),
                ),
            ],
            UnitSelectorType::WindSpeed => [
                ("m/s", WindSpeedUnit::MetersPerSecond),
                ("mph", WindSpeedUnit::MilesPerHour),
                ("km/h", WindSpeedUnit::KilometersPerHour),
                ("knots", WindSpeedUnit::Knots),
                ("Beaufort", WindSpeedUnit::Beaufort),
                ("ft/min", WindSpeedUnit::FeetPerMinute),
            ]
            .into_iter()
            .map(|(label, unit)| {
                UnitOption::new(
                    label,
                    self.config.wind_speed_unit == unit,
                    Message::SetWindSpeedUnit(unit),
                )
            })
            .collect(),
            UnitSelectorType::Pressure => [
                ("mb", PressureUnit::Millibar),
                ("hPa", PressureUnit::Hectopascal),
                ("inHg", PressureUnit::InchesOfMercury),
                ("mmHg", PressureUnit::MillimetersOfMercury),
            ]
            .into_iter()
            .map(|(label, unit)| {
                UnitOption::new(
                    label,
                    self.config.pressure_unit == unit,
                    Message::SetPressureUnit(unit),
                )
            })
            .collect(),
            UnitSelectorType::Precipitation => [
                ("mm", PrecipitationUnit::Millimeters),
                ("cm", PrecipitationUnit::Centimeters),
                ("inches", PrecipitationUnit::Inches),
            ]
            .into_iter()
            .map(|(label, unit)| {
                UnitOption::new(
                    label,
                    self.config.precipitation_unit == unit,
                    Message::SetPrecipitationUnit(unit),
                )
            })
            .collect(),
            UnitSelectorType::Distance => [
                ("km", DistanceUnit::Kilometers),
                ("miles", DistanceUnit::Miles),
            ]
            .into_iter()
            .map(|(label, unit)| {
                UnitOption::new(
                    label,
                    self.config.distance_unit == unit,
                    Message::SetDistanceUnit(unit),
                )
            })
            .collect(),
        }
    }

    fn option_button(option: UnitOption) -> Element<'static, Message> {
        button::custom(
            row()
                .push(text(option.label).size(14).width(Length::Fill))
                .push_maybe(
                    option
                        .selected
                        .then(|| widget::icon::from_name("object-select-symbolic").size(16)),
                )
                .align_y(Alignment::Center)
                .spacing(8)
                .width(Length::Fill),
        )
        .on_press(option.message)
        .padding(12)
        .width(Length::Fill)
        .class(if option.selected {
            cosmic::theme::Button::Suggested
        } else {
            cosmic::theme::Button::Text
        })
        .into()
    }
}

impl UnitOption {
    fn new(label: &'static str, selected: bool, message: Message) -> Self {
        Self {
            label,
            selected,
            message,
        }
    }
}
