// SPDX-License-Identifier: GPL-3.0-or-later

use crate::app::Message;
use cosmic::Element;
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, column, horizontal_space, row, text};

pub(super) struct AlertCard {
    title: String,
    status: String,
    details: String,
    enabled: bool,
    on_toggle: Option<fn(bool) -> Message>,
}

impl AlertCard {
    pub(super) fn new(
        title: impl Into<String>,
        status: impl Into<String>,
        details: impl Into<String>,
        enabled: bool,
        on_toggle: Option<fn(bool) -> Message>,
    ) -> Self {
        Self {
            title: title.into(),
            status: status.into(),
            details: details.into(),
            enabled,
            on_toggle,
        }
    }

    pub(super) fn render(self) -> Element<'static, Message> {
        let accent = cosmic::theme::active().cosmic().accent_color();
        widget::container(
            column()
                .push(
                    row()
                        .push(text(self.title).size(14))
                        .push(horizontal_space())
                        .push(
                            widget::toggler(self.enabled)
                                .on_toggle_maybe(self.on_toggle)
                                .width(Length::Shrink),
                        )
                        .spacing(8)
                        .align_y(Alignment::Center)
                        .width(Length::Fill),
                )
                .push(
                    text(self.status)
                        .size(12)
                        .class(cosmic::theme::Text::Color(accent.into())),
                )
                .push(text(self.details).size(11))
                .spacing(4)
                .width(Length::Fill),
        )
        .padding(12)
        .width(Length::FillPortion(1))
        .class(cosmic::theme::Container::Card)
        .into()
    }
}
