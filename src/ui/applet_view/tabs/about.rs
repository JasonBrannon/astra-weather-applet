// SPDX-License-Identifier: GPL-3.0-or-later

use crate::app::Message;
use crate::fl;
use cosmic::Element;
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, button, column, container, horizontal_space, row, text};

pub(super) struct AboutView {
    space_s: u16,
    space_m: u16,
}

impl AboutView {
    pub(super) fn new(space_s: u16, space_m: u16) -> Self {
        Self { space_s, space_m }
    }

    pub(super) fn render(self) -> Element<'static, Message> {
        widget::scrollable(
            container(
                column()
                    .push(
                        column()
                            .push(widget::icon::from_name("com.slagmine.astra-symbolic").size(64))
                            .push(text("Astra Weather Applet").size(24))
                            .push(text(fl!("about-tagline")).size(14))
                            .spacing(self.space_s)
                            .align_x(Alignment::Center)
                            .width(Length::Fill),
                    )
                    .push(widget::divider::horizontal::default())
                    .push(
                        row()
                            .push(text(fl!("about-version")).size(14))
                            .push(horizontal_space())
                            .push(text(env!("CARGO_PKG_VERSION")).size(14))
                            .spacing(self.space_s)
                            .width(Length::Fill),
                    )
                    .push(widget::divider::horizontal::default())
                    .push(text(fl!("about-description")).size(14))
                    .push(widget::divider::horizontal::default())
                    .push(
                        row()
                            .push(text(fl!("about-repository")).size(14))
                            .push(horizontal_space())
                            .push(button::link(env!("CARGO_PKG_REPOSITORY")).on_press(
                                Message::OpenUrl(env!("CARGO_PKG_REPOSITORY").to_string()),
                            ))
                            .spacing(self.space_s)
                            .width(Length::Fill),
                    )
                    .push(widget::divider::horizontal::default())
                    .push(
                        row()
                            .push(text(fl!("about-license")).size(14))
                            .push(horizontal_space())
                            .push(text(env!("CARGO_PKG_LICENSE")).size(14))
                            .spacing(self.space_s)
                            .width(Length::Fill),
                    )
                    .spacing(self.space_m)
                    .width(Length::Fill),
            )
            .padding(self.space_m)
            .width(Length::Fill),
        )
        .into()
    }
}
