// SPDX-License-Identifier: GPL-3.0-or-later

use crate::app::{AppState, Message};
use crate::config::Config;
use cosmic::cosmic_config::CosmicConfigEntry;
use cosmic::iced::Subscription;
use cosmic::iced::time;
use cosmic::{Element, Task, cosmic_config};
use std::time::Duration;

pub fn run() -> cosmic::iced::Result {
    cosmic::applet::run::<WeatherApplet>(())
}

pub struct WeatherApplet {
    pub state: AppState,
}

impl cosmic::Application for WeatherApplet {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = "com.slagmine.astra";

    fn core(&self) -> &cosmic::Core {
        &self.state.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.state.core
    }

    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        let config = cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
            .map(|context| match Config::get_entry(&context) {
                Ok(config) => config,
                Err((_errors, config)) => config,
            })
            .unwrap_or_default();

        tracing::info!(
            "🚀 Applet initializing - API key present: {}, configured station: {:?}",
            config.has_api_key(),
            config.selected_station_id
        );

        let state = AppState::new(core, config);

        let applet = WeatherApplet { state };

        // Fetch stations on startup if we have an API key
        // This will trigger station restoration if selected_station_id is in config
        let init_task = if !applet.state.is_locked {
            tracing::info!("[INIT] API key found - scheduling station fetch for restoration");
            // Schedule station fetch as an async task to avoid blocking init
            // Use a small delay to ensure the applet is fully initialized first
            Task::perform(
                async {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                },
                |_| cosmic::Action::App(Message::FetchStations),
            )
        } else {
            tracing::warn!("🔒 App locked - no API key, skipping station fetch");
            Task::none()
        };

        (applet, init_task)
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let mut subscriptions = vec![
            // Weather data refresh (less frequent)
            time::every(Duration::from_secs(
                self.state.config.refresh_interval_minutes as u64 * 60,
            ))
            .map(|_| Message::Tick),
            // Forecast refresh (hourly)
            time::every(Duration::from_secs(
                crate::constants::FORECAST_REFRESH_INTERVAL_MINUTES as u64 * 60,
            ))
            .map(|_| Message::ForecastTick),
            // Lightning flash animation update (WebSocket provides real-time lightning events)
            time::every(Duration::from_millis(500)).map(|_| Message::UpdateLightningFlash),
            // Periodic health check - increased frequency for faster sleep/hibernate recovery
            // Checks network availability and WebSocket health every 5 seconds
            time::every(Duration::from_secs(5)).map(|_| Message::PeriodicHealthCheck),
        ];

        // Add WebSocket event polling if receiver exists
        if self.state.websocket_event_receiver.is_some() {
            subscriptions
                .push(time::every(Duration::from_millis(100)).map(|_| Message::WebSocketPoll));
        }

        Subscription::batch(subscriptions)
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        self.state.update(message)
    }

    fn view(&self) -> Element<'_, Self::Message> {
        self.state.view()
    }

    fn view_window(&self, id: cosmic::iced::window::Id) -> Element<'_, Self::Message> {
        self.state.view_window(id)
    }

    fn on_close_requested(&self, id: cosmic::iced::window::Id) -> Option<Self::Message> {
        if self.state.popup == Some(id) {
            Some(Message::PopupClosed(id))
        } else {
            None
        }
    }
}
