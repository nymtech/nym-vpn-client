// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::tunnel_settings::TunnelNetworkSettings;
use crate::VpnError;
use std::{fmt::Debug, os::fd::RawFd, sync::Arc};

#[uniffi::export(with_foreign)]
pub trait ConnectivityObserver: Send + Sync + std::fmt::Debug {
    fn on_network_change(&self, is_online: bool);
}

#[uniffi::export(with_foreign)]
pub trait AndroidTunProvider: Send + Sync + Debug {
    fn bypass(&self, socket: i32);
    fn configure_tunnel(&self, config: TunnelNetworkSettings) -> Result<RawFd, VpnError>;

    fn add_connectivity_observer(&self, observer: Arc<dyn ConnectivityObserver>);
    fn remove_connectivity_observer(&self, observer: Arc<dyn ConnectivityObserver>);
}

/// Adapter type for `AndroidTunProvider` defined by `nym_vpn_lib`
#[derive(Debug, Clone)]
pub struct AndroidTunProviderImpl {
    inner: Arc<dyn AndroidTunProvider>,
}

impl AndroidTunProviderImpl {
    pub fn new(inner: Arc<dyn AndroidTunProvider>) -> Self {
        Self { inner }
    }
}

impl nym_vpn_lib::AndroidTunProvider for AndroidTunProviderImpl {
    fn bypass(&self, socket: i32) {
        self.inner.bypass(socket);
    }

    fn configure_tunnel(
        &self,
        config: nym_vpn_lib::tunnel_provider::TunnelSettings,
    ) -> std::io::Result<RawFd> {
        self.inner
            .configure_tunnel(config.into())
            .map_err(|e| std::io::Error::other(e.to_string()))
    }

    fn add_connectivity_observer(
        &self,
        observer: Arc<dyn nym_vpn_lib::tunnel_provider::ConnectivityObserver>,
    ) {
        self.inner
            .add_connectivity_observer(ConnectivityObserverImpl::new(observer));
    }

    fn remove_connectivity_observer(
        &self,
        observer: Arc<dyn nym_vpn_lib::tunnel_provider::ConnectivityObserver>,
    ) {
        self.inner
            .remove_connectivity_observer(ConnectivityObserverImpl::new(observer));
    }
}

#[derive(Debug, Clone)]
pub struct ConnectivityObserverImpl {
    inner: Arc<dyn ConnectivityObserver>,
}

impl ConnectivityObserverImpl {
    pub fn new(inner: Arc<dyn ConnectivityObserver>) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl nym_vpn_lib::tunnel_provider::ConnectivityObserver for ConnectivityObserverImpl {
    fn on_network_change(&self, is_online: bool) {
        self.inner.on_network_change(is_online);
    }
}
