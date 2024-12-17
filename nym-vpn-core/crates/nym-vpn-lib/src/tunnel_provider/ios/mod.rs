// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::tunnel_settings::TunnelNetworkSettings;
use crate::platform::error::VpnError;

pub mod interface;

#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait OSTunProvider: Send + Sync + std::fmt::Debug {
    /// Set network settings including tun, dns, ip.
    async fn set_tunnel_network_settings(
        &self,
        tunnel_settings: TunnelNetworkSettings,
    ) -> Result<(), VpnError>;
}
