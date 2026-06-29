// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::tunnel_state_machine::TunnelMetadata;
use nym_registration_common::WireguardConfiguration;
use std::net::SocketAddr;

use nym_vpn_lib_types::BridgeAddress;

pub mod connected_tunnel;

#[cfg(target_os = "ios")]
pub mod dns64;
#[cfg(target_os = "android")]
pub mod dns_filter_proxy;
#[cfg(unix)]
pub mod fd;
#[cfg(unix)]
pub mod metadata_tcp_proxy;
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

pub enum MetadataEvent {
    MetadataProxy(SocketAddr),
    TunnelMetadata(TunnelMetadata),
}

impl From<MetadataEvent> for nym_wg_metadata_client::TunUpSendData {
    fn from(event: MetadataEvent) -> Self {
        match event {
            MetadataEvent::MetadataProxy(proxy_addr) => {
                nym_wg_metadata_client::TunUpSendData::TcpProxy(proxy_addr)
            }
            MetadataEvent::TunnelMetadata(metadata) => {
                nym_wg_metadata_client::TunUpSendData::InterfaceName(metadata.interface)
            }
        }
    }
}

pub type MetadataSender = tokio::sync::oneshot::Sender<MetadataEvent>;
pub type MetadataReceiver = tokio::sync::oneshot::Receiver<MetadataEvent>;

pub(crate) fn single_tun_exit_metadata_event(
    exit_proxy_addr: Option<SocketAddr>,
    fallback_tunnel: TunnelMetadata,
) -> MetadataEvent {
    match exit_proxy_addr {
        Some(proxy_addr) => MetadataEvent::MetadataProxy(proxy_addr),
        None => MetadataEvent::TunnelMetadata(fallback_tunnel),
    }
}

pub(crate) fn two_tunnel_bandwidth_metadata_events(
    entry: &TunnelMetadata,
    exit: &TunnelMetadata,
) -> (MetadataEvent, MetadataEvent) {
    (
        MetadataEvent::TunnelMetadata(entry.clone()),
        MetadataEvent::TunnelMetadata(exit.clone()),
    )
}

#[cfg(test)]
mod bandwidth_metadata_events_tests {
    use std::net::{IpAddr, Ipv4Addr};

    use super::*;

    fn sample_metadata(interface: &str) -> TunnelMetadata {
        TunnelMetadata {
            interface: interface.to_string(),
            ips: vec![IpAddr::V4(Ipv4Addr::new(10, 1, 0, 2))],
            ipv4_gateway: None,
            ipv6_gateway: None,
        }
    }

    #[test]
    fn single_tun_exit_uses_metadata_proxy_when_listener_started() {
        let exit = sample_metadata("utun42");
        let proxy = SocketAddr::from(([127, 0, 0, 1], 50607));

        assert!(matches!(
            single_tun_exit_metadata_event(Some(proxy), exit),
            MetadataEvent::MetadataProxy(addr) if addr == proxy
        ));
    }

    #[test]
    fn single_tun_exit_falls_back_to_tunnel_metadata_when_proxy_unavailable() {
        let exit = sample_metadata("utun42");

        assert!(matches!(
            single_tun_exit_metadata_event(None, exit.clone()),
            MetadataEvent::TunnelMetadata(metadata) if metadata.interface == exit.interface
        ));
    }

    #[test]
    fn single_tun_exit_proxy_maps_to_tcp_proxy_transport() {
        let exit = sample_metadata("utun42");
        let proxy = SocketAddr::from(([127, 0, 0, 1], 50607));
        let event = single_tun_exit_metadata_event(Some(proxy), exit);

        assert!(matches!(
            nym_wg_metadata_client::TunUpSendData::from(event),
            nym_wg_metadata_client::TunUpSendData::TcpProxy(addr) if addr == proxy
        ));
    }

    #[test]
    fn two_tunnel_uses_tunnel_metadata_for_entry_and_exit() {
        let entry = sample_metadata("wg0");
        let exit = sample_metadata("wg1");

        let (entry_event, exit_event) = two_tunnel_bandwidth_metadata_events(&entry, &exit);

        assert!(matches!(
            entry_event,
            MetadataEvent::TunnelMetadata(metadata) if metadata.interface == entry.interface
        ));
        assert!(matches!(
            exit_event,
            MetadataEvent::TunnelMetadata(metadata) if metadata.interface == exit.interface
        ));
    }
}
