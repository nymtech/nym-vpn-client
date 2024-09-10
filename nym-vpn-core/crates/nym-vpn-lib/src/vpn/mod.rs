// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod base;
mod messages;
mod mixnet;
mod start;
mod wireguard;

pub(crate) use base::MIXNET_CLIENT_STARTUP_TIMEOUT_SECS;
pub(crate) use mixnet::{MixnetConnectionInfo, MixnetExitConnectionInfo, MixnetVpn};
pub(crate) use wireguard::{WireguardConnectionInfo, WireguardVpn};

pub use base::{GenericNymVpnConfig, NymVpn, SpecificVpn};
pub use messages::{NymVpnCtrlMessage, NymVpnExitStatusMessage, NymVpnStatusMessage};
pub use mixnet::MixnetClientConfig;
pub use start::{spawn_nym_vpn, spawn_nym_vpn_with_new_runtime, NymVpnHandle};
