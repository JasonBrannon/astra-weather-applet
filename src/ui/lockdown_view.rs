// SPDX-License-Identifier: GPL-3.0-or-later

//! Lockdown view displayed when no API key is configured

use crate::app::{AppState, Message};
use cosmic::iced::{Alignment, Length};
use cosmic::prelude::*;
use cosmic::widget::{self, button, column, container, row, text, text_input};
use cosmic::{cosmic_theme, theme};

pub fn lockdown_view(state: &AppState) -> Element<'_, Message> {
    let cosmic_theme::Spacing {
        space_xs,
        space_s,
        space_m,
        ..
    } = theme::active().cosmic().spacing;

    // Header section with title and description - tightly grouped
    let header = column()
        .push(text("API Key Required").size(16).width(Length::Fill))
        .push(text(
            "Enter your WeatherFlow Tempest API key below to access weather data",
        ))
        .spacing(space_xs); // Minimal spacing for header group

    // Instructions section with "How to get" - moderate spacing
    let instructions = column()
        .push(text("How to get your API key:").size(16))
        .push(
            row()
                .push(text("1. Visit "))
                .push(
                    button::link("https://tempestwx.com/settings/tokens").on_press(
                        Message::OpenUrl("https://tempestwx.com/settings/tokens".to_string()),
                    ),
                )
                .push(widget::icon::from_name("external-link-symbolic").size(12))
                .spacing(4)
                .align_y(Alignment::Center),
        )
        .push(text("2. Log in to your WeatherFlow account"))
        .push(text("3. Create a new personal access token"))
        .push(text("4. Copy the token and paste it above"))
        .spacing(space_xs); // Tight spacing for instruction steps

    // Main content column with controlled spacing between major sections
    let content =
        column()
            .push(header)
            .push(api_key_input_section(state, space_s))
            .push(widget::divider::horizontal::default())
            .push(instructions)
            .push(
                row()
                    .push(button::standard("Open WeatherFlow Token Page").on_press(
                        Message::OpenUrl("https://tempestwx.com/settings/tokens".to_string()),
                    ))
                    .push(widget::horizontal_space())
                    .width(Length::Fill),
            )
            .spacing(space_s) // Use smaller spacing for main sections
            .width(Length::Fill);

    // Wrap in container to match other views' sizing behavior
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(space_m)
        .into()
}

fn api_key_input_section(state: &AppState, space_s: u16) -> Element<'_, Message> {
    column()
        .push(
            text_input("Paste your Tempest API key here...", &state.api_key_input)
                .on_input(Message::UpdateApiKeyInput)
                .width(Length::Fill)
                .password(),
        )
        .push(
            // COSMIC HIG: Primary action right, secondary left
            row()
                .push(button::text("Cancel").on_press(Message::UpdateApiKeyInput(String::new())))
                .push(widget::horizontal_space())
                .push(button::suggested("Save API Key").on_press_maybe(
                    if state.api_key_input.trim().len() >= crate::constants::MIN_API_KEY_LENGTH {
                        Some(Message::SetApiKey(state.api_key_input.clone()))
                    } else {
                        None
                    },
                ))
                .spacing(space_s)
                .width(Length::Fill),
        )
        .spacing(space_s)
        .into()
}
