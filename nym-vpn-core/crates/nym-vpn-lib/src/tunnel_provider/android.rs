// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::tunnel_settings::TunnelNetworkSettings;
use crate::platform::error::VpnError;
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
