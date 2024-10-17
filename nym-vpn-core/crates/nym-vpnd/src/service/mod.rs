// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod config;
mod error;
mod vpn_service;

pub(crate) use config::{default_log_dir, DEFAULT_LOG_FILE};
pub(crate) use error::{AccountError, ConnectionFailedError};
pub(crate) use vpn_service::{
    ConnectArgs, ConnectOptions, ConnectedStateDetails, NymVpnService, VpnServiceCommand,
    VpnServiceConnectResult, VpnServiceDisconnectResult, VpnServiceInfoResult,
    VpnServiceStateChange, VpnServiceStatusResult,
};
