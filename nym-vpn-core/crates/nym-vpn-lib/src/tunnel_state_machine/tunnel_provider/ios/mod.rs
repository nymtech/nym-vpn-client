// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use super::tunnel_settings::TunnelNetworkSettings;
use crate::platform::error::VpnError;

pub mod default_path_observer;
pub mod interface;

use default_path_observer::OSDefaultPathObserver;

#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait OSTunProvider: Send + Sync + std::fmt::Debug {
    /// Set network settings including tun, dns, ip.
    async fn set_tunnel_network_settings(
        &self,
        tunnel_settings: TunnelNetworkSettings,
    ) -> Result<(), VpnError>;

    /// Set or unset the default path observer.
    fn set_default_path_observer(
        &self,
        observer: Option<Arc<dyn OSDefaultPathObserver>>,
    ) -> Result<(), VpnError>;
}
