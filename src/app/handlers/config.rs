// SPDX-License-Identifier: GPL-3.0-or-later

//! Configuration and settings handlers

use crate::app::{AppState, Message};
use crate::weather::{DistanceUnit, PrecipitationUnit, PressureUnit, TemperatureUnit, WindSpeedUnit};
use cosmic::Task;

pub fn handle_set_api_key(state: &mut AppState, api_key: String) -> Task<cosmic::Action<Message>> {
    match state.config.set_api_key(&api_key) {
        Ok(()) => {
            tracing::info!("[CONFIG] API key stored successfully");
            state.unlock();
            state.api_key_input.clear();

            // Always fetch stations when API key is set
            // This will either populate the station list for selection
            // or reload the configured station
            state.update(Message::FetchStations)
        }
        Err(e) => {
            tracing::error!("[CONFIG] Failed to store API key: {}", e);
            Task::none()
        }
    }
}

pub fn handle_update_api_key_input(
    state: &mut AppState,
    input: String,
) -> Task<cosmic::Action<Message>> {
    state.api_key_input = input;
    Task::none()
}

pub fn handle_remove_api_key(state: &mut AppState) -> Task<cosmic::Action<Message>> {
    match state.config.remove_api_key() {
        Ok(()) => {
            tracing::info!("[CONFIG] API key removed successfully");
            state.lock();
        }
        Err(e) => {
            tracing::error!("[CONFIG] Failed to remove API key: {}", e);
        }
    }
    Task::none()
}

pub fn handle_show_api_key_input(state: &mut AppState) -> Task<cosmic::Action<Message>> {
    state.show_api_key_input = true;
    state.api_key_input.clear();
    Task::none()
}

pub fn handle_hide_api_key_input(state: &mut AppState) -> Task<cosmic::Action<Message>> {
    state.show_api_key_input = false;
    state.api_key_input.clear();
    Task::none()
}

pub fn handle_change_temperature_unit(
    state: &mut AppState,
    unit: TemperatureUnit,
) -> Task<cosmic::Action<Message>> {
    state.config.temperature_unit = unit;
    state.save_config()
}

pub fn handle_set_wind_speed_unit(
    state: &mut AppState,
    unit: WindSpeedUnit,
) -> Task<cosmic::Action<Message>> {
    state.config.wind_speed_unit = unit;
    state.save_config()
}

pub fn handle_set_pressure_unit(
    state: &mut AppState,
    unit: PressureUnit,
) -> Task<cosmic::Action<Message>> {
    state.config.pressure_unit = unit;
    state.save_config()
}

pub fn handle_set_precipitation_unit(
    state: &mut AppState,
    unit: PrecipitationUnit,
) -> Task<cosmic::Action<Message>> {
    state.config.precipitation_unit = unit;
    state.save_config()
}

pub fn handle_set_distance_unit(
    state: &mut AppState,
    unit: DistanceUnit,
) -> Task<cosmic::Action<Message>> {
    state.config.distance_unit = unit;
    state.save_config()
}

pub fn handle_set_notifications(
    state: &mut AppState,
    enabled: bool,
) -> Task<cosmic::Action<Message>> {
    state.config.notifications_enabled = enabled;
    tracing::info!(
        "[CONFIG] Notifications {}",
        if enabled { "enabled" } else { "disabled" }
    );
    state.save_config()
}

pub fn handle_set_lightning_notifications(
    state: &mut AppState,
    enabled: bool,
) -> Task<cosmic::Action<Message>> {
    state.config.lightning_notifications_enabled = enabled;
    tracing::info!(
        "[CONFIG] Lightning notifications {}",
        if enabled { "enabled" } else { "disabled" }
    );
    state.save_config()
}

pub fn handle_set_rain_notifications(
    state: &mut AppState,
    enabled: bool,
) -> Task<cosmic::Action<Message>> {
    state.config.rain_notifications_enabled = enabled;
    tracing::info!(
        "[CONFIG] Rain notifications {}",
        if enabled { "enabled" } else { "disabled" }
    );
    state.save_config()
}

pub fn handle_set_wind_notifications(
    state: &mut AppState,
    enabled: bool,
) -> Task<cosmic::Action<Message>> {
    state.config.wind_notifications_enabled = enabled;
    tracing::info!(
        "[CONFIG] Wind notifications {}",
        if enabled { "enabled" } else { "disabled" }
    );
    state.save_config()
}
