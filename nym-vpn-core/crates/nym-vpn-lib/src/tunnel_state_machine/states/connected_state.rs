// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use std::net::SocketAddr;

#[cfg(any(target_os = "linux", target_os = "windows"))]
use nym_dns::DnsConfig;

use nym_vpn_lib_types::ErrorStateReason;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[cfg(target_os = "ios")]
use crate::tunnel_state_machine::Result;
use crate::tunnel_state_machine::{
    ConnectionData, NextTunnelState, PrivateActionAfterDisconnect, PrivateTunnelState, SharedState,
    TunnelCommand, TunnelInterface, TunnelStateHandler,
    states::{ConnectingState, DisconnectingState},
    tunnel::SelectedGateways,
    tunnel_monitor::{TunnelMonitorEvent, TunnelMonitorEventReceiver, TunnelMonitorHandle},
};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use crate::tunnel_state_machine::{Error, Result, gateway_ext::GatewayExt};
#[cfg(not(any(target_os = "android")))]
use nym_common::trace_err_chain;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use nym_firewall::{
    AllowedClients, AllowedDns, AllowedEndpoint, Endpoint, FirewallPolicy, TransportProtocol,
};
use nym_http_api_client::HickoryDnsResolver;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use nym_vpn_lib_types::TunnelConnectionData;

use super::ErrorState;

pub struct ConnectedState {
    tunnel_monitor_handle: TunnelMonitorHandle,
    tunnel_monitor_event_receiver: TunnelMonitorEventReceiver,
    selected_gateways: SelectedGateways,
    #[cfg_attr(
        not(any(target_os = "linux", target_os = "windows", target_os = "ios")),
        allow(unused)
    )]
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
        // Real tunnel replaced the blocking placeholder via configure_tunnel; drop the stale FD.
        #[cfg(target_os = "android")]
        shared_state.clear_android_blocking_tun();

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
        let firewall_policy_params = {
            // Include entry gateway WebSocket endpoints
            let mut ws_endpoints = selected_gateways.entry_gateway().endpoints();
            // Also include exit gateway WebSocket endpoints for SOCKS5 support in 2-hop mode.
            // These endpoints are whitelisted in firewall rules (peer_endpoints), allowing SOCKS5
            // to establish direct connections to the exit gateway
            ws_endpoints.extend(selected_gateways.exit_gateway().endpoints());

            #[cfg(target_os = "macos")]
            let redirect_interface = shared_state.split_tunnel.interface().await;

            ConnectedPolicyParameters {
                enable_ipv6: shared_state.tunnel_settings.enable_ipv6,
                allow_lan: shared_state.tunnel_settings.allow_lan,
                wg_entry_endpoint,
                ws_entry_endpoints: ws_endpoints,
                dns_config: AllowedDns::new_with_tunnel_dns(
                    shared_state.tunnel_settings.allowed_dns_endpoints(),
                ),
                tunnel_interface: tunnel_interface.clone(),
                #[cfg(target_os = "macos")]
                redirect_interface,
            }
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
                Some(connected_state.tunnel_monitor_handle),
                shared_state,
            )
            .await;
        }

        // point the internal DNS resolver to the system so that it routes over the tunnel
        // using the custom / commodity DNS flow while in the connected state
        HickoryDnsResolver::shared().use_system_resolver();

        #[cfg(not(any(target_os = "android")))]
        if let Err(e) = connected_state.set_dns(shared_state).await {
            trace_err_chain!(e, "failed to set dns");
            return DisconnectingState::enter(
                PrivateActionAfterDisconnect::Error(ErrorStateReason::SetDns),
                Some(connected_state.tunnel_monitor_handle),
                shared_state,
            )
            .await;
        }

        shared_state
            .recents_manager
            .add_recent(
                connection_data.tunnel.tunnel_type(),
                connection_data.entry_gateway.id.clone(),
                connection_data.exit_gateway.id.clone(),
            )
            .await;

        // Statistics reports must be sent through a socket bound to the tunnel interface,
        // since the packet tunnel provider's traffic is otherwise excluded from the tunnel.
        #[cfg(target_os = "ios")]
        shared_state
            .statistics_event_sender
            .report_tunnel_interface(Some(
                connected_state
                    .tunnel_interface
                    .exit_tunnel_metadata()
                    .interface
                    .clone(),
            ));

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

        nym_http_api_client::network_reconfigured();
        shared_state
            .firewall
            .apply_policy(policy)
            .map_err(Error::SetFirewallPolicy)
    }

    #[cfg(not(target_os = "android"))]
    async fn set_dns(&self, shared_state: &mut SharedState) -> Result<()> {
        let dns_config = shared_state.tunnel_settings.resolver_config();

        #[cfg(any(target_os = "linux", target_os = "windows", target_os = "ios"))]
        let tunnel_metadata = self.tunnel_interface.exit_tunnel_metadata();

        tracing::debug!(
            "Enabling local DNS forwarder to: {}",
            dns_config
                .iter()
                .map(|ns| {
                    let protos = ns
                        .connections
                        .iter()
                        .map(|conn| format!("{}/{}", conn.port, conn.protocol.to_protocol()))
                        .collect::<Vec<_>>()
                        .join(",");
                    format!("{} ({})", ns.ip, protos)
                })
                .collect::<Vec<_>>()
                .join(", ")
        );

        if let Err(err) = shared_state
            .filtering_resolver
            .enable_forward(
                dns_config,
                #[cfg(target_os = "ios")]
                Some(tunnel_metadata.interface.clone()),
            )
            .await
        {
            trace_err_chain!(err, "failed to enable dns forwarding");
        }

        #[cfg(any(target_os = "linux", target_os = "windows"))]
        {
            // Point the tunnel interface DNS at the local filtering resolver so that the OS actually
            // sends DNS queries to it.
            let listen_addr = shared_state.filtering_resolver.listen_addr();
            let system_dns = DnsConfig {
                addresses: vec![listen_addr.ip()],
                port: listen_addr.port(),
            };
            shared_state
                .dns_handler
                .set(&tunnel_metadata.interface, system_dns)
                .await
                .map_err(Error::SetDns)?;
        }

        Ok(())
    }

    #[cfg(not(target_os = "android"))]
    async fn reset_dns(shared_state: &mut SharedState) {
        if let Err(err) = shared_state.filtering_resolver.disable_forward().await {
            trace_err_chain!(err, "failed to disable dns forwarding");
        }

        #[cfg(any(target_os = "linux", target_os = "windows"))]
        if let Err(error) = shared_state
            .dns_handler
            .reset_before_interface_removal()
            .await
        {
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
        Self::prepare_for_disconnect(shared_state).await;

        NextTunnelState::NewState(
            DisconnectingState::enter(
                after_disconnect,
                Some(self.tunnel_monitor_handle),
                shared_state,
            )
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

        Self::prepare_for_disconnect(shared_state).await;

        match error_state_reason {
            Some(block_reason) => {
                NextTunnelState::NewState(ErrorState::enter(block_reason, shared_state).await)
            }
            None => NextTunnelState::NewState(
                ConnectingState::enter(0, Some(self.selected_gateways), shared_state).await,
            ),
        }
    }

    async fn prepare_for_disconnect(shared_state: &mut SharedState) {
        #[cfg(target_os = "ios")]
        shared_state
            .statistics_event_sender
            .report_tunnel_interface(None);

        // Revert the internal resolver to use the configured nameserver group
        HickoryDnsResolver::shared().use_configured_resolver();
        nym_http_api_client::network_reconfigured();

        #[cfg(not(target_os = "android"))]
        Self::reset_dns(shared_state).await;

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        Self::reset_routes(shared_state).await;

        #[cfg(target_os = "android")]
        let _ = shared_state; // Avoid unused variable warning
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
                        let diff = shared_state.tunnel_settings.diff(&tunnel_settings);
                        if diff.is_empty() {
                            return NextTunnelState::SameState(self);
                        }
                        shared_state.set_tunnel_settings(tunnel_settings).await;

                        #[cfg(not(any(target_os = "android", target_os = "ios")))]
                        let mut new_firewall_policy = self.firewall_policy_params.clone();
                        #[cfg(not(any(target_os = "android", target_os = "ios")))]
                        {
                            new_firewall_policy.allow_lan = shared_state.tunnel_settings.allow_lan;
                        }

                        #[cfg(any(target_os = "macos", target_os = "windows"))]
                        {
                            if diff.split_tunnel_changed() || diff.geo_exclusion_enabled_changed() {
                                match shared_state.set_split_tunnel_exclude_paths().await {
                                    Ok(interface_changed) => {
                                        if interface_changed {
                                            #[cfg(target_os = "macos")]
                                            {
                                                new_firewall_policy.redirect_interface = shared_state.split_tunnel.interface().await;
                                            }
                                        }
                                    }
                                    Err(st_error_cause) => {
                                        let after_disconnect = match st_error_cause {
                                            nym_split_tunnel::SplitTunnelErrorCause::Other => {
                                                PrivateActionAfterDisconnect::Error(ErrorStateReason::SplitTunnel)
                                            }
                                            #[cfg(target_os = "macos")]
                                            nym_split_tunnel::SplitTunnelErrorCause::NeedFullDiskPermissions => {
                                                PrivateActionAfterDisconnect::Error(ErrorStateReason::NeedFullDiskPermissions)
                                            }
                                            #[cfg(target_os = "macos")]
                                            nym_split_tunnel::SplitTunnelErrorCause::IsOffline => {
                                                PrivateActionAfterDisconnect::Offline {
                                                    reconnect: true,
                                                    gateways: Some(self.selected_gateways.clone()),
                                                }
                                            }
                                        };
                                        return self.disconnect(after_disconnect, shared_state).await;
                                    }
                                }
                            }
                        }

                        #[cfg(not(target_os = "ios"))]
                        if diff.geo_exclusion_enabled_changed() {
                            shared_state
                                .start_or_stop_socks5_proxy()
                                .await;
                        } else if diff.geo_exclusion_excluded_countries_changed() {
                            shared_state.set_socks5_proxy_excluded_countries();
                        }

                        if diff.enable_ad_blocking_changed() {
                            shared_state.enable_ad_blocking(shared_state.tunnel_settings.enable_ad_blocking).await;
                        }

                        #[cfg(not(any(target_os = "android", target_os = "ios")))]
                        {
                            if new_firewall_policy != self.firewall_policy_params {
                                self.firewall_policy_params = new_firewall_policy;

                                if let Err(e) = Self::set_firewall_policy(shared_state, &self.firewall_policy_params) {
                                    trace_err_chain!(e, "failed to set firewall policy");
                                    return self.disconnect(PrivateActionAfterDisconnect::Error(ErrorStateReason::SetFirewallPolicy), shared_state).await
                                }
                            }
                        }

                        // Not all changes require the tunnel to be reconnected
                        if diff.should_reconnect(shared_state.tunnel_settings.tunnel_type_used()) {
                            self.disconnect(PrivateActionAfterDisconnect::Reconnect, shared_state).await
                        } else {
                            NextTunnelState::SameState(self)
                        }
                    }
                    TunnelCommand::Block(reason) => {
                        self.disconnect(PrivateActionAfterDisconnect::Error(reason), shared_state).await
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
#[derive(Debug, Clone, Eq, PartialEq)]
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
    dns_config: nym_firewall::AllowedDns,

    /// Tunnel interface
    tunnel_interface: TunnelInterface,

    /// Split tunnel redirect interface
    #[cfg(target_os = "macos")]
    redirect_interface: Option<String>,
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
                    #[cfg(target_os = "linux")]
                    // On Linux, All is needed so the mangle chain rule sets fwmark for outbound traffic
                    AllowedClients::All,
                    #[cfg(target_os = "macos")]
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
                    #[cfg(target_os = "linux")]
                    // On Linux, All is needed so the mangle chain rule sets fwmark for outbound traffic
                    AllowedClients::All,
                    #[cfg(target_os = "macos")]
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
            #[cfg(target_os = "macos")]
            redirect_interface: self.redirect_interface.clone(),
        }
    }
}

#[cfg(test)]
#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    fn create_mock_gateway_with_websocket_endpoints(
        ip: Ipv4Addr,
        ws_port: u16,
        wss_port: u16,
    ) -> nym_gateway_directory::Gateway {
        use nym_gateway_directory::Gateway;
        use nym_sdk::mixnet::NodeIdentity;

        // Create a dummy identity for testing
        let dummy_identity =
            NodeIdentity::from_base58_string("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42")
                .expect("Valid test identity");

        Gateway::builder()
            .identity(dummy_identity)
            .ips(vec![IpAddr::V4(ip)])
            .clients_ws_port(Some(ws_port))
            .clients_wss_port(Some(wss_port))
            .build()
    }

    #[test]
    fn test_firewall_policy_includes_exit_gateway_endpoints() {
        // Create mock entry gateway with WebSocket on port 9000 (WS) and 9001 (WSS)
        let entry_gateway =
            create_mock_gateway_with_websocket_endpoints(Ipv4Addr::new(192, 168, 1, 1), 9000, 9001);
        let entry_endpoints = entry_gateway.endpoints();

        // Create mock exit gateway with WebSocket on port 9000 (WS) and 9001 (WSS)
        let exit_gateway =
            create_mock_gateway_with_websocket_endpoints(Ipv4Addr::new(192, 168, 1, 2), 9000, 9001);
        let exit_endpoints = exit_gateway.endpoints();

        // Create ConnectedPolicyParameters (simulating what happens in enter())
        // We'll directly test with the endpoints without needing SelectedGateways
        let mut ws_endpoints = entry_endpoints.clone();
        ws_endpoints.extend(exit_endpoints.clone());

        // Create a minimal TunnelInterface for testing
        use crate::tunnel_state_machine::TunnelMetadata;
        use ipnetwork::IpNetwork;
        let tunnel_metadata = TunnelMetadata {
            interface: "test0".to_string(),
            ips: vec![
                IpNetwork::new(Ipv4Addr::new(10, 0, 0, 1).into(), 24)
                    .unwrap()
                    .network(),
            ],
            ipv4_gateway: Some(Ipv4Addr::new(10, 0, 0, 1)),
            ipv6_gateway: None,
        };
        let tunnel_interface = TunnelInterface::One(tunnel_metadata);

        let dns_config = AllowedDns::new(
            vec![
                Endpoint::new(
                    IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
                    53,
                    TransportProtocol::Tcp,
                ),
                Endpoint::new(
                    IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
                    53,
                    TransportProtocol::Udp,
                ),
            ],
            vec![],
        );

        let params = ConnectedPolicyParameters {
            enable_ipv6: false,
            allow_lan: false,
            wg_entry_endpoint: None,
            ws_entry_endpoints: ws_endpoints,
            dns_config,
            tunnel_interface,
            #[cfg(target_os = "macos")]
            redirect_interface: None,
        };

        // Build firewall policy
        let policy = params.as_policy();

        // Extract peer endpoints
        let peer_endpoints = policy.peer_endpoints();

        // Verify entry gateway endpoints are included
        assert!(
            entry_endpoints.iter().any(|entry_ep| {
                peer_endpoints.iter().any(|allowed_ep| {
                    allowed_ep.endpoint.address == *entry_ep
                        && allowed_ep.endpoint.protocol == TransportProtocol::Tcp
                })
            }),
            "Entry gateway endpoints should be in peer_endpoints"
        );

        // Verify exit gateway endpoints are included
        assert!(
            exit_endpoints.iter().any(|exit_ep| {
                peer_endpoints.iter().any(|allowed_ep| {
                    allowed_ep.endpoint.address == *exit_ep
                        && allowed_ep.endpoint.protocol == TransportProtocol::Tcp
                })
            }),
            "Exit gateway endpoints should be in peer_endpoints for SOCKS5 support"
        );

        // Verify we have endpoints from both gateways
        assert!(
            peer_endpoints.len() >= entry_endpoints.len() + exit_endpoints.len(),
            "peer_endpoints should contain endpoints from both entry and exit gateways"
        );
    }
}
