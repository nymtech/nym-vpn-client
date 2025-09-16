// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_registration_client::GatewayData;

pub mod connected_tunnel;

#[cfg(target_os = "ios")]
pub mod dns64;
#[cfg(unix)]
pub mod fd;
pub mod two_hop_config;

pub struct ConnectionData {
    pub entry: GatewayData,
    pub exit: GatewayData,
}
