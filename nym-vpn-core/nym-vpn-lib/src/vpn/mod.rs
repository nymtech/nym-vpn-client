// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod common;
mod messages;
mod mixnet;
mod start;
mod wireguard;

#[cfg(target_os = "ios")]
use crate::platform::swift::OSTunProvider;

pub(crate) use common::{NymVpnExitError, MIXNET_CLIENT_STARTUP_TIMEOUT_SECS};
pub(crate) use mixnet::{MixnetConnectionInfo, MixnetExitConnectionInfo, MixnetVpn};
pub(crate) use wireguard::{WireguardConnectionInfo, WireguardVpn};

pub use common::{GenericNymVpnConfig, NymVpn, SpecificVpn};
pub use messages::{NymVpnCtrlMessage, NymVpnExitStatusMessage, NymVpnStatusMessage};
pub use mixnet::MixnetClientConfig;
pub use start::{spawn_nym_vpn, spawn_nym_vpn_with_new_runtime, NymVpnHandle};
