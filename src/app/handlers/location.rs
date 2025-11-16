// SPDX-License-Identifier: GPL-3.0-or-later

//! Location search handlers

use crate::app::{AppState, Message};
use cosmic::Task;

pub fn handle_search_location(
    state: &mut AppState,
    query: String,
) -> Task<cosmic::Action<Message>> {
    state.location_search_query = query.clone();
    // In a real implementation, this would perform an async search
    // For now, we'll simulate some search results
    let mock_results = vec![
        crate::weather::Location {
            latitude: 40.7128,
            longitude: -74.0060,
            city: Some("New York".to_string()),
            country: Some("US".to_string()),
        },
        crate::weather::Location {
            latitude: 34.0522,
            longitude: -118.2437,
            city: Some("Los Angeles".to_string()),
            country: Some("US".to_string()),
        },
    ];
    state.location_search_results = mock_results
        .into_iter()
        .filter(|loc| {
            loc.city
                .as_ref()
                .map(|city| city.to_lowercase().contains(&query.to_lowercase()))
                .unwrap_or(false)
        })
        .collect();
    Task::none()
}

pub fn handle_location_search_result(
    state: &mut AppState,
    results: Vec<crate::weather::Location>,
) -> Task<cosmic::Action<Message>> {
    state.location_search_results = results;
    Task::none()
}
