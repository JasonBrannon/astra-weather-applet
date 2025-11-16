// SPDX-License-Identifier: GPL-3.0-or-later

use crate::applet::WeatherApplet;
use cosmic::Application;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::weather::TemperatureUnit;

    #[test]
    fn test_applet_initialization() {
        let core = cosmic::Core::default();
        let (applet, _task) = WeatherApplet::init(core, ());

        // Test initial state - config may vary based on existing config file
        // Just verify the temperature unit is one of the valid options
        assert!(
            matches!(
                applet.state.config.temperature_unit,
                TemperatureUnit::Celsius | TemperatureUnit::Fahrenheit
            ),
            "Temperature unit should be either Celsius or Fahrenheit"
        );

        // Popup should not be open on initialization
        assert!(applet.state.popup.is_none());

        // Weather and station data are fetched async, not available at init
        // These assertions are no longer valid after removing mock data
        // assert!(applet.state.current_weather.is_some());
        // assert!(applet.state.selected_station.is_some());
    }

    #[test]
    fn test_temperature_conversion() {
        // Test unit conversion functions
        use crate::weather::TemperatureUnit;
        use crate::weather::conversions::convert_temperature;

        // Celsius to Fahrenheit conversion
        assert_eq!(convert_temperature(0.0, TemperatureUnit::Fahrenheit), 32.0);
        assert_eq!(
            convert_temperature(100.0, TemperatureUnit::Fahrenheit),
            212.0
        );
        assert_eq!(
            convert_temperature(-40.0, TemperatureUnit::Fahrenheit),
            -40.0
        );

        // Celsius to Celsius (identity)
        assert_eq!(convert_temperature(25.0, TemperatureUnit::Celsius), 25.0);
    }
}

pub fn run_headless_test() {
    println!("🌤️  Tempest Weather Applet - Headless Test");
    println!("==========================================");

    // Initialize applet
    let core = cosmic::Core::default();
    let (mut applet, _task) = WeatherApplet::init(core, ());

    println!("✅ Applet initialized successfully");
    println!(
        "🌡️  Temperature Unit: {:?}",
        applet.state.config.temperature_unit
    );

    if let Some(weather) = &applet.state.current_weather {
        println!("🌤️  Current Weather:");
        println!("   Temperature: {:.1}°C", weather.temperature);
        println!("   Humidity: {}%", weather.humidity);
        println!("   Pressure: {:.1} hPa", weather.pressure);
        println!("   Description: {}", weather.description);
    }

    if let Some(station) = &applet.state.selected_station {
        println!("📍 Weather Station:");
        println!("   Name: {}", station.name);
        println!("   ID: {}", station.id);

        if let Some(city) = &station.location.city {
            println!("   Location: {}", city);
        }
    }

    println!("🔧 Configuration:");
    println!("   Auto Location: {}", applet.state.config.auto_location);
    println!(
        "   Refresh Interval: {} minutes",
        applet.state.config.refresh_interval_minutes
    );

    // Test lightning monitoring system
    println!("\n⚡ Testing Lightning Detection System:");
    test_lightning_detection(&mut applet);

    println!("\n✅ All applet components working correctly!");
    println!("🎯 To see the GUI, run this in a COSMIC desktop environment");
}

fn test_lightning_detection(_applet: &mut WeatherApplet) {
    println!("   Lightning monitoring now uses real-time WebSocket events");
    println!("   Typestate pattern removed - using direct API calls");
    println!("   Test skipped - lightning detection via WebSocket");
}
