// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::ErrorStateReason;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[cfg(target_os = "macos")]
use nym_dns::DnsConfig;
#[cfg(target_os = "macos")]
use nym_firewall::LOCAL_DNS_RESOLVER;
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use nym_firewall::{AllowedClients, AllowedEndpoint, Endpoint, FirewallPolicy, TransportProtocol};
use nym_gateway_directory::ResolvedConfig;
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use nym_vpn_lib_types::TunnelConnectionData;

use crate::tunnel_state_machine::{
    states::{ConnectingState, DisconnectingState},
    tunnel::SelectedGateways,
    tunnel_monitor::{TunnelMonitorEvent, TunnelMonitorEventReceiver, TunnelMonitorHandle},
    ConnectionData, NextTunnelState, PrivateActionAfterDisconnect, PrivateTunnelState, SharedState,
    TunnelCommand, TunnelInterface, TunnelStateHandler,
};
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use crate::tunnel_state_machine::{Error, Result};

use super::ErrorState;

/// Default websocket port used as a fallback
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
const DEFAULT_WS_PORT: u16 = 80;

pub struct ConnectedState {
    tunnel_monitor_handle: TunnelMonitorHandle,
    tunnel_monitor_event_receiver: TunnelMonitorEventReceiver,
    selected_gateways: SelectedGateways,
    #[cfg_attr(any(target_os = "android", target_os = "ios"), allow(unused))]
    tunnel_interface: TunnelInterface,
    #[cfg_attr(any(target_os = "android", target_os = "ios"), allow(unused))]
    resolved_gateway_config: ResolvedConfig,
}

impl ConnectedState {
    pub async fn enter(
        tunnel_interface: TunnelInterface,
        connection_data: ConnectionData,
        selected_gateways: SelectedGateways,
        resolved_gateway_config: ResolvedConfig,
        tunnel_monitor_handle: TunnelMonitorHandle,
        tunnel_monitor_event_receiver: TunnelMonitorEventReceiver,
        _shared_state: &mut SharedState,
    ) -> (Box<dyn TunnelStateHandler>, PrivateTunnelState) {
        let connected_state = Self {
            tunnel_monitor_handle,
            tunnel_monitor_event_receiver,
            selected_gateways,
            tunnel_interface,
            resolved_gateway_config,
        };

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        if let Err(e) = connected_state
            .set_firewall_policy(_shared_state, &connection_data)
            .await
        {
            return DisconnectingState::enter(
                PrivateActionAfterDisconnect::Error(
                    e.error_state_reason()
                        .expect("failed to obtain error state reason"),
                ),
                connected_state.tunnel_monitor_handle,
                _shared_state,
            );
        } else if let Err(e) = connected_state.set_dns(_shared_state).await {
            return DisconnectingState::enter(
                PrivateActionAfterDisconnect::Error(
                    e.error_state_reason()
                        .expect("failed to obtain error state reason"),
                ),
                connected_state.tunnel_monitor_handle,
                _shared_state,
            );
        }

        (
            Box::new(connected_state),
            PrivateTunnelState::Connected { connection_data },
        )
    }

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    async fn set_firewall_policy(
        &self,
        shared_state: &mut SharedState,
        connection_data: &ConnectionData,
    ) -> Result<()> {
        let wg_entry_endpoint = match connection_data.tunnel {
            TunnelConnectionData::Wireguard(ref wireguard_data) => {
                Some(wireguard_data.entry.endpoint)
            }
            TunnelConnectionData::Mixnet(_) => None,
        };

        let ws_port = self
            .selected_gateways
            .entry
            .clients_wss_port
            .or(self.selected_gateways.entry.clients_ws_port)
            .unwrap_or(DEFAULT_WS_PORT);

        let mut peer_endpoints = self
            .selected_gateways
            .entry
            .ips
            .iter()
            .map(|ip| {
                AllowedEndpoint::new(
                    Endpoint::new(*ip, ws_port, TransportProtocol::Tcp),
                    #[cfg(any(target_os = "linux", target_os = "macos"))]
                    AllowedClients::Root,
                    #[cfg(target_os = "windows")]
                    AllowedClients::current_exe(),
                )
            })
            .collect::<Vec<_>>();

        if let Some(wg_peer_endpoint) = wg_entry_endpoint {
            let allowed_endpoint = AllowedEndpoint::new(
                Endpoint::from_socket_address(wg_peer_endpoint, TransportProtocol::Udp),
                #[cfg(any(target_os = "linux", target_os = "macos"))]
                AllowedClients::Root,
                #[cfg(target_os = "windows")]
                AllowedClients::current_exe(),
            );
            peer_endpoints.push(allowed_endpoint);
        }

        let dns_config = shared_state.tunnel_settings.dns.to_dns_config().resolve(
            &crate::DEFAULT_DNS_SERVERS,
            #[cfg(target_os = "macos")]
            53,
        );

        let allowed_endpoints = self
            .resolved_gateway_config
            .all_socket_addrs()
            .into_iter()
            .map(|addr| {
                AllowedEndpoint::new(
                    Endpoint::from_socket_address(addr, TransportProtocol::Tcp),
                    #[cfg(any(target_os = "linux", target_os = "macos"))]
                    AllowedClients::Root,
                    #[cfg(target_os = "windows")]
                    AllowedClients::current_exe(),
                )
            })
            .collect();

        let policy = FirewallPolicy::Connected {
            peer_endpoints,
            tunnel: nym_firewall::TunnelInterface::from(self.tunnel_interface.clone()),
            // todo: fetch this from config
            allow_lan: true,
            allowed_endpoints,
            dns_config,
            // todo: split tunneling
            #[cfg(target_os = "macos")]
            redirect_interface: None,
            #[cfg(target_os = "macos")]
            dns_redirect_port: shared_state.filtering_resolver.listening_port(),
        };

        shared_state
            .firewall
            .apply_policy(policy)
            .map_err(Error::CreateFirewall)
    }

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    async fn set_dns(&self, shared_state: &mut SharedState) -> Result<()> {
        let dns_config = shared_state.tunnel_settings.dns.to_dns_config().resolve(
            &crate::DEFAULT_DNS_SERVERS,
            #[cfg(target_os = "macos")]
            53,
        );

        let tunnel_metadata = match &self.tunnel_interface {
            TunnelInterface::One(interface) => interface,
            TunnelInterface::Two { exit, .. } => exit,
        };

        #[cfg(any(target_os = "linux", target_os = "windows"))]
        shared_state
            .dns_handler
            .set(tunnel_metadata.interface.clone(), dns_config)
            .await
            .map_err(Error::SetDns)?;

        // On macOS, configure only the local DNS resolver
        #[cfg(target_os = "macos")]
        // We do not want to forward DNS queries to *our* local resolver if we do not run a local
        // DNS resolver *or* if the DNS config points to a loopback address.
        if dns_config.is_loopback() || !*LOCAL_DNS_RESOLVER {
            log::debug!("Not enabling local DNS resolver");
            shared_state
                .dns_handler
                .set(tunnel_metadata.interface.clone(), dns_config)
                .await
                .map_err(Error::SetDns)?;
        } else {
            log::debug!("Enabling local DNS resolver");
            // Tell local DNS resolver to start forwarding DNS queries to whatever `dns_config`
            // specifies as DNS.
            shared_state
                .filtering_resolver
                .enable_forward(dns_config.addresses().collect())
                .await;

            // Set system DNS to our local DNS resolver
            let system_dns = DnsConfig::default().resolve(
                &[std::net::Ipv4Addr::LOCALHOST.into()],
                shared_state.filtering_resolver.listening_port(),
            );
            shared_state
                .dns_handler
                .set("lo".to_owned(), system_dns)
                .await
                .map_err(Error::SetDns)?;
        }

        Ok(())
    }

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    async fn reset_dns(shared_state: &mut SharedState) {
        #[cfg(any(target_os = "linux", target_os = "windows"))]
        if let Err(error) = shared_state
            .dns_handler
            .reset_before_interface_removal()
            .await
        {
            error.trace_chain_with_msg("Failed to reset DNS");
        }

        // On macOS, configure only the local DNS resolver
        #[cfg(target_os = "macos")]
        shared_state.filtering_resolver.disable_forward().await;
    }

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    async fn reset_routes(shared_state: &mut SharedState) {
        shared_state.route_handler.remove_routes().await
    }

    async fn disconnect(
        self,
        after_disconnect: PrivateActionAfterDisconnect,
        shared_state: &mut SharedState,
    ) -> NextTunnelState {
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        {
            Self::reset_dns(shared_state).await;
            Self::reset_routes(shared_state).await;
        }

        NextTunnelState::NewState(DisconnectingState::enter(
            after_disconnect,
            self.tunnel_monitor_handle,
            shared_state,
        ))
    }

    async fn handle_tunnel_down(
        self,
        error_state_reason: Option<ErrorStateReason>,
        shared_state: &mut SharedState,
    ) -> NextTunnelState {
        if error_state_reason.is_none() {
            tracing::info!("Tunnel closed. Reconnecting.");
        }

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
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
                match command {
                    TunnelCommand::Connect => {
                        self.disconnect(PrivateActionAfterDisconnect::Reconnect, shared_state).await
                    },
                    TunnelCommand::Disconnect => {
                        self.disconnect(PrivateActionAfterDisconnect::Nothing, shared_state).await
                    },
                    TunnelCommand::SetTunnelSettings(tunnel_settings) => {
                        if shared_state.tunnel_settings == tunnel_settings {
                            NextTunnelState::SameState(self)
                        } else {
                            shared_state.tunnel_settings = tunnel_settings;
                            self.disconnect(PrivateActionAfterDisconnect::Reconnect, shared_state).await
                        }
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
