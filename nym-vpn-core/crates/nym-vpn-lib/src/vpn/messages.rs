// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tracing::error;

use super::{MixnetConnectionInfo, MixnetExitConnectionInfo, WireguardConnectionInfo};

#[derive(thiserror::Error, Clone, Debug)]
pub enum NymVpnStatusMessage {
    #[error("mixnet connection info")]
    MixConnectionInfo {
        mixnet_connection_info: MixnetConnectionInfo,
        mixnet_exit_connection_info: Box<MixnetExitConnectionInfo>,
    },
    #[error("wireguard connection info")]
    WgConnectionInfo {
        entry_connection_info: WireguardConnectionInfo,
        exit_connection_info: WireguardConnectionInfo,
    },
}

#[derive(Debug)]
pub enum NymVpnCtrlMessage {
    Stop,
}

#[derive(Debug)]
pub enum NymVpnExitStatusMessage {
    Stopped,
    Failed(Box<dyn std::error::Error + Send + Sync + 'static>),
}
