// SPDX-License-Identifier: GPL-3.0-or-later

use crate::config::Config;
use crate::weather::{
    DistanceUnit, ExtendedForecast, LightningEvent, PrecipitationUnit, PressureUnit,
    RainStartEvent, RapidWindEvent, StationHealth, TemperatureUnit, TempestObservation,
    WeatherData, WeatherStation, WindSpeedUnit,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTab {
    Forecast,
    Weather,
    Stations,
    Alerts,
    Settings,
    About,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    ForecastTick,
    WebSocketChannelDisconnected,
    #[allow(dead_code)]
    LightningCheck,
    ToggleWindow,
    PopupClosed(cosmic::iced::window::Id),
    #[allow(dead_code)]
    RefreshWeather,
    #[allow(dead_code)]
    WeatherDataReceived(WeatherData),
    #[allow(dead_code)]
    StationSelected(WeatherStation),
    #[allow(dead_code)]
    ConfigUpdate(Config),
    ChangeTemperatureUnit(TemperatureUnit),
    SelectStation(WeatherStation),
    #[allow(dead_code)]
    ToggleAutoLocation,
    #[allow(dead_code)]
    SetRefreshInterval(u32),
    #[allow(dead_code)]
    LightningDetected(LightningEvent),
    UpdateLightningFlash,
    SetNotifications(bool),
    SelectTab(AppTab),
    #[allow(dead_code)]
    ForecastDataReceived(ExtendedForecast),
    #[allow(dead_code)]
    RefreshForecast,
    #[allow(dead_code)]
    StationHealthReceived(Vec<StationHealth>),
    #[allow(dead_code)]
    RefreshStations,
    #[allow(dead_code)]
    ToggleStationStatus(String),
    SetApiKey(String),
    UpdateApiKeyInput(String),
    RemoveApiKey,
    ShowApiKeyInput,
    HideApiKeyInput,
    #[allow(dead_code)]
    SearchLocation(String),
    #[allow(dead_code)]
    LocationSearchResult(Vec<crate::weather::Location>),
    FetchStations,
    StationsListReceived(Vec<WeatherStation>),
    SetDefaultStation(WeatherStation),
    FetchWeatherData,
    WeatherDataFetched(Result<(WeatherData, TempestObservation), String>),
    #[allow(dead_code)]
    WebSocketConnected,
    #[allow(dead_code)]
    WebSocketDisconnected,
    #[allow(dead_code)]
    WebSocketReconnect,
    #[allow(dead_code)]
    WebSocketLightningStrike(LightningEvent),
    #[allow(dead_code)]
    WebSocketRainStart(RainStartEvent),
    #[allow(dead_code)]
    WebSocketRapidWind(RapidWindEvent),
    #[allow(dead_code)]
    WebSocketWindDirection(f64),
    WebSocketEventReceived(crate::weather::WebSocketEvent),
    SetWindSpeedUnit(WindSpeedUnit),
    SetPressureUnit(PressureUnit),
    SetPrecipitationUnit(PrecipitationUnit),
    SetDistanceUnit(DistanceUnit),
    ToggleForecastView,
    ForecastLoadComplete,
    #[allow(dead_code)]
    DetailedObservationReceived(TempestObservation),
    OpenUrl(String),
    PeriodicHealthCheck,
    NetworkAvailabilityChecked(bool), // Result of network availability check
    ShowStationsView,                 // Open popup and switch to Stations tab
    BetterForecastReceived(Result<crate::weather::BetterForecastResponse, String>),
    HourlyForecastScrolled(f32), // Scroll offset for hourly forecast
    ShowUnitSelector(UnitSelectorType),
    CloseUnitSelector,
    // Per-event notification toggles
    SetLightningNotifications(bool),
    SetRainNotifications(bool),
    SetWindNotifications(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitSelectorType {
    Temperature,
    WindSpeed,
    Pressure,
    Precipitation,
    Distance,
}
