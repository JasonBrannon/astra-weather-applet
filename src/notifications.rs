// SPDX-License-Identifier: GPL-3.0-or-later

use crate::weather::LightningEvent;
use notify_rust::{Notification, Timeout};

pub struct NotificationManager;

impl NotificationManager {
    pub fn new() -> Self {
        Self
    }

    pub fn send_lightning_notification(
        &self,
        event: &LightningEvent,
        distance_unit: crate::weather::DistanceUnit,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let title = "⚡ Lightning Detected!";

        let body = if let Some(distance_km) = event.distance_km {
            // Convert distance to user's preferred unit
            let converted_distance =
                crate::weather::conversions::convert_distance(distance_km, distance_unit);
            format!(
                "{} lightning detected at {:.1} {} away\nIntensity: {:?}\nStrikes: {}",
                event.strike_type,
                converted_distance,
                distance_unit,
                event.intensity,
                event.strike_count
            )
        } else {
            format!(
                "{} lightning detected\nIntensity: {:?}\nStrikes: {}",
                event.strike_type, event.intensity, event.strike_count
            )
        };

        let mut notification = Notification::new();
        notification
            .summary(title)
            .body(&body)
            .icon("weather-storm")
            .timeout(Timeout::Milliseconds(5000)) // Show for 5 seconds
            .urgency(notify_rust::Urgency::Normal);

        // Set app name and show notification
        notification.appname("Astra Weather");
        notification.show()?;

        tracing::info!("Lightning notification sent: {}", body);
        Ok(())
    }

    pub fn send_notification(
        &self,
        title: &str,
        body: &str,
        timeout: Timeout,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut notification = Notification::new();
        notification
            .summary(title)
            .body(body)
            .icon("weather-storm")
            .timeout(timeout)
            .urgency(notify_rust::Urgency::Normal);

        notification.appname("Astra Weather");
        notification.show()?;

        tracing::info!("Notification sent: {} - {}", title, body);
        Ok(())
    }
}
