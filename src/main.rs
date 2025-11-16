// SPDX-License-Identifier: GPL-3.0-or-later

mod app;
mod applet;
mod config;
mod constants;
mod i18n;
mod notifications;
mod security;
mod test_applet;
mod time_utils;
mod ui;
mod weather;

fn main() -> cosmic::iced::Result {
    // Check if running in headless mode
    if std::env::args().any(|arg| arg == "--test" || arg == "--headless") {
        test_applet::run_headless_test();
        return Ok(());
    }

    // Set up tracing for logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Get the system's preferred languages.
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    // Enable localizations to be applied.
    i18n::init(&requested_languages);

    // Run the applet
    applet::run()
}
