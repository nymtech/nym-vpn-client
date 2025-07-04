// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod config;
mod error;
mod vpn_service;
#[cfg(windows)]
pub mod windows_service;

pub use config::{
    DEFAULT_GLOBAL_CONFIG_FILE, DEFAULT_LOG_FILE, DEFAULT_OLD_LOG_FILE, config_dir,
    create_config_file, log_dir, read_config_file, write_config_file,
};
pub use error::SetNetworkError;
pub use vpn_service::{
    ListCountriesOptions, ListGatewaysOptions, NymVpnService, VpnServiceCommand,
};
