// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod command;
mod nym_vpn_service;
mod response;
mod shared_state;
mod state;

pub(crate) use {
    command::{ConnectArgs, ConnectOptions, VpnServiceCommand},
    response::{
        VpnServiceConnectResult, VpnServiceDisconnectResult, VpnServiceInfoResult,
        VpnServiceStatusResult,
    },
    shared_state::VpnServiceStateChange,
    state::ConnectedStateDetails,
};

pub(super) use {
    nym_vpn_service::NymVpnService,
    shared_state::SharedVpnState,
    state::{
        MixConnectedStateDetails, VpnConnectedStateDetails, VpnState, WgConnectedStateDetails,
    },
};
