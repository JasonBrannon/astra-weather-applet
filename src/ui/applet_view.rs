// SPDX-License-Identifier: GPL-3.0-or-later

use crate::app::{AppState, Message};
use cosmic::Element;

use self::panel::PanelView;
use self::popup::PopupView;

mod navigation;
mod panel;
mod popup;
mod tabs;

impl AppState {
    pub fn view(&self) -> Element<'_, Message> {
        PanelView::new(self).render()
    }

    pub fn view_window(&self, id: cosmic::iced::window::Id) -> Element<'_, Message> {
        PopupView::new(self, id).render()
    }
}
