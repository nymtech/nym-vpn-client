// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod config;
mod error;
mod vpn_service;

pub(crate) use config::{
    config_dir, create_config_file, log_dir, read_config_file, write_config_file,
    DEFAULT_GLOBAL_CONFIG_FILE, DEFAULT_LOG_FILE,
};
pub(crate) use error::{
    AccountError, ConnectionFailedError, SetNetworkError, VpnServiceConnectError,
    VpnServiceDisconnectError,
};
pub(crate) use vpn_service::{
    ConnectArgs, ConnectOptions, ConnectedStateDetails, NymVpnService, VpnServiceCommand,
    VpnServiceInfo, VpnServiceStateChange, VpnServiceStatus,
};
