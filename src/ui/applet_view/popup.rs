// SPDX-License-Identifier: GPL-3.0-or-later

use super::{navigation::NavigationView, tabs::TabsView};
use crate::app::messages::AppTab;
use crate::app::{AppState, Message};
use cosmic::widget::{column, text};
use cosmic::{Element, cosmic_theme, theme};

pub(super) struct PopupView<'a> {
    state: &'a AppState,
    id: cosmic::iced::window::Id,
}

impl<'a> PopupView<'a> {
    pub(super) fn new(state: &'a AppState, id: cosmic::iced::window::Id) -> Self {
        Self { state, id }
    }

    pub(super) fn render(self) -> Element<'a, Message> {
        render_popup(self.state, self.id)
    }
}

fn render_popup(state: &AppState, id: cosmic::iced::window::Id) -> Element<'_, Message> {
    // Only show content for our popup window
    if state.popup != Some(id) {
        return text("").into();
    }

    let cosmic_theme::Spacing {
        space_s, space_m, ..
    } = theme::active().cosmic().spacing;

    // Build content based on locked state
    let content = if state.is_locked {
        crate::ui::lockdown_view(state)
    } else {
        // Force Stations tab if station selection is required and we're not already on it
        // This handles the case where popup opens with wrong tab selected
        let should_force_stations =
            state.needs_station_selection && state.current_tab != AppTab::Stations;

        if should_force_stations {
            tracing::debug!("Forcing Stations tab because station selection is required");
        }

        column()
            .push(NavigationView::new(state, space_s).render())
            .push(TabsView::new(state, space_s, space_m).render())
            .spacing(space_m)
            .padding(space_m)
            .into()
    };

    // Use popup_container() like official COSMIC applets (applies correct theme)
    state
        .core
        .applet
        .popup_container(content)
        .limits(
            cosmic::iced::Limits::NONE
                .min_width(900.0)
                .max_width(900.0)
                .max_height(550.0),
        )
        .into()
}
