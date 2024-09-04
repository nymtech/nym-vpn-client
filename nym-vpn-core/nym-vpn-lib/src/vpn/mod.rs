// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod base;
mod messages;
mod mixnet;
mod start;
mod wireguard;

pub use base::{GenericNymVpnConfig, NymVpn, SpecificVpn};
pub(crate) use base::{NymVpnExitError, MIXNET_CLIENT_STARTUP_TIMEOUT_SECS};
pub use messages::{NymVpnCtrlMessage, NymVpnExitStatusMessage, NymVpnStatusMessage};
pub use mixnet::MixnetClientConfig;
pub(crate) use mixnet::{MixnetConnectionInfo, MixnetExitConnectionInfo, MixnetVpn};
pub use start::{spawn_nym_vpn, spawn_nym_vpn_with_new_runtime, NymVpnHandle};
pub(crate) use wireguard::{WireguardConnectionInfo, WireguardVpn};
