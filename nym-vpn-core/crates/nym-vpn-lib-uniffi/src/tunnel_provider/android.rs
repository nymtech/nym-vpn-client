// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{fmt::Debug, os::fd::RawFd, sync::Arc};

use super::tunnel_settings::TunnelNetworkSettings;
use crate::VpnError;

/// Per-connection app bypass ("steering") configuration.
///
/// Passing this to `NymVpnServiceCommandSender::set_app_bypass` turns on in-tunnel routing of
/// the excluded apps' traffic, which is the only way to keep them connected under VPN
/// lockdown. Passing `None` turns it off, leaving per-app exclusion to
/// `VpnService.Builder.addDisallowedApplication`.
#[derive(uniffi::Record, Clone, Debug)]
pub struct AppBypassConfig {
    /// UIDs of the apps that must bypass the tunnel.
    pub excluded_uids: Vec<u32>,

    /// DNS servers of the underlying (non-VPN) network, as IP address strings.
    /// Unparseable entries are ignored.
    pub underlying_dns: Vec<String>,

    /// Route local-network destinations directly (LAN bypass) instead of over
    /// the tunnel, so "allow local network access" keeps working under lockdown.
    pub bypass_lan: bool,

    /// The underlying network's real local subnet(s) as CIDR strings (e.g.
    /// "10.223.228.0/24"). Only the device's actual local network(s), not the
    /// whole RFC1918 space, so the tunnel's own in-tunnel addresses stay tunneled.
    pub lan_prefixes: Vec<String>,
}

impl From<AppBypassConfig> for nym_vpn_lib::tunnel_provider::AppBypassConfig {
    fn from(config: AppBypassConfig) -> Self {
        Self {
            excluded_uids: config.excluded_uids,
            underlying_dns: config
                .underlying_dns
                .iter()
                .filter_map(|addr| {
                    addr.parse()
                        .inspect_err(|e| {
                            tracing::warn!("Ignoring unparseable underlying dns server {addr}: {e}")
                        })
                        .ok()
                })
                .collect(),
            bypass_lan: config.bypass_lan,
            lan_prefixes: config.lan_prefixes,
        }
    }
}

/// Abstract Android tunnel provider.
#[uniffi::export(with_foreign)]
pub trait AndroidTunProvider: Send + Sync + Debug {
    /// Bypass VPN for a given socket.
    fn bypass(&self, socket: i32);

    /// Configure VPN tunnel with the given settings returning a file descriptor to tunnel device that can be used to read and write packets.
    fn configure_tunnel(&self, config: TunnelNetworkSettings) -> Result<RawFd, VpnError>;

    /// Resolve the UID owning a connection (ConnectivityManager.getConnectionOwnerUid).
    /// Returns -1 when unknown. protocol: 6 = TCP, 17 = UDP.
    fn get_connection_owner_uid(&self, protocol: i32, source: String, destination: String) -> i32;
}

/// Adapter type for `nym_vpn_lib::tun_provider::AndroidTunProvider`
#[derive(Debug, Clone)]
pub struct AndroidTunProviderImpl {
    inner: Arc<dyn AndroidTunProvider>,
}

impl AndroidTunProviderImpl {
    pub fn new(inner: Arc<dyn AndroidTunProvider>) -> Self {
        Self { inner }
    }
}

impl nym_vpn_lib::tunnel_provider::AndroidTunProvider for AndroidTunProviderImpl {
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

    fn get_connection_owner_uid(&self, protocol: i32, source: String, destination: String) -> i32 {
        self.inner
            .get_connection_owner_uid(protocol, source, destination)
    }
}
