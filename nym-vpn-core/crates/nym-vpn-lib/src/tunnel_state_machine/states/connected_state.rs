// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use std::net::SocketAddr;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use nym_dns::ResolvedDnsConfig;
use nym_vpn_lib_types::ErrorStateReason;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[cfg(target_os = "macos")]
use crate::tunnel_state_machine::resolver::LOCAL_DNS_RESOLVER;
use crate::tunnel_state_machine::{
    ConnectionData, NextTunnelState, PrivateActionAfterDisconnect, PrivateTunnelState, SharedState,
    TunnelCommand, TunnelInterface, TunnelStateHandler,
    states::{ConnectingState, DisconnectingState},
    tunnel::SelectedGateways,
    tunnel_monitor::{TunnelMonitorEvent, TunnelMonitorEventReceiver, TunnelMonitorHandle},
};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use crate::tunnel_state_machine::{Error, Result, gateway_ext::GatewayExt};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use nym_common::trace_err_chain;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use nym_firewall::{AllowedClients, AllowedEndpoint, Endpoint, FirewallPolicy, TransportProtocol};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use nym_vpn_lib_types::TunnelConnectionData;

use super::ErrorState;

pub struct ConnectedState {
    tunnel_monitor_handle: TunnelMonitorHandle,
    tunnel_monitor_event_receiver: TunnelMonitorEventReceiver,
    selected_gateways: SelectedGateways,
    #[cfg_attr(any(target_os = "android", target_os = "ios"), allow(unused))]
    tunnel_interface: TunnelInterface,
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    firewall_policy_params: ConnectedPolicyParameters,
}

impl ConnectedState {
    pub async fn enter(
        tunnel_interface: TunnelInterface,
        connection_data: ConnectionData,
        selected_gateways: SelectedGateways,
        tunnel_monitor_handle: TunnelMonitorHandle,
        tunnel_monitor_event_receiver: TunnelMonitorEventReceiver,
        shared_state: &mut SharedState,
    ) -> (Box<dyn TunnelStateHandler>, PrivateTunnelState) {
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        let wg_entry_endpoint =
            if let TunnelConnectionData::Wireguard(ref wg) = connection_data.tunnel {
                if shared_state.tunnel_settings.bridges_enabled() {
                    // this will be `Some` if we get to the connected state with bridges enabled.
                    wg.entry_bridge_addr.as_ref().map(|addr| addr.remote_addr)
                } else {
                    Some(wg.entry.endpoint)
                }
            } else {
                None
            };

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        let firewall_policy_params = ConnectedPolicyParameters {
            enable_ipv6: shared_state.tunnel_settings.enable_ipv6,
            allow_lan: shared_state.tunnel_settings.allow_lan,
            wg_entry_endpoint,
            ws_entry_endpoints: selected_gateways.entry_gateway().endpoints(),
            dns_config: shared_state.tunnel_settings.resolved_dns_config(),
            tunnel_interface: tunnel_interface.clone(),
        };

        let connected_state = Self {
            tunnel_monitor_handle,
            tunnel_monitor_event_receiver,
            selected_gateways,
            tunnel_interface,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            firewall_policy_params,
        };

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        if let Err(e) =
            Self::set_firewall_policy(shared_state, &connected_state.firewall_policy_params)
        {
            trace_err_chain!(e, "failed to apply firewall policy");
            return DisconnectingState::enter(
                PrivateActionAfterDisconnect::Error(ErrorStateReason::SetFirewallPolicy),
                connected_state.tunnel_monitor_handle,
                shared_state,
            )
            .await;
        } else if let Err(e) = connected_state.set_dns(shared_state).await {
            trace_err_chain!(e, "failed to set dns");
            return DisconnectingState::enter(
                PrivateActionAfterDisconnect::Error(ErrorStateReason::SetDns),
                connected_state.tunnel_monitor_handle,
                shared_state,
            )
            .await;
        }

        // Reset DNS resolver overrides since connections can be established over the tunnel
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        shared_state.reset_resolver_overrides().await;

        // We can use slower network fetches now
        shared_state.topology_provider.use_network(true).await;

        (
            Box::new(connected_state),
            PrivateTunnelState::Connected { connection_data },
        )
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    fn set_firewall_policy(
        shared_state: &mut SharedState,
        params: &ConnectedPolicyParameters,
    ) -> Result<()> {
        let policy = params.as_policy();

        shared_state
            .firewall
            .apply_policy(policy)
            .map_err(Error::SetFirewallPolicy)
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    async fn set_dns(&self, shared_state: &mut SharedState) -> Result<()> {
        let dns_config = shared_state.tunnel_settings.resolved_dns_config();
        let tunnel_metadata = self.tunnel_interface.exit_tunnel_metadata();

        #[cfg(any(target_os = "linux", target_os = "windows"))]
        shared_state
            .dns_handler
            .set(tunnel_metadata.interface.clone(), dns_config)
            .await
            .map_err(Error::SetDns)?;

        #[cfg(target_os = "macos")]
        // We do not want to forward DNS queries to *our* local resolver if we do not run a local
        // DNS resolver *or* if the DNS config points to a loopback address.
        if *LOCAL_DNS_RESOLVER {
            let ips = dns_config.addresses().collect::<Vec<_>>();
            tracing::debug!("Enabling local DNS forwarder to: {:?}", ips);
            shared_state.filtering_resolver.enable_forward(ips).await;
        } else {
            tracing::debug!("Not enabling local DNS resolver");
            shared_state
                .dns_handler
                .set(tunnel_metadata.interface.clone(), dns_config)
                .await
                .map_err(Error::SetDns)?;
        }

        Ok(())
    }

    #[cfg(any(target_os = "linux", target_os = "windows"))]
    async fn reset_dns(shared_state: &mut SharedState) {
        if let Err(error) = shared_state
            .dns_handler
            .reset_before_interface_removal()
            .await
        {
            trace_err_chain!(error, "Failed to reset DNS");
        }
    }

    #[cfg(target_os = "macos")]
    async fn reset_dns(shared_state: &mut SharedState) {
        // On macOS, configure only the local DNS resolver
        if *LOCAL_DNS_RESOLVER {
            shared_state.filtering_resolver.disable_forward().await;
        } else if let Err(error) = shared_state.dns_handler.reset().await {
            trace_err_chain!(error, "Failed to reset DNS");
        }
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    async fn reset_routes(shared_state: &mut SharedState) {
        shared_state.route_handler.remove_routes().await
    }

    async fn disconnect(
        self,
        after_disconnect: PrivateActionAfterDisconnect,
        shared_state: &mut SharedState,
    ) -> NextTunnelState {
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            Self::reset_dns(shared_state).await;
            Self::reset_routes(shared_state).await;
        }

        NextTunnelState::NewState(
            DisconnectingState::enter(after_disconnect, self.tunnel_monitor_handle, shared_state)
                .await,
        )
    }

    async fn handle_tunnel_down(
        self,
        error_state_reason: Option<ErrorStateReason>,
        shared_state: &mut SharedState,
    ) -> NextTunnelState {
        if error_state_reason.is_none() {
            tracing::info!("Tunnel closed. Reconnecting.");
        }

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            Self::reset_dns(shared_state).await;
            Self::reset_routes(shared_state).await;
        }

        match error_state_reason {
            Some(block_reason) => {
                NextTunnelState::NewState(ErrorState::enter(block_reason, shared_state).await)
            }
            None => NextTunnelState::NewState(
                ConnectingState::enter(0, Some(self.selected_gateways), shared_state).await,
            ),
        }
    }
}

#[async_trait::async_trait]
impl TunnelStateHandler for ConnectedState {
    async fn handle_event(
        mut self: Box<Self>,
        shutdown_token: &CancellationToken,
        command_rx: &'async_trait mut mpsc::UnboundedReceiver<TunnelCommand>,
        shared_state: &'async_trait mut SharedState,
    ) -> NextTunnelState {
        tokio::select! {
            Some(command) = command_rx.recv() => {
                tracing::debug!("ConnectedState received command: {command:?}");
                match command {
                    TunnelCommand::Connect => {
                        self.disconnect(PrivateActionAfterDisconnect::Reconnect, shared_state).await
                    },
                    TunnelCommand::Disconnect => {
                        self.disconnect(PrivateActionAfterDisconnect::Nothing, shared_state).await
                    },
                    TunnelCommand::SetTunnelSettings(tunnel_settings) => {
                        let Some(diff) = shared_state.tunnel_settings.diff(&tunnel_settings) else {
                            return NextTunnelState::SameState(self);
                        };

                        #[cfg(not(any(target_os = "android", target_os = "ios")))]
                        {
                            if diff.allow_lan_changed() {
                                self.firewall_policy_params.allow_lan = tunnel_settings.allow_lan;

                                if let Err(e) = Self::set_firewall_policy(shared_state, &self.firewall_policy_params) {
                                    trace_err_chain!(e, "failed to set firewall policy");
                                    return NextTunnelState::NewState(ErrorState::enter(ErrorStateReason::SetFirewallPolicy, shared_state).await);
                                }

                                // If the only change was Allow LAN, then don't restart the tunnel.
                                if diff.only_allow_lan_changed() {
                                    shared_state.tunnel_settings.allow_lan = tunnel_settings.allow_lan;
                                    return NextTunnelState::SameState(self);
                                }
                            }
                        }

                        shared_state.tunnel_settings = tunnel_settings;
                        self.disconnect(PrivateActionAfterDisconnect::Reconnect, shared_state).await
                    }
                }
            }
            Some(monitor_event) = self.tunnel_monitor_event_receiver.recv() => {
                match monitor_event {
                    TunnelMonitorEvent::Down { error_state_reason, reply_tx } => {
                        _ = reply_tx.send(());
                        self.handle_tunnel_down(error_state_reason, shared_state).await
                    }
                    _ => {
                        NextTunnelState::SameState(self)
                    }
                }
            }
            Some(connectivity) = shared_state.connectivity_handle.next() => {
                if connectivity.is_offline() {
                    let after_disconnect = PrivateActionAfterDisconnect::Offline {
                        reconnect: true,
                        gateways: Some(self.selected_gateways.clone())
                    };
                    self.disconnect(after_disconnect, shared_state).await
                } else {
                    NextTunnelState::SameState(self)
                }
            }
            _ = shutdown_token.cancelled() => {
                self.disconnect(PrivateActionAfterDisconnect::Nothing, shared_state).await
            }
        }
    }
}

/// Firewall policy configuration when connected
#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[derive(Debug, Clone)]
struct ConnectedPolicyParameters {
    /// Whether IPv6 is enabled
    enable_ipv6: bool,

    /// Whether to allow LAN traffic
    allow_lan: bool,

    /// WireGuard entry endpoint
    wg_entry_endpoint: Option<SocketAddr>,

    /// Entry gateway websocket endpoints
    ws_entry_endpoints: Vec<SocketAddr>,

    /// Resolved DNS configuration including in-tunnel and out-of-tunnel DNS servers
    dns_config: ResolvedDnsConfig,

    /// Tunnel interface
    tunnel_interface: TunnelInterface,
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
impl ConnectedPolicyParameters {
    pub fn as_policy(&self) -> FirewallPolicy {
        // Allow websocket entry endpoints
        let mut peer_endpoints = self
            .ws_entry_endpoints
            .iter()
            .filter(|addr| addr.is_ipv4() || (self.enable_ipv6 && addr.is_ipv6()))
            .map(|addr| {
                AllowedEndpoint::new(
                    Endpoint::from_socket_address(*addr, TransportProtocol::Tcp),
                    #[cfg(any(target_os = "linux", target_os = "macos"))]
                    AllowedClients::Root,
                    #[cfg(target_os = "windows")]
                    AllowedClients::current_exe(),
                )
            })
            .collect::<Vec<_>>();

        // Allow WireGuard / Quic entry endpoint
        if let Some(addr) = self.wg_entry_endpoint {
            if addr.is_ipv4() || (self.enable_ipv6 && addr.is_ipv6()) {
                let allow_wg_endpoint = AllowedEndpoint::new(
                    Endpoint::from_socket_address(addr, TransportProtocol::Udp),
                    #[cfg(any(target_os = "linux", target_os = "macos"))]
                    AllowedClients::Root,
                    #[cfg(target_os = "windows")]
                    AllowedClients::current_exe(),
                );

                peer_endpoints.push(allow_wg_endpoint);
            } else {
                tracing::warn!("WireGuard endpoint contains IPv6 address, but IPv6 is disabled!");
            }
        }

        let tunnel = nym_firewall::TunnelInterface::from(self.tunnel_interface.clone());

        FirewallPolicy::Connected {
            peer_endpoints,
            tunnel,
            allow_lan: self.allow_lan,
            dns_config: self.dns_config.clone(),
            // todo: split tunneling
            #[cfg(target_os = "macos")]
            redirect_interface: None,
        }
    }
}
