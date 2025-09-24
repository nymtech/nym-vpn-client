// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Types providing a bridge between uniffi and nym-vpn-lib-types.

uniffi::setup_scaffolding!();

mod account_controller;
mod error;
mod gateway_directory;
mod ip_pair;
mod ipnetwork;
mod network_config;
mod service;
mod std;
mod time;
mod tunnel_state_machine;
mod url;
mod user_agent;
mod vpn_api_client;

// Uses wildcard imports to pick up generated uniffi constants that need to be made visible for use in the library.
pub use account_controller::*;
pub use error::*;
pub use gateway_directory::*;
pub use ip_pair::*;
pub use ipnetwork::*;
pub use network_config::*;
pub use service::*;
pub use std::*;
pub use time::*;
pub use tunnel_state_machine::*;
pub use url::*;
pub use user_agent::*;
pub use vpn_api_client::*;
