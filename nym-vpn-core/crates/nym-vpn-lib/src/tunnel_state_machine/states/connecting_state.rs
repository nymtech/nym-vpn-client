// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use std::net::SocketAddr;

use futures::{
    future::{BoxFuture, Fuse},
    FutureExt,
};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use nym_common::ErrorExt;
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use nym_dns::DnsConfig;
#[cfg(target_os = "macos")]
use nym_firewall::LOCAL_DNS_RESOLVER;
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use nym_firewall::{
    AllowedClients, AllowedEndpoint, AllowedTunnelTraffic, Endpoint, FirewallPolicy,
    TransportProtocol,
};
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use nym_gateway_directory::Gateway;
use nym_gateway_directory::ResolvedConfig;

use crate::tunnel_state_machine::{
    states::{ConnectedState, DisconnectedState, DisconnectingState, ErrorState, OfflineState},
    tunnel::{SelectedGateways, Tombstone},
    tunnel_monitor::{
        TunnelMonitor, TunnelMonitorEvent, TunnelMonitorEventReceiver, TunnelMonitorEventSender,
        TunnelMonitorHandle, TunnelParameters,
    },
    Error, ErrorStateReason, NextTunnelState, PrivateActionAfterDisconnect, PrivateTunnelState,
    Result, SharedState, TunnelCommand, TunnelInterface, TunnelStateHandler,
};

/// Default websocket port used as a fallback
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
const DEFAULT_WS_PORT: u16 = 80;

type ResolveConfigFuture = BoxFuture<'static, Result<ResolvedConfig>>;

pub struct ConnectingState {
    retry_attempt: u32,
    monitor_handle: Option<TunnelMonitorHandle>,
    monitor_event_sender: Option<TunnelMonitorEventSender>,
    monitor_event_receiver: TunnelMonitorEventReceiver,
    selected_gateways: Option<SelectedGateways>,
    resolved_gateway_config: Option<ResolvedConfig>,
    resolve_config_fut: Fuse<ResolveConfigFuture>,
}

impl ConnectingState {
    pub async fn enter(
        retry_attempt: u32,
        selected_gateways: Option<SelectedGateways>,
        shared_state: &mut SharedState,
    ) -> (Box<dyn TunnelStateHandler>, PrivateTunnelState) {
        #[cfg(target_os = "macos")]
        if let Err(e) = Self::set_local_dns_resolver(shared_state).await {
            return ErrorState::enter(
                e.error_state_reason()
                    .expect("failed to map to error state reason"),
                shared_state,
            )
            .await;
        }

        if shared_state
            .offline_monitor
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
            return OfflineState::enter(true, retry_attempt, selected_gateways, shared_state).await;
        }

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        {
            let entry_gateway = selected_gateways.as_ref().map(|x| x.entry.as_ref());
            if let Err(e) =
                Self::set_firewall_policy(shared_state, None, entry_gateway, None, &[]).await
            {
                return ErrorState::enter(
                    e.error_state_reason()
                        .expect("failed to map to error state reason"),
                    shared_state,
                )
                .await;
            }
        }

        let gateway_config = shared_state.nym_config.gateway_config.clone();
        let resolve_config_fut = async move {
            nym_gateway_directory::resolve_config(&gateway_config)
                .await
                .map_err(Error::ResolveGatewayAddrs)
        }
        .boxed()
        .fuse();

        let (monitor_event_sender, monitor_event_receiver) = mpsc::unbounded_channel();

        (
            Box::new(Self {
                monitor_handle: None,
                monitor_event_sender: Some(monitor_event_sender),
                monitor_event_receiver,
                retry_attempt,
                selected_gateways,
                resolved_gateway_config: None,
                resolve_config_fut,
            }),
            PrivateTunnelState::Connecting {
                connection_data: None,
            },
        )
    }

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    async fn set_firewall_policy(
        shared_state: &mut SharedState,
        tunnel: Option<TunnelInterface>,
        entry_gateway: Option<&Gateway>,
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
        let dns_config = DnsConfig::from_addresses(&[], &crate::DEFAULT_DNS_SERVERS).resolve(
            // pass empty because we already override the config with non-tunnel addresses.
            &[],
            #[cfg(target_os = "macos")]
            53,
        );

        let allowed_endpoints = resolved_gateway_addresses
            .iter()
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
            #[cfg(target_os = "macos")]
            dns_redirect_port: shared_state.filtering_resolver.listening_port(),
        };

        shared_state
            .firewall
            .apply_policy(policy)
            .inspect_err(|error| {
                tracing::error!(
                    "{}",
                    error.display_chain_with_msg(
                        "Failed to apply firewall policy for connecting state"
                    )
                );
            })
            .map_err(Error::ApplyFirewallPolicy)
    }

    #[cfg(target_os = "macos")]
    async fn set_local_dns_resolver(shared_state: &mut SharedState) -> Result<()> {
        #[cfg(target_os = "macos")]
        if *LOCAL_DNS_RESOLVER {
            // Set system DNS to our local DNS resolver
            let system_dns = DnsConfig::default().resolve(
                &[std::net::Ipv4Addr::LOCALHOST.into()],
                shared_state.filtering_resolver.listening_port(),
            );
            let _ = shared_state
                .dns_handler
                .set("lo".to_owned(), system_dns)
                .await
                .inspect_err(|err| {
                    tracing::error!(
                        "{}",
                        err.display_chain_with_msg(
                            "Failed to configure system to use filtering resolver"
                        )
                    );
                });
        }

        Ok(())
    }

    async fn handle_tunnel_close(mut tombstone: Tombstone, _shared_state: &mut SharedState) {
        #[cfg(any(target_os = "linux", target_os = "windows"))]
        {
            if let Err(e) = _shared_state
                .dns_handler
                .reset_before_interface_removal()
                .await
            {
                tracing::error!("Failed to reset dns before interface removal: {}", e);
            }
        }

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        _shared_state.route_handler.remove_routes().await;

        #[cfg(windows)]
        tombstone.wg_instances.clear();
        tombstone.tun_devices.clear();
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
                return NextTunnelState::NewState(
                    ErrorState::enter(
                        e.error_state_reason()
                            .expect("failed to map to error reason"),
                        shared_state,
                    )
                    .await,
                );
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
                return NextTunnelState::NewState(
                    ErrorState::enter(
                        e.error_state_reason()
                            .expect("failed to map to error state reason"),
                        shared_state,
                    )
                    .await,
                );
            }
        }

        if resolved_gateway_config.nym_vpn_api_socket_addrs.is_none()
            || resolved_gateway_config
                .nym_vpn_api_socket_addrs
                .as_ref()
                .is_some_and(|x| x.is_empty())
        {
            tracing::warn!("nym_vpn_api_socket_addrs is empty which may result into firewall blocking the API requests.");
        } else if let Err(e) = shared_state
            .account_command_tx
            .set_static_api_addresses(resolved_gateway_config.nym_vpn_api_socket_addrs.to_owned())
            .await
        {
            tracing::error!("Failed to set static API addresses: {}", e);
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

        let Some(monitor_event_sender) = self.monitor_event_sender.take() else {
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

        let tunnel_parameters = TunnelParameters {
            nym_config: shared_state.nym_config.clone(),
            resolved_gateway_config: resolved_gateway_config.clone(),
            tunnel_settings: shared_state.tunnel_settings.clone(),
            selected_gateways: self.selected_gateways.clone(),
            retry_attempt: self.retry_attempt,
        };
        let monitor_handle = TunnelMonitor::start(
            tunnel_parameters,
            shared_state.account_command_tx.clone(),
            monitor_event_sender,
            shared_state.mixnet_event_sender.clone(),
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            shared_state.route_handler.clone(),
            #[cfg(any(target_os = "ios", target_os = "android"))]
            shared_state.tun_provider.clone(),
        );

        self.monitor_handle = Some(monitor_handle);
        self.resolved_gateway_config = Some(resolved_gateway_config);

        NextTunnelState::SameState(self)
    }

    async fn handle_interface_up(
        &mut self,
        _tunnel_interface: TunnelInterface,
        _shared_state: &mut SharedState,
    ) -> Result<()> {
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
            resolved_gateway_config = &mut self.resolve_config_fut => {
                self.handle_resolved_gateway_config(resolved_gateway_config, shared_state).await
            }
            Some(monitor_event) = self.monitor_event_receiver.recv() => {
            match monitor_event {
                TunnelMonitorEvent::InitializingClient => {
                    NextTunnelState::SameState(self)
                }
                TunnelMonitorEvent::SyncingAccount => {
                    NextTunnelState::SameState(self)
                }
                TunnelMonitorEvent::RegisteringDevice => {
                    NextTunnelState::SameState(self)
                }
                TunnelMonitorEvent::RequestingZkNyms => {
                    NextTunnelState::SameState(self)
                }
                TunnelMonitorEvent::SelectingGateways => {
                    NextTunnelState::SameState(self)
                }
                TunnelMonitorEvent::SelectedGateways {
                    gateways, reply_tx
                } => {
                   let next_state = match self.handle_selected_gateways(gateways, shared_state).await {
                        Ok(()) => {
                            NextTunnelState::SameState(self)
                        }
                        Err(e) => {
                            if let Some(monitor_handle) = self.monitor_handle {
                                NextTunnelState::NewState(DisconnectingState::enter(
                                    // todo: fix that expect()
                                    PrivateActionAfterDisconnect::Error(e.error_state_reason().expect("failed to obtain error state reason")),
                                    monitor_handle,
                                    shared_state
                                ))
                            } else {
                                NextTunnelState::NewState(ErrorState::enter(
                                    // todo: fix that expect()
                                    e.error_state_reason().expect("failed to obtain error state reason"),
                                    shared_state
                                ).await)
                            }
                        }
                    };
                    _ = reply_tx.send(());
                    next_state
                }
                TunnelMonitorEvent::InterfaceUp {
                    tunnel_interface, connection_data, reply_tx
                }  => {
                    let next_state = match self.handle_interface_up(tunnel_interface, shared_state).await {
                        Ok(()) => {
                            NextTunnelState::NewState((self, PrivateTunnelState::Connecting { connection_data: Some(*connection_data) }))
                        },
                        Err(e) => {
                            if let Some(monitor_handle) = self.monitor_handle {
                                NextTunnelState::NewState(DisconnectingState::enter(
                                    // todo: fix that expect()
                                    PrivateActionAfterDisconnect::Error(e.error_state_reason().expect("failed to obtain error state reason")),
                                    monitor_handle,
                                    shared_state
                                ))
                            } else {
                                NextTunnelState::NewState(ErrorState::enter(
                                    // todo: fix that expect()
                                    e.error_state_reason().expect("failed to obtain error state reason"),
                                    shared_state
                                ).await)
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
                        self.resolved_gateway_config.expect("resolved gateway config must be set!"),
                        self.monitor_handle.expect("monitor handle must be set!"),
                        self.monitor_event_receiver,
                        shared_state,
                    ).await)
                }
                TunnelMonitorEvent::Down { error_state_reason, reply_tx } => {
                    // Signal that the message was received first.
                    _ = reply_tx.send(());

                    if let Some(error_state_reason) = error_state_reason {
                        NextTunnelState::NewState(DisconnectingState::enter(
                            PrivateActionAfterDisconnect::Error(error_state_reason),
                            self.monitor_handle.expect("monitor handle must be set!"),
                            shared_state
                        ))
                    } else {
                        if let Some(monitor_handle) = self.monitor_handle {
                            let tombstone = monitor_handle.wait().await;
                            Self::handle_tunnel_close(tombstone, shared_state).await;
                        }

                        let next_attempt = self.retry_attempt.saturating_add(1);
                        tracing::info!(
                            "Tunnel closed. Reconnecting, attempt {}.",
                            next_attempt
                        );
                        NextTunnelState::NewState(ConnectingState::enter(
                            next_attempt,
                            self.selected_gateways,
                            shared_state
                        ).await)
                    }
                }
            }
           }
            Some(command) = command_rx.recv() => {
                match command {
                    TunnelCommand::Connect => NextTunnelState::SameState(self),
                    TunnelCommand::Disconnect => {
                        if let Some(monitor_handle ) = self.monitor_handle {
                            NextTunnelState::NewState(DisconnectingState::enter(
                                PrivateActionAfterDisconnect::Nothing,
                                monitor_handle,
                                shared_state,
                            ))
                        } else {
                            NextTunnelState::NewState(DisconnectedState::enter(shared_state).await)
                        }
                    },
                    TunnelCommand::SetTunnelSettings(tunnel_settings) => {
                        if shared_state.tunnel_settings == tunnel_settings {
                            NextTunnelState::SameState(self)
                        } else {
                            shared_state.tunnel_settings = tunnel_settings;

                            if let Some(monitor_handle) = self.monitor_handle {
                                NextTunnelState::NewState(DisconnectingState::enter(
                                    PrivateActionAfterDisconnect::Reconnect { retry_attempt: 0 },
                                     monitor_handle,
                                    shared_state,
                                ))
                            } else {
                                NextTunnelState::NewState(ConnectingState::enter(self.retry_attempt, self.selected_gateways, shared_state).await)
                            }
                        }
                    }
                }
            }
            Some(connectivity) = shared_state.offline_monitor.next() => {
                if connectivity.is_offline() {
                    if let Some(monitor_handle) = self.monitor_handle {
                        NextTunnelState::NewState(DisconnectingState::enter(
                            PrivateActionAfterDisconnect::Offline {
                                reconnect: true,
                                retry_attempt: self.retry_attempt,
                                gateways: self.selected_gateways
                            },
                            monitor_handle,
                            shared_state
                        ))
                    } else {
                        NextTunnelState::NewState(OfflineState::enter(true, self.retry_attempt, self.selected_gateways, shared_state).await)
                    }
                } else {
                    NextTunnelState::SameState(self)
                }
            }
            _ = shutdown_token.cancelled() => {
                if let Some(monitor_handle) = self.monitor_handle {
                    NextTunnelState::NewState(DisconnectingState::enter(
                        PrivateActionAfterDisconnect::Nothing,
                        monitor_handle,
                        shared_state,
                    ))
                } else {
                    NextTunnelState::NewState(DisconnectedState::enter(shared_state).await)
                }
            }
        }
    }
}
