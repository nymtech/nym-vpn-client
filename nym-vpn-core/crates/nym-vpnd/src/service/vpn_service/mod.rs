// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod command;
mod nym_vpn_service;
mod response;
mod shared_state;
mod state;

pub(crate) use {
    command::{ConnectArgs, ConnectOptions, VpnServiceCommand},
    nym_vpn_service::NymVpnService,
    response::{
        VpnServiceConnectResult, VpnServiceDisconnectResult, VpnServiceInfoResult,
        VpnServiceStatusResult,
    },
    shared_state::{SharedVpnState, VpnServiceStateChange},
    state::{
        ConnectedStateDetails, MixConnectedStateDetails, VpnConnectedStateDetails, VpnState,
        WgConnectedStateDetails,
    },
};
