// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod config;
mod error;
mod vpn_service;
#[cfg(windows)]
pub mod windows_service;

pub use config::{
    config_dir, create_config_file, log_dir, read_config_file, write_config_file,
    DEFAULT_GLOBAL_CONFIG_FILE, DEFAULT_LOG_FILE,
};
pub use error::{
    AccountError, SetNetworkError, VpnServiceConnectError, VpnServiceDeleteLogFileError,
    VpnServiceDisconnectError,
};
pub use vpn_service::{
    ConnectArgs, ConnectOptions, NymVpnService, VpnServiceCommand, VpnServiceInfo,
};
