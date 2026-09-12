// SPDX-License-Identifier: GPL-3.0-or-later

use crate::app::Message;
use cosmic::Element;
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, column, row, text};

type Accent = cosmic::cosmic_theme::palette::Alpha<cosmic::cosmic_theme::palette::rgb::Rgb, f32>;

pub(super) struct Metric {
    label: &'static str,
    value: String,
    description: String,
}

impl Metric {
    pub(super) fn new(
        label: &'static str,
        value: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            label,
            value: value.into(),
            description: description.into(),
        }
    }

    pub(super) fn render(self, accent: Accent) -> Element<'static, Message> {
        row()
            .push(
                column()
                    .push(text(self.label).size(13))
                    .push(text(self.description).size(11))
                    .spacing(2)
                    .width(Length::Fixed(140.0)),
            )
            .push(
                text(self.value)
                    .size(18)
                    .class(cosmic::theme::Text::Color(accent.into()))
                    .wrapping(cosmic::iced_core::text::Wrapping::None),
            )
            .spacing(12)
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .into()
    }
}

pub(super) fn section_header(title: &'static str, icon: &'static str) -> Element<'static, Message> {
    row()
        .push(widget::icon::from_name(icon).size(14))
        .push(text(title).size(13))
        .spacing(6)
        .align_y(Alignment::Center)
        .into()
}
