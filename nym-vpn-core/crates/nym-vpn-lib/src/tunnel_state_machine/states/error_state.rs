// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "ios")]
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    sync::Arc,
};

#[cfg(target_os = "ios")]
use ipnetwork::IpNetwork;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[cfg(target_os = "ios")]
use crate::tunnel_provider::{ios::OSTunProvider, tunnel_settings::TunnelSettings};
#[cfg(target_os = "ios")]
use crate::tunnel_state_machine::tunnel::wireguard::two_hop_config::MIN_IPV6_MTU;
use crate::tunnel_state_machine::{
    states::{ConnectingState, DisconnectedState, OfflineState},
    ErrorStateReason, NextTunnelState, PrivateTunnelState, SharedState, TunnelCommand,
    TunnelStateHandler,
};

/// Interface addresses used as placeholders when in error state.
#[cfg(target_os = "ios")]
const BLOCKING_INTERFACE_ADDRS: [IpAddr; 2] = [
    IpAddr::V4(Ipv4Addr::new(169, 254, 0, 10)),
    IpAddr::V6(Ipv6Addr::new(
        0xfdcc, 0x9fc0, 0xe75a, 0x53c3, 0xfa25, 0x241f, 0x21c0, 0x70d0,
    )),
];

pub struct ErrorState;

impl ErrorState {
    pub async fn enter(
        reason: ErrorStateReason,
        _shared_state: &mut SharedState,
    ) -> (Box<dyn TunnelStateHandler>, PrivateTunnelState) {
        #[cfg(target_os = "ios")]
        {
            Self::set_blocking_network_settings(_shared_state.tun_provider.clone()).await;
        }

        (Box::new(Self), PrivateTunnelState::Error(reason))
    }

    /// Configure tunnel with network settings blocking all traffic
    #[cfg(target_os = "ios")]
    async fn set_blocking_network_settings(tun_provider: Arc<dyn OSTunProvider>) {
        let tunnel_network_settings = TunnelSettings {
            remote_addresses: vec![],
            interface_addresses: BLOCKING_INTERFACE_ADDRS.map(IpNetwork::from).to_vec(),
            dns_servers: vec![],
            mtu: MIN_IPV6_MTU,
        };

        if let Err(e) = tun_provider
            .set_tunnel_network_settings(tunnel_network_settings.into_tunnel_network_settings())
            .await
        {
            tracing::error!("Failed to set tunnel network settings: {}", e);
        }
    }
}

#[async_trait::async_trait]
impl TunnelStateHandler for ErrorState {
    async fn handle_event(
        mut self: Box<Self>,
        shutdown_token: &CancellationToken,
        command_rx: &'async_trait mut mpsc::UnboundedReceiver<TunnelCommand>,
        shared_state: &'async_trait mut SharedState,
    ) -> NextTunnelState {
        tokio::select! {
            Some(command) = command_rx.recv() => {
                match command {
                    TunnelCommand::Connect => {
                        if shared_state.offline_monitor.connectivity().await.is_offline() {
                            NextTunnelState::NewState(OfflineState::enter(true,  0, None))
                        } else {
                            NextTunnelState::NewState(ConnectingState::enter(0, None, shared_state).await)
                        }
                    },
                    TunnelCommand::Disconnect => {
                        if shared_state.offline_monitor.connectivity().await.is_offline() {
                            NextTunnelState::NewState(OfflineState::enter(false,  0, None))
                        } else {
                            NextTunnelState::NewState(DisconnectedState::enter())
                        }
                    },
                    TunnelCommand::SetTunnelSettings(tunnel_settings) => {
                        shared_state.tunnel_settings = tunnel_settings;
                        NextTunnelState::SameState(self)
                    }
                }
            }
            _ = shutdown_token.cancelled() => {
                NextTunnelState::Finished
            }
            else => NextTunnelState::Finished
        }
    }
}
