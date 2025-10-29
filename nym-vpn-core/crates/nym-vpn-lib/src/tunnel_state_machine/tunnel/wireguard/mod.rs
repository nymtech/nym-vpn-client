// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::SocketAddr;

use crate::tunnel_state_machine::TunnelMetadata;
use nym_registration_common::GatewayData;

pub mod connected_tunnel;

#[cfg(target_os = "ios")]
pub mod dns64;
#[cfg(unix)]
pub mod fd;
pub mod two_hop_config;

#[derive(Debug, Clone)]
pub struct ConnectionData {
    pub entry_bridge_addr: Option<SocketAddr>,
    pub entry: GatewayData,
    pub exit: GatewayData,
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
            MetadataEvent::TunnelMetadata(_metadata) => {
                #[cfg(target_os = "linux")]
                {
                    nym_wg_metadata_client::TunUpSendData::InterfaceName(_metadata.interface)
                }

                #[cfg(not(target_os = "linux"))]
                {
                    nym_wg_metadata_client::TunUpSendData::Signal
                }
            }
        }
    }
}

pub type MetadataSender = tokio::sync::oneshot::Sender<MetadataEvent>;
pub type MetadataReceiver = tokio::sync::oneshot::Receiver<MetadataEvent>;
