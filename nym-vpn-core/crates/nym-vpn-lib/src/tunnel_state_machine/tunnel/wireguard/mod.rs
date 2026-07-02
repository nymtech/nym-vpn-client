// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::tunnel_state_machine::TunnelMetadata;
use nym_registration_common::WireguardConfiguration;
use std::net::SocketAddr;
use tokio::sync::oneshot;

use nym_vpn_lib_types::BridgeAddress;

pub mod connected_tunnel;

#[cfg(target_os = "ios")]
pub mod dns64;
#[cfg(target_os = "android")]
pub mod dns_filter_proxy;
#[cfg(unix)]
pub mod fd;
pub mod two_hop_config;

#[derive(Debug)]
pub struct ConnectionData {
    pub entry_bridge_addr: Option<BridgeAddress>,
    pub entry: WireguardConfiguration,
    pub exit: WireguardConfiguration,
}

impl ConnectionData {
    /// Returns effective entry endpoint set to bridge listen endpoint when entry bridge address is available. Otherwise, returns the wireguard entry endpoint.
    pub fn effective_entry_endpoint(&self) -> SocketAddr {
        self.entry_bridge_addr
            .as_ref()
            .map(|addr| addr.listen_addr)
            .unwrap_or(self.entry.endpoint)
    }

    /// Returns effective entry gateway data set to bridge listen endpoint when entry bridge address is available, along with exit data
    pub fn into_effective_entry_and_exit_data(
        self,
    ) -> (WireguardConfiguration, WireguardConfiguration) {
        let effective_entry_endpoint = self.effective_entry_endpoint();
        let mut entry_wireguard_config = self.entry;
        entry_wireguard_config.endpoint = effective_entry_endpoint;
        (entry_wireguard_config, self.exit)
    }

    /// Returns effective *remote* entry endpoint set to bridge remote endpoint when entry bridge address is available. Otherwise, returns the wireguard entry endpoint.
    pub fn effective_remote_entry_endpoint(&self) -> SocketAddr {
        self.entry_bridge_addr
            .as_ref()
            .map(|addr| addr.remote_addr)
            .unwrap_or(self.entry.endpoint)
    }
}

pub struct MetadataEvent {
    metadata_endpoint_reachable_tx: oneshot::Sender<bool>,
    event_type: MetadataEventType,
}

impl MetadataEvent {
    pub fn new_proxy(
        metadata_endpoint_reachable_tx: oneshot::Sender<bool>,
        addr: SocketAddr,
    ) -> Self {
        Self {
            metadata_endpoint_reachable_tx,
            event_type: MetadataEventType::MetadataProxy(addr),
        }
    }

    pub fn new_tunnel(
        metadata_endpoint_reachable_tx: oneshot::Sender<bool>,
        tunnel_metadata: TunnelMetadata,
    ) -> Self {
        Self {
            metadata_endpoint_reachable_tx,
            event_type: MetadataEventType::TunnelMetadata(tunnel_metadata),
        }
    }
}

pub enum MetadataEventType {
    MetadataProxy(SocketAddr),
    TunnelMetadata(TunnelMetadata),
}

impl From<MetadataEvent> for nym_wg_metadata_client::TunUpSendData {
    fn from(event: MetadataEvent) -> Self {
        match event.event_type {
            MetadataEventType::MetadataProxy(proxy_addr) => nym_wg_metadata_client::TunUpSendData {
                metadata_endpoint_reachable_tx: event.metadata_endpoint_reachable_tx,
                data_type: nym_wg_metadata_client::TunUpSendDataType::TcpProxy(proxy_addr),
            },
            MetadataEventType::TunnelMetadata(metadata) => nym_wg_metadata_client::TunUpSendData {
                metadata_endpoint_reachable_tx: event.metadata_endpoint_reachable_tx,
                data_type: nym_wg_metadata_client::TunUpSendDataType::InterfaceName(
                    metadata.interface,
                ),
            },
        }
    }
}

pub type MetadataSender = tokio::sync::oneshot::Sender<MetadataEvent>;
pub type MetadataReceiver = tokio::sync::oneshot::Receiver<MetadataEvent>;
