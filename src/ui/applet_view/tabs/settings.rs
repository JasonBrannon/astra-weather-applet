// SPDX-License-Identifier: GPL-3.0-or-later

use crate::app::{AppState, Message};
use crate::fl;
use cosmic::Element;
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, button, column, container, horizontal_space, row, text, text_input};

mod unit_selector;

use self::unit_selector::UnitSelectorView;

pub(super) struct SettingsView<'a> {
    state: &'a AppState,
    space_s: u16,
    space_m: u16,
}

impl<'a> SettingsView<'a> {
    pub(super) fn new(state: &'a AppState, space_s: u16, space_m: u16) -> Self {
        Self {
            state,
            space_s,
            space_m,
        }
    }

    pub(super) fn render(self) -> Element<'a, Message> {
        let settings_content = widget::scrollable(
            container(
                column()
                    .push(self.api_key_section())
                    .push(widget::divider::horizontal::default())
                    .push(text(fl!("settings-units-title")).size(16))
                    .push(self.units_section())
                    .spacing(self.space_s)
                    .width(Length::Fill),
            )
            .padding(self.space_m)
            .width(Length::Fill),
        );

        if let Some(selector_type) = self.state.active_unit_selector {
            cosmic::iced::widget::stack![
                settings_content,
                row()
                    .push(widget::horizontal_space())
                    .push(
                        UnitSelectorView::new(&self.state.config, selector_type, self.space_m)
                            .render(),
                    )
                    .width(Length::Fill),
            ]
            .into()
        } else {
            settings_content.into()
        }
    }

    fn api_key_section(&self) -> cosmic::widget::Column<'a, Message> {
        if self.state.show_api_key_input {
            column()
                .push(text(fl!("settings-api-key-title")).size(16))
                .push(
                    text_input(
                        fl!("settings-api-key-placeholder"),
                        &self.state.api_key_input,
                    )
                    .on_input(Message::UpdateApiKeyInput)
                    .password()
                    .width(Length::Fill),
                )
                .push(
                    row()
                        .push(button::text(fl!("common-cancel")).on_press(Message::HideApiKeyInput))
                        .push(horizontal_space())
                        .push(
                            button::suggested(fl!("settings-api-key-save")).on_press_maybe(
                                (self.state.api_key_input.trim().len()
                                    >= crate::constants::MIN_API_KEY_LENGTH)
                                    .then(|| Message::SetApiKey(self.state.api_key_input.clone())),
                            ),
                        )
                        .spacing(self.space_s)
                        .width(Length::Fill),
                )
                .spacing(self.space_s)
        } else {
            let is_configured = !self.state.is_locked;
            column()
                .push(text(fl!("settings-api-key-title")).size(16))
                .push(
                    row()
                        .push(
                            text(if is_configured {
                                fl!("settings-api-key-configured")
                            } else {
                                fl!("settings-api-key-not-configured")
                            })
                            .size(14),
                        )
                        .push(horizontal_space())
                        .push(if is_configured {
                            button::destructive(fl!("settings-api-key-remove"))
                                .on_press(Message::RemoveApiKey)
                        } else {
                            button::standard(fl!("settings-api-key-add-button"))
                                .on_press(Message::ShowApiKeyInput)
                        })
                        .push(if is_configured {
                            button::standard(fl!("settings-api-key-change"))
                                .on_press(Message::ShowApiKeyInput)
                        } else {
                            button::text("")
                        })
                        .spacing(self.space_s)
                        .align_y(Alignment::Center)
                        .width(Length::Fill),
                )
                .spacing(self.space_s)
        }
    }

    fn units_section(&self) -> Element<'a, Message> {
        let units = [
            "Temperature",
            "Wind Speed",
            "Pressure",
            "Precipitation",
            "Distance",
        ];
        let content =
            units
                .into_iter()
                .enumerate()
                .fold(column().spacing(0), |content, (index, unit)| {
                    let content = if index == 0 {
                        content
                    } else {
                        content.push(widget::divider::horizontal::light())
                    };
                    content.push(crate::ui::compact_unit_row(unit, &self.state.config))
                });

        container(content.width(Length::Fill))
            .padding(12)
            .width(Length::Fill)
            .class(cosmic::theme::Container::Card)
            .into()
    }
}
