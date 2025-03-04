// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

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
    states::DisconnectingState,
    tunnel::SelectedGateways,
    tunnel_monitor::{TunnelMonitorEvent, TunnelMonitorEventReceiver, TunnelMonitorHandle},
    ConnectionData, NextTunnelState, PrivateActionAfterDisconnect, PrivateTunnelState, SharedState,
    TunnelCommand, TunnelInterface, TunnelStateHandler,
};
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use crate::tunnel_state_machine::{Error, Result};

/// Default websocket port used as a fallback
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
const DEFAULT_WS_PORT: u16 = 80;

pub struct ConnectedState {
    monitor_handle: TunnelMonitorHandle,
    monitor_event_receiver: TunnelMonitorEventReceiver,
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
        monitor_handle: TunnelMonitorHandle,
        monitor_event_receiver: TunnelMonitorEventReceiver,
        _shared_state: &mut SharedState,
    ) -> (Box<dyn TunnelStateHandler>, PrivateTunnelState) {
        let connected_state = Self {
            monitor_handle,
            monitor_event_receiver,
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
                connected_state.monitor_handle,
                _shared_state,
            );
        } else if let Err(e) = connected_state.set_dns(_shared_state).await {
            return DisconnectingState::enter(
                PrivateActionAfterDisconnect::Error(
                    e.error_state_reason()
                        .expect("failed to obtain error state reason"),
                ),
                connected_state.monitor_handle,
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

        #[cfg(any(target_os = "linux", target_os = "windows"))]
        shared_state
            .dns_handler
            .set(tunnel_metadata.interface.clone(), dns_config)
            .await
            .map_err(Error::SetDns)?;

        Ok(())
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
                    TunnelCommand::Connect => NextTunnelState::SameState(self),
                    TunnelCommand::Disconnect => {
                        NextTunnelState::NewState(DisconnectingState::enter(
                            PrivateActionAfterDisconnect::Nothing,
                            self.monitor_handle,
                            shared_state
                        ))
                    },
                    TunnelCommand::SetTunnelSettings(tunnel_settings) => {
                        if shared_state.tunnel_settings == tunnel_settings {
                            NextTunnelState::SameState(self)
                        } else {
                            shared_state.tunnel_settings = tunnel_settings;
                            NextTunnelState::NewState(DisconnectingState::enter(
                                PrivateActionAfterDisconnect::Reconnect { retry_attempt: 0 },
                                self.monitor_handle,
                                shared_state
                            ))
                        }
                    }
                }
            }
            Some(monitor_event) = self.monitor_event_receiver.recv() => {
                match monitor_event {
                    TunnelMonitorEvent::Down { error_state_reason, reply_tx } => {
                        let after_disconnect = error_state_reason.map(PrivateActionAfterDisconnect::Error)
                            .unwrap_or(PrivateActionAfterDisconnect::Reconnect { retry_attempt: 0 });
                        _ = reply_tx.send(());

                        NextTunnelState::NewState(DisconnectingState::enter(after_disconnect, self.monitor_handle, shared_state))
                    }
                    _ => {
                        NextTunnelState::SameState(self)
                    }
                }
            }
            Some(connectivity) = shared_state.offline_monitor.next() => {
                if connectivity.is_offline() {
                    NextTunnelState::NewState(DisconnectingState::enter(
                        PrivateActionAfterDisconnect::Offline {
                            reconnect: true,
                            retry_attempt: 0,
                            gateways: Some(self.selected_gateways)
                        },
                        self.monitor_handle,
                        shared_state
                    ))
                } else {
                    NextTunnelState::SameState(self)
                }
            }
            _ = shutdown_token.cancelled() => {
                NextTunnelState::NewState(DisconnectingState::enter(
                    PrivateActionAfterDisconnect::Nothing,
                    self.monitor_handle,
                    shared_state
                ))
            }
        }
    }
}
