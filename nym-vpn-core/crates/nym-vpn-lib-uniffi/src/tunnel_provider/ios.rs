// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use super::tunnel_settings::TunnelNetworkSettings;
use crate::VpnError;

#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait OSTunProvider: Send + Sync + std::fmt::Debug {
    /// Set network settings including tun, dns, ip.
    async fn set_tunnel_network_settings(
        &self,
        tunnel_settings: TunnelNetworkSettings,
    ) -> Result<(), VpnError>;
}

/// Adapter type for `OSTunProvider` defined by `nym_vpn_lib`
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct OSTunProviderImpl {
    inner: Arc<dyn OSTunProvider>,
}

impl OSTunProviderImpl {
    pub fn new(inner: Arc<dyn OSTunProvider>) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl nym_vpn_lib::tunnel_provider::OSTunProvider for OSTunProviderImpl {
    async fn set_tunnel_network_settings(
        &self,
        tunnel_settings: nym_vpn_lib::tunnel_provider::TunnelSettings,
    ) -> std::io::Result<()> {
        self.inner
            .set_tunnel_network_settings(tunnel_settings.into())
            .await
            .map_err(|e| std::io::Error::other(e.to_string()))
    }
}
