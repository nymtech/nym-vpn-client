// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod config;
mod error;
mod socks5;
mod vpn_service;

pub use config::{
    ConfigSetupError, DEFAULT_GLOBAL_CONFIG_FILE_JSON, DEFAULT_GLOBAL_CONFIG_FILE_TOML,
    DEFAULT_LOG_FILE, DEFAULT_OLD_LOG_FILE, config_dir, log_dir, read_json_config_file,
    read_toml_config_file, write_json_config_file,
};
pub use error::SetNetworkError;
pub use socks5::{
    Socks5Error, Socks5Service, Socks5Status, socks5_idle_timeout, socks5_request_timeout,
};
pub use vpn_service::{NymVpnService, NymVpnServiceParameters, VpnServiceCommand};
