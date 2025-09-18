// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use std::net::SocketAddr;
use std::time::Duration;

use futures::{
    FutureExt,
    future::{BoxFuture, Fuse},
};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use nym_common::trace_err_chain;
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use nym_dns::DnsConfig;
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use nym_firewall::{
    AllowedClients, AllowedEndpoint, AllowedTunnelTraffic, Endpoint, FirewallPolicy,
    TransportProtocol,
};
use nym_gateway_directory::ResolvedConfig;
use nym_vpn_lib_types::{EstablishConnectionData, EstablishConnectionState, GatewayId};

#[cfg(target_os = "macos")]
use crate::tunnel_state_machine::resolver::LOCAL_DNS_RESOLVER;
use crate::tunnel_state_machine::{
    Error, ErrorStateReason, NextTunnelState, PrivateActionAfterDisconnect, PrivateTunnelState,
    Result, SharedState, TunnelCommand, TunnelInterface, TunnelStateHandler,
    states::{ConnectedState, DisconnectedState, DisconnectingState, ErrorState, OfflineState},
    tunnel::{SelectedGateways, Tombstone},
    tunnel_monitor::{
        TunnelMonitor, TunnelMonitorEvent, TunnelMonitorEventReceiver, TunnelMonitorEventSender,
        TunnelMonitorHandle, TunnelParameters,
    },
};

/// Initial delay between retry attempts.
const INITIAL_WAIT_DELAY: Duration = Duration::from_secs(2);

/// Wait delay multiplier used for each subsequent retry attempt.
const DELAY_MULTIPLIER: u32 = 2;

/// Max wait delay between retry attempts.
const MAX_WAIT_DELAY: Duration = Duration::from_secs(15);

/// Default websocket port used as a fallback
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
const DEFAULT_WS_PORT: u16 = 80;

type ResolveConfigFuture = BoxFuture<'static, Result<ResolvedConfig>>;
type ReconnectDelayFuture = BoxFuture<'static, ()>;

pub struct ConnectingState {
    retry_attempt: u32,
    tunnel_monitor_handle: Option<TunnelMonitorHandle>,
    tunnel_monitor_event_sender: Option<TunnelMonitorEventSender>,
    tunnel_monitor_event_receiver: TunnelMonitorEventReceiver,
    selected_gateways: Option<SelectedGateways>,
    connection_data: Option<EstablishConnectionData>,
    resolved_gateway_config: Option<ResolvedConfig>,
    resolve_config_fut: Fuse<ResolveConfigFuture>,
    reconnect_delay_fut: Fuse<ReconnectDelayFuture>,
}

impl ConnectingState {
    pub async fn enter(
        retry_attempt: u32,
        selected_gateways: Option<SelectedGateways>,
        shared_state: &mut SharedState,
    ) -> (Box<dyn TunnelStateHandler>, PrivateTunnelState) {
        #[cfg(target_os = "macos")]
        if let Err(e) = Self::set_local_dns_resolver(shared_state).await {
            trace_err_chain!(e, "Failed to configure system to use filtering resolver",);
            return ErrorState::enter(ErrorStateReason::SetDns, shared_state).await;
        }

        if shared_state
            .connectivity_handle
            .connectivity()
            .await
            .is_offline()
        {
            // FIXME: Temporary: Nudge route manager to update the default interface
            #[cfg(target_os = "macos")]
            {
                tracing::debug!("Poking route manager to update default routes");
                shared_state.route_handler.refresh_routes().await;
            }
            return OfflineState::enter(true, selected_gateways, shared_state).await;
        }

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        {
            let entry_gateway = selected_gateways.as_ref().map(|x| x.entry.as_ref());
            if let Err(e) =
                Self::set_firewall_policy(shared_state, None, entry_gateway, None, &[]).await
            {
                trace_err_chain!(e, "failed to set firewall policy");
                return ErrorState::enter(ErrorStateReason::SetFirewallPolicy, shared_state).await;
            }
        }

        // If that fails, it's not really important
        let _ = shared_state
            .account_command_tx
            .set_vpn_api_firewall_up()
            .await;

        let resolve_config_fut = Fuse::terminated();
        let reconnect_delay_fut = if retry_attempt > 0 {
            let wait_delay = wait_delay(retry_attempt);
            tracing::info!("Waiting {}s before reconnect", wait_delay.as_secs());
            tokio::time::sleep(wait_delay).boxed().fuse()
        } else {
            std::future::ready(()).boxed().fuse()
        };

        let (monitor_event_sender, monitor_event_receiver) = mpsc::unbounded_channel();

        let initial_connection_data =
            selected_gateways
                .as_ref()
                .map(|gateways| EstablishConnectionData {
                    entry_gateway: GatewayId::from(*gateways.entry.clone()),
                    exit_gateway: GatewayId::from(*gateways.exit.clone()),
                    tunnel: None,
                });

        let connecting_state = Self {
            tunnel_monitor_handle: None,
            tunnel_monitor_event_sender: Some(monitor_event_sender),
            tunnel_monitor_event_receiver: monitor_event_receiver,
            retry_attempt,
            selected_gateways,
            connection_data: initial_connection_data.clone(),
            resolved_gateway_config: None,
            resolve_config_fut,
            reconnect_delay_fut,
        };

        let tunnel_state = connecting_state.make_connecting_tunnel_state(
            shared_state,
            EstablishConnectionState::ResolvingApiAddresses,
        );

        (Box::new(connecting_state), tunnel_state)
    }

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    async fn set_firewall_policy(
        shared_state: &mut SharedState,
        tunnel: Option<TunnelInterface>,
        entry_gateway: Option<&nym_gateway_directory::Gateway>,
        wg_entry_endpoint: Option<SocketAddr>,
        resolved_gateway_addresses: &[SocketAddr],
    ) -> Result<()> {
        let mut peer_endpoints = entry_gateway
            .map(|entry_gateway| {
                let ws_port = entry_gateway
                    .clients_wss_port
                    .or(entry_gateway.clients_ws_port)
                    .unwrap_or(DEFAULT_WS_PORT);

                entry_gateway
                    .ips
                    .iter()
                    .filter(|ip| {
                        ip.is_ipv4() || (shared_state.tunnel_settings.enable_ipv6 && ip.is_ipv6())
                    })
                    .map(|ip| {
                        AllowedEndpoint::new(
                            Endpoint::new(*ip, ws_port, TransportProtocol::Tcp),
                            #[cfg(any(target_os = "linux", target_os = "macos"))]
                            AllowedClients::Root,
                            #[cfg(target_os = "windows")]
                            AllowedClients::current_exe(),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        if let Some(wg_entry_endpoint) = wg_entry_endpoint {
            let allowed_endpoint = AllowedEndpoint::new(
                Endpoint::from_socket_address(wg_entry_endpoint, TransportProtocol::Udp),
                #[cfg(any(target_os = "linux", target_os = "macos"))]
                AllowedClients::Root,
                #[cfg(target_os = "windows")]
                AllowedClients::current_exe(),
            );
            peer_endpoints.push(allowed_endpoint);
        }
        // Set non-tunnel DNS to allow api client to use those DNS servers.
        let dns_config =
            DnsConfig::from_addresses(&[], &shared_state.tunnel_settings.default_dns_ips())
                .resolve(
                    // pass empty because we already override the config with non-tunnel addresses.
                    &[],
                    #[cfg(target_os = "macos")]
                    53,
                );

        let allowed_endpoints = resolved_gateway_addresses
            .iter()
            .filter(|ip| ip.is_ipv4() || (shared_state.tunnel_settings.enable_ipv6 && ip.is_ipv6()))
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

        let policy = FirewallPolicy::Connecting {
            peer_endpoints,
            tunnel: tunnel.map(nym_firewall::TunnelInterface::from),
            // todo: fetch this from config
            allow_lan: true,
            dns_config,
            allowed_endpoints,
            // todo: only allow connection towards entry endpoint?
            allowed_entry_tunnel_traffic: AllowedTunnelTraffic::All,
            allowed_exit_tunnel_traffic: AllowedTunnelTraffic::All,
            // todo: split tunneling
            #[cfg(target_os = "macos")]
            redirect_interface: None,
        };

        shared_state
            .firewall
            .apply_policy(policy)
            .inspect_err(|error| {
                trace_err_chain!(
                    error,
                    "Failed to apply firewall policy for connecting state"
                );
            })
            .map_err(Error::SetFirewallPolicy)
    }

    #[cfg(target_os = "macos")]
    async fn set_local_dns_resolver(shared_state: &mut SharedState) -> Result<()> {
        if *LOCAL_DNS_RESOLVER {
            // Set system DNS to our local DNS resolver
            let system_dns = DnsConfig::default().resolve(
                &[shared_state.filtering_resolver.listen_addr().ip()],
                shared_state.filtering_resolver.listen_addr().port(),
            );
            shared_state
                .dns_handler
                .set("lo".to_owned(), system_dns)
                .await
                .map_err(Error::SetDns)
        } else {
            Ok(())
        }
    }

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    async fn reset_routes(shared_state: &mut SharedState) {
        shared_state.route_handler.remove_routes().await
    }

    async fn reconnect(self, shared_state: &mut SharedState) -> NextTunnelState {
        let next_attempt = self.retry_attempt.saturating_add(1);
        let next_gateways = if next_attempt.is_multiple_of(2) {
            None
        } else {
            self.selected_gateways
        };

        tracing::info!("Reconnecting, attempt {next_attempt}");

        NextTunnelState::NewState(
            ConnectingState::enter(next_attempt, next_gateways, shared_state).await,
        )
    }

    async fn disconnect(
        after_disconnect: PrivateActionAfterDisconnect,
        tunnel_monitor_handle: TunnelMonitorHandle,
        shared_state: &mut SharedState,
    ) -> NextTunnelState {
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        Self::reset_routes(shared_state).await;

        NextTunnelState::NewState(DisconnectingState::enter(
            after_disconnect,
            tunnel_monitor_handle,
            shared_state,
        ))
    }

    async fn handle_tunnel_close(tombstone: Tombstone, _shared_state: &mut SharedState) {
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        _shared_state.route_handler.remove_routes().await;

        // drop tombstone to close tunnel devices
        let _ = tombstone;
    }

    async fn handle_resolved_gateway_config(
        mut self: Box<Self>,
        resolver_result: Result<ResolvedConfig>,
        shared_state: &mut SharedState,
    ) -> NextTunnelState {
        let resolved_gateway_config = match resolver_result {
            Ok(resolved_gateway_config) => {
                tracing::info!("Resolved gateway config: {:?}", resolved_gateway_config);
                resolved_gateway_config
            }
            Err(e) => {
                trace_err_chain!(e, "Failed to resolve gateway config");
                return self.reconnect(shared_state).await;
            }
        };

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        {
            let entry_gateway = self.selected_gateways.as_ref().map(|x| x.entry.as_ref());

            if let Err(e) = Self::set_firewall_policy(
                shared_state,
                None,
                entry_gateway,
                None,
                &resolved_gateway_config.all_socket_addrs(),
            )
            .await
            {
                trace_err_chain!(e, "failed to set firewall policy");
                return NextTunnelState::NewState(
                    ErrorState::enter(ErrorStateReason::SetFirewallPolicy, shared_state).await,
                );
            }
        }

        if resolved_gateway_config.nym_vpn_api_socket_addrs.is_none()
            || resolved_gateway_config
                .nym_vpn_api_socket_addrs
                .as_ref()
                .is_some_and(|x| x.is_empty())
        {
            tracing::warn!(
                "nym_vpn_api_socket_addrs is empty which may result into firewall blocking the API requests."
            );
        } else if let Err(e) = shared_state
            .account_command_tx
            .set_static_api_addresses(resolved_gateway_config.nym_vpn_api_socket_addrs.to_owned())
            .await
        {
            trace_err_chain!(e, "Failed to set static API addresses");
            return NextTunnelState::NewState(
                ErrorState::enter(
                    ErrorStateReason::Internal(
                        "Failed to set static NYM API addresses to account controller".to_owned(),
                    ),
                    shared_state,
                )
                .await,
            );
        }

        let _ = shared_state
            .account_command_tx
            .set_vpn_api_firewall_down()
            .await;

        let Some(tunnel_monitor_event_sender) = self.tunnel_monitor_event_sender.take() else {
            return NextTunnelState::NewState(
                ErrorState::enter(
                    ErrorStateReason::Internal(
                        "Monitor event sender is not set. This is a logical error.".to_owned(),
                    ),
                    shared_state,
                )
                .await,
            );
        };

        // Make mixnet client use pre-fetched topology
        shared_state.topology_provider.use_network(false).await;

        let tunnel_parameters = TunnelParameters {
            nym_config: shared_state.nym_config.clone(),
            resolved_gateway_config: resolved_gateway_config.clone(),
            tunnel_settings: shared_state.tunnel_settings.clone(),
            tunnel_constants: shared_state.tunnel_constants,
            selected_gateways: self.selected_gateways.clone(),
        };
        let tunnel_monitor_handle = TunnelMonitor::start(
            tunnel_parameters,
            shared_state.account_controller_state.clone(),
            shared_state.gateway_cache_handle.clone(),
            shared_state.topology_provider.clone(),
            tunnel_monitor_event_sender,
            shared_state.mixnet_event_sender.clone(),
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            shared_state.route_handler.clone(),
            #[cfg(any(target_os = "ios", target_os = "android"))]
            shared_state.tun_provider.clone(),
        );

        self.tunnel_monitor_handle = Some(tunnel_monitor_handle);
        self.resolved_gateway_config = Some(resolved_gateway_config);

        NextTunnelState::SameState(self)
    }

    async fn handle_interface_up(
        &mut self,
        _tunnel_interface: TunnelInterface,
        connection_data: Box<EstablishConnectionData>,
        _shared_state: &mut SharedState,
    ) -> Result<()> {
        self.connection_data = Some(*connection_data);

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        {
            let resolved_addrs =
                if let Some(resolved_config) = self.resolved_gateway_config.as_ref() {
                    resolved_config.all_socket_addrs()
                } else {
                    tracing::warn!("Resolved gateway config is not set. This is a logical error!");
                    Vec::new()
                };

            Self::set_firewall_policy(
                _shared_state,
                Some(_tunnel_interface),
                self.selected_gateways.as_ref().map(|x| x.entry.as_ref()),
                None,
                &resolved_addrs,
            )
            .await?;
        }

        Ok(())
    }

    async fn handle_selected_gateways(
        &mut self,
        gateways: Box<SelectedGateways>,
        _shared_state: &mut SharedState,
    ) -> Result<()> {
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        let set_policy_result = {
            let resolved_addrs =
                if let Some(resolved_config) = self.resolved_gateway_config.as_ref() {
                    resolved_config.all_socket_addrs()
                } else {
                    tracing::warn!("Resolved gateway config is not set. This is a logical error!");
                    Vec::new()
                };

            Self::set_firewall_policy(
                _shared_state,
                None,
                Some(gateways.entry.as_ref()),
                None,
                &resolved_addrs,
            )
            .await
        };
        self.selected_gateways = Some(*gateways);

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        {
            set_policy_result
        }

        #[cfg(any(target_os = "ios", target_os = "android"))]
        Ok(())
    }

    fn make_connecting_tunnel_state(
        &self,
        shared_state: &SharedState,
        state: EstablishConnectionState,
    ) -> PrivateTunnelState {
        PrivateTunnelState::Connecting {
            retry_attempt: self.retry_attempt,
            state,
            tunnel_type: shared_state.tunnel_settings.tunnel_type,
            connection_data: self.connection_data.clone(),
        }
    }
}

#[async_trait::async_trait]
impl TunnelStateHandler for ConnectingState {
    async fn handle_event(
        mut self: Box<Self>,
        shutdown_token: &CancellationToken,
        command_rx: &'async_trait mut mpsc::UnboundedReceiver<TunnelCommand>,
        shared_state: &'async_trait mut SharedState,
    ) -> NextTunnelState {
        tokio::select! {
            _ = &mut self.reconnect_delay_fut => {
                let gateway_config = shared_state.nym_config.gateway_config.clone();

                self.resolve_config_fut = async move {
                    nym_gateway_directory::resolve_config(&gateway_config)
                        .await
                        .map_err(Error::ResolveApiHostnames)
                }
                .boxed()
                .fuse();

                NextTunnelState::SameState(self)
            },
            resolved_gateway_config = &mut self.resolve_config_fut => {
                self.handle_resolved_gateway_config(resolved_gateway_config, shared_state).await
            }
            Some(monitor_event) = self.tunnel_monitor_event_receiver.recv() => {
                match monitor_event {
                    TunnelMonitorEvent::AwaitingAccountReadiness => {
                        let new_state = self.make_connecting_tunnel_state(shared_state, EstablishConnectionState::AwaitingAccountReadiness);
                        NextTunnelState::NewState((self, new_state))
                    }
                    TunnelMonitorEvent::RefreshingGateways => {
                        let new_state = self.make_connecting_tunnel_state(shared_state, EstablishConnectionState::RefreshingGateways);
                        NextTunnelState::NewState((self, new_state))
                    }
                    TunnelMonitorEvent::ConnectingMixnetClient => {
                        let new_state = self.make_connecting_tunnel_state(shared_state, EstablishConnectionState::ConnectingMixnetClient);
                        NextTunnelState::NewState((self, new_state))
                    }
                    TunnelMonitorEvent::SelectingGateways => {
                        let new_state = self.make_connecting_tunnel_state(shared_state, EstablishConnectionState::SelectingGateways);
                        NextTunnelState::NewState((self, new_state))
                    }
                    TunnelMonitorEvent::SelectedGateways {
                        gateways, reply_tx
                    } => {
                    let next_state = match self.handle_selected_gateways(gateways, shared_state).await {
                            Ok(()) => {
                                NextTunnelState::SameState(self)
                            }
                            Err(e) => {
                                trace_err_chain!(e, "Failed to set firewall policy");
                                if let Some(tunnel_monitor_handle) = self.tunnel_monitor_handle {
                                    NextTunnelState::NewState(DisconnectingState::enter(
                                        PrivateActionAfterDisconnect::Error(ErrorStateReason::SetFirewallPolicy),
                                        tunnel_monitor_handle,
                                        shared_state
                                    ))
                                } else {
                                    NextTunnelState::NewState(ErrorState::enter(ErrorStateReason::SetFirewallPolicy, shared_state).await)
                                }
                            }
                        };
                        _ = reply_tx.send(());
                        next_state
                    }
                    TunnelMonitorEvent::InterfaceUp {
                        tunnel_interface, connection_data, reply_tx
                    }  => {
                        let next_state = match self.handle_interface_up(tunnel_interface, connection_data, shared_state).await {
                            Ok(()) => {
                                let state = self.make_connecting_tunnel_state(shared_state, EstablishConnectionState::ConnectingTunnel);
                                NextTunnelState::NewState((self, state))
                            },
                            Err(e) => {
                                trace_err_chain!(e, "Failed to set firewall policy");
                                if let Some(tunnel_monitor_handle) = self.tunnel_monitor_handle {
                                    NextTunnelState::NewState(DisconnectingState::enter(
                                        PrivateActionAfterDisconnect::Error(ErrorStateReason::SetFirewallPolicy),
                                        tunnel_monitor_handle,
                                        shared_state
                                    ))
                                } else {
                                    NextTunnelState::NewState(ErrorState::enter(ErrorStateReason::SetFirewallPolicy, shared_state).await)
                                }
                            }
                        };
                        _ = reply_tx.send(());
                        next_state
                    }
                    TunnelMonitorEvent::Up { tunnel_interface, connection_data } => {
                        NextTunnelState::NewState(ConnectedState::enter(
                            tunnel_interface,
                            *connection_data,
                            self.selected_gateways.expect("selected gateways must be set"),
                            self.tunnel_monitor_handle.expect("monitor handle must be set!"),
                            self.tunnel_monitor_event_receiver,
                            shared_state,
                        ).await)
                    }
                    TunnelMonitorEvent::Down { error_state_reason, reply_tx } => {
                        // Signal that the message was received first.
                        _ = reply_tx.send(());

                        if let Some(error_state_reason) = error_state_reason {
                            NextTunnelState::NewState(DisconnectingState::enter(
                                PrivateActionAfterDisconnect::Error(error_state_reason),
                                self.tunnel_monitor_handle.expect("monitor handle must be set!"),
                                shared_state
                            ))
                        } else {
                            if let Some(tunnel_monitor_handle) = self.tunnel_monitor_handle.take() {
                                let tombstone = tunnel_monitor_handle.wait().await;
                                Self::handle_tunnel_close(tombstone, shared_state).await;
                            }

                            tracing::info!("Tunnel closed");

                            self.reconnect(shared_state).await
                        }
                    }
                }
           }
            Some(command) = command_rx.recv() => {
                match command {
                    TunnelCommand::Connect => {
                        if let Some(tunnel_monitor_handle) = self.tunnel_monitor_handle {
                            Self::disconnect(PrivateActionAfterDisconnect::Reconnect, tunnel_monitor_handle, shared_state).await
                        } else {
                            NextTunnelState::NewState(ConnectingState::enter(self.retry_attempt, None, shared_state).await)
                        }
                    },
                    TunnelCommand::Disconnect => {
                        if let Some(tunnel_monitor_handle) = self.tunnel_monitor_handle {
                            Self::disconnect(PrivateActionAfterDisconnect::Nothing, tunnel_monitor_handle, shared_state).await
                        } else {
                            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
                            Self::reset_routes(shared_state).await;
                            NextTunnelState::NewState(DisconnectedState::enter(None, shared_state).await)
                        }
                    },
                    TunnelCommand::SetTunnelSettings(tunnel_settings) => {
                        if shared_state.tunnel_settings == tunnel_settings {
                            NextTunnelState::SameState(self)
                        } else {
                            let gateways_changed = shared_state.tunnel_settings.entry_point != tunnel_settings.entry_point ||
                                shared_state.tunnel_settings.exit_point != tunnel_settings.exit_point;

                            shared_state.tunnel_settings = tunnel_settings;

                            if let Some(tunnel_monitor_handle) = self.tunnel_monitor_handle {
                                Self::disconnect(PrivateActionAfterDisconnect::Reconnect, tunnel_monitor_handle, shared_state).await
                            } else {
                                let next_gateways = if gateways_changed {
                                    None
                                } else {
                                    self.selected_gateways
                                };
                                NextTunnelState::NewState(ConnectingState::enter(self.retry_attempt, next_gateways, shared_state).await)
                            }
                        }
                    }
                }
            }
            Some(connectivity) = shared_state.connectivity_handle.next() => {
                if connectivity.is_offline() {
                    if let Some(tunnel_monitor_handle) = self.tunnel_monitor_handle {
                        Self::disconnect(PrivateActionAfterDisconnect::Offline {
                            reconnect: true,
                            gateways: self.selected_gateways
                        }, tunnel_monitor_handle, shared_state).await
                    } else {
                        NextTunnelState::NewState(OfflineState::enter(true, self.selected_gateways, shared_state).await)
                    }
                } else {
                    NextTunnelState::SameState(self)
                }
            }
            _ = shutdown_token.cancelled() => {
                if let Some(tunnel_monitor_handle) = self.tunnel_monitor_handle {
                    Self::disconnect(PrivateActionAfterDisconnect::Nothing, tunnel_monitor_handle, shared_state).await
                } else {
                    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
                    Self::reset_routes(shared_state).await;
                    NextTunnelState::NewState(DisconnectedState::enter(None, shared_state).await)
                }
            }
        }
    }
}

fn wait_delay(retry_attempt: u32) -> Duration {
    let multiplier = retry_attempt.saturating_mul(DELAY_MULTIPLIER);
    let delay = INITIAL_WAIT_DELAY.saturating_mul(multiplier);
    std::cmp::min(delay, MAX_WAIT_DELAY)
}
