// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "ios")]
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    sync::Arc,
};

#[cfg(target_os = "ios")]
use ipnetwork::IpNetwork;
#[cfg(target_os = "macos")]
use nym_dns::DnsConfig;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[cfg(any(
    target_os = "linux",
    target_os = "macos",
    target_os = "windows",
    target_os = "ios"
))]
use nym_common::trace_err_chain;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use nym_firewall::FirewallPolicy;

#[cfg(target_os = "ios")]
use crate::tunnel_provider::{OSTunProvider, TunnelSettings};
#[cfg(target_os = "macos")]
use crate::tunnel_state_machine::resolver::LOCAL_DNS_RESOLVER;
#[cfg(target_os = "ios")]
use crate::tunnel_state_machine::tunnel::wireguard::two_hop_config::MIN_IPV6_MTU;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use crate::tunnel_state_machine::{Error, Result};
use crate::tunnel_state_machine::{
    ErrorStateReason, NextTunnelState, PrivateTunnelState, SharedState, TunnelCommand,
    TunnelStateHandler,
    states::{ConnectingState, DisconnectedState, OfflineState},
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
        shared_state: &mut SharedState,
    ) -> (Box<dyn TunnelStateHandler>, PrivateTunnelState) {
        #[cfg(target_os = "macos")]
        if !reason.prevents_filtering_resolver() {
            // Set system DNS to our local DNS resolver
            if Self::set_local_dns_resolver(shared_state).await.is_err() {
                return Box::pin(Self::enter(ErrorStateReason::SetDns, shared_state)).await;
            }
        }

        #[cfg(target_os = "ios")]
        {
            Self::set_blocking_network_settings(shared_state.tun_provider.clone()).await;
        }

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        if let Err(e) = Self::set_firewall_policy(shared_state) {
            trace_err_chain!(e, "Failed to apply firewall policy for blocked state");
        }
        let _ = shared_state
            .account_command_tx
            .set_vpn_api_firewall_up()
            .await;
        (Box::new(Self), PrivateTunnelState::Error(reason))
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    fn set_firewall_policy(shared_state: &mut SharedState) -> Result<()> {
        let policy = FirewallPolicy::Blocked {
            allow_lan: shared_state.tunnel_settings.allow_lan,
            allowed_endpoints: Vec::new(),
        };

        shared_state
            .firewall
            .apply_policy(policy)
            .map_err(Error::SetFirewallPolicy)
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    fn reset_firewall_policy(shared_state: &mut SharedState) {
        if let Err(e) = shared_state.firewall.reset_policy() {
            trace_err_chain!(e, "Failed to reset firewall policy");
        }
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    async fn reset_dns(shared_state: &mut SharedState) {
        if let Err(error) = shared_state.dns_handler.reset().await {
            trace_err_chain!(error, "Unable to disable filtering resolver");
        }
    }

    #[cfg(target_os = "macos")]
    async fn set_local_dns_resolver(shared_state: &mut SharedState) -> Result<()> {
        // Set system DNS to our local DNS resolver
        let system_dns = DnsConfig::default().resolve(
            &[shared_state.filtering_resolver.listen_addr().ip()],
            shared_state.filtering_resolver.listen_addr().port(),
        );
        shared_state
            .dns_handler
            .set("lo".to_owned(), system_dns)
            .await
            .inspect_err(|err| {
                trace_err_chain!(err, "Failed to configure system to use filtering resolver");
            })
            .map_err(Error::SetDns)
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
            .set_tunnel_network_settings(tunnel_network_settings)
            .await
        {
            trace_err_chain!(e, "Failed to set tunnel network settings");
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
                        #[cfg(target_os = "macos")]
                        if !*LOCAL_DNS_RESOLVER {
                            // This is probably unnecessary, since DNS is already configured on the
                            // primary interface.
                            Self::reset_dns(shared_state).await;
                        }

                        #[cfg(any(target_os = "linux", target_os = "windows"))]
                        Self::reset_dns(shared_state).await;

                        if shared_state.connectivity_handle.connectivity().await.is_offline() {
                            NextTunnelState::NewState(OfflineState::enter(true, None, shared_state).await)
                        } else {
                            NextTunnelState::NewState(ConnectingState::enter(0, None, shared_state).await)
                        }
                    },
                    TunnelCommand::Disconnect => {
                        #[cfg(not(any(target_os = "android", target_os = "ios")))]
                        Self::reset_dns(shared_state).await;

                        if shared_state.connectivity_handle.connectivity().await.is_offline() {
                            NextTunnelState::NewState(OfflineState::enter(false, None, shared_state).await)
                        } else {
                            NextTunnelState::NewState(DisconnectedState::enter(None, shared_state).await)
                        }
                    },
                    TunnelCommand::SetAllowLan(allow_lan) => {
                        if shared_state.set_allow_lan(allow_lan) {
                            todo!()
                        } else {
                            NextTunnelState::SameState(self)
                        }
                    },
                    TunnelCommand::SetTunnelSettings(tunnel_settings) => {
                        shared_state.tunnel_settings = tunnel_settings;
                        NextTunnelState::SameState(self)
                    }
                }
            }
            _ = shutdown_token.cancelled() => {
                #[cfg(not(any(target_os = "android", target_os = "ios")))]
                {
                    Self::reset_dns(shared_state).await;
                    Self::reset_firewall_policy(shared_state);
                }
                NextTunnelState::Finished
            }
        }
    }
}
