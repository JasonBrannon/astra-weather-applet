// SPDX-License-Identifier: GPL-3.0-or-later

//! Message handlers organized by category
//!
//! This module splits the monolithic `update()` function into focused handlers.
//! Each submodule handles a specific category of messages.

pub mod ui;
pub mod config;
pub mod location;
pub mod lifecycle;
pub mod forecast;
pub mod network;
pub mod stations;
pub mod weather;
pub mod websocket;
