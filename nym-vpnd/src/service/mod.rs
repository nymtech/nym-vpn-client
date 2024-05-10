// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod config;
mod exit_listener;
mod start;
mod status_listener;
mod vpn_service;

pub(crate) use start::start_vpn_service;
pub(crate) use vpn_service::{
    ConnectArgs, ConnectOptions, VpnServiceCommand, VpnServiceConnectResult,
    VpnServiceDisconnectResult, VpnServiceImportUserCredentialResult, VpnServiceStateChange,
    VpnServiceStatusResult,
};
