// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use std::net::IpAddr;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use std::net::SocketAddr;
use std::time::Duration;

use futures::{
    FutureExt,
    future::{BoxFuture, Fuse},
};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use nym_common::trace_err_chain;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use crate::tunnel_state_machine::gateway_ext::GatewayExt;
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

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use nym_dns::DnsConfig;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use nym_firewall::{
    AllowedClients, AllowedEndpoint, AllowedTunnelTraffic, Endpoint, FirewallPolicy,
    TransportProtocol,
};
use nym_gateway_directory::ResolvedConfig;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use nym_vpn_lib_types::TunnelConnectionData;
use nym_vpn_lib_types::{EstablishConnectionData, EstablishConnectionState, GatewayId};
use nym_vpn_network_config::{DiscoveryRefresherCommand, DiscoveryRefresherEvent};

/// Initial delay between retry attempts.
const INITIAL_WAIT_DELAY: Duration = Duration::from_secs(2);

/// Wait delay multiplier used for each subsequent retry attempt.
const DELAY_MULTIPLIER: u32 = 2;

/// Max wait delay between retry attempts.
const MAX_WAIT_DELAY: Duration = Duration::from_secs(15);

/// Number of fast retry attempts before switching to exponential backoff.
const FAST_RETRY_ATTEMPTS: u32 = 2;

/// Fast retry delay for network recovery scenarios (first FAST_RETRY_ATTEMPTS).
const NETWORK_RECOVERY_DELAY: Duration = Duration::from_millis(500);

type ResolveApiAddrsFuture = BoxFuture<'static, Result<ResolvedConfig>>;
type ReconnectDelayFuture = BoxFuture<'static, ()>;

pub struct ConnectingState {
    retry_attempt: u32,
    tunnel_monitor_handle: Option<TunnelMonitorHandle>,
    tunnel_monitor_event_sender: Option<TunnelMonitorEventSender>,
    tunnel_monitor_event_receiver: TunnelMonitorEventReceiver,
    selected_gateways: Option<SelectedGateways>,
    connection_data: Option<EstablishConnectionData>,
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    firewall_policy_params: ConnectingPolicyParameters,
    resolve_api_addrs_fut: Fuse<ResolveApiAddrsFuture>,
    reconnect_delay_fut: Fuse<ReconnectDelayFuture>,
}

impl ConnectingState {
    pub async fn enter(
        retry_attempt: u32,
        selected_gateways: Option<SelectedGateways>,
        shared_state: &mut SharedState,
    ) -> (Box<dyn TunnelStateHandler>, PrivateTunnelState) {
        // Pause Discovery Refresher until we have resolved all the domains
        shared_state
            .discovery_refresher_command_tx
            .send(DiscoveryRefresherCommand::Pause(true))
            .await
            .ok();

        // This prevents hickory resolver from getting confused when system DNS points to
        // local forwarder but it expects responses from upstream DNS servers.
        #[cfg(target_os = "macos")]
        if retry_attempt > FAST_RETRY_ATTEMPTS {
            if let Err(e) = Self::set_local_dns_resolver(shared_state).await {
                trace_err_chain!(e, "Failed to configure system to use filtering resolver",);
                return ErrorState::enter(ErrorStateReason::SetDns, shared_state).await;
            }
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

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        let firewall_policy_params = {
            let mut bridge_endpoints = Vec::new();
            if shared_state.tunnel_settings.bridges_enabled()
                && let Some(gateways) = &selected_gateways
                && let Some(params) = &gateways.entry_gateway().bridge_params
            {
                bridge_endpoints = params.get_addrs();
            }

            let firewall_policy_params = ConnectingPolicyParameters {
                enable_ipv6: shared_state.tunnel_settings.enable_ipv6,
                allow_lan: shared_state.tunnel_settings.allow_lan,
                wg_entry_endpoint: None,
                bridge_endpoints,
                ws_entry_endpoints: selected_gateways
                    .as_ref()
                    .map(|v| v.entry_gateway().endpoints())
                    .unwrap_or_default(),
                api_endpoints: Vec::new(),
                dns_servers: shared_state.tunnel_settings.default_dns_ips(),
                tunnel_interface: None,
            };

            if let Err(err) = Self::set_firewall_policy(shared_state, &firewall_policy_params) {
                trace_err_chain!(err, "failed to set firewall policy");
                return ErrorState::enter(ErrorStateReason::SetFirewallPolicy, shared_state).await;
            }
            firewall_policy_params
        };

        // If that fails, it's not really important
        let _ = shared_state
            .account_command_tx
            .set_vpn_api_firewall_up()
            .await;

        let resolve_config_fut = Fuse::terminated();
        let reconnect_delay_fut = if retry_attempt > 0 {
            let wait_delay = wait_delay(retry_attempt);
            tracing::info!("Waiting {}ms before reconnect", wait_delay.as_millis());
            tokio::time::sleep(wait_delay).boxed().fuse()
        } else {
            std::future::ready(()).boxed().fuse()
        };

        let (monitor_event_sender, monitor_event_receiver) = mpsc::unbounded_channel();

        let initial_connection_data =
            selected_gateways
                .as_ref()
                .map(|gateways| EstablishConnectionData {
                    entry_gateway: GatewayId::from(gateways.entry_gateway().clone()),
                    exit_gateway: GatewayId::from(gateways.exit_gateway().clone()),
                    tunnel: None,
                });

        let connecting_state = Self {
            tunnel_monitor_handle: None,
            tunnel_monitor_event_sender: Some(monitor_event_sender),
            tunnel_monitor_event_receiver: monitor_event_receiver,
            retry_attempt,
            selected_gateways,
            connection_data: initial_connection_data.clone(),
            resolve_api_addrs_fut: resolve_config_fut,
            reconnect_delay_fut,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            firewall_policy_params,
        };

        let tunnel_state = connecting_state.make_connecting_tunnel_state(
            shared_state,
            EstablishConnectionState::ResolvingApiAddresses,
        );

        (Box::new(connecting_state), tunnel_state)
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    fn set_firewall_policy(
        shared_state: &mut SharedState,
        params: &ConnectingPolicyParameters,
    ) -> Result<()> {
        let policy = params.as_policy();

        shared_state
            .firewall
            .apply_policy(policy)
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

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
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
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        Self::reset_routes(shared_state).await;

        NextTunnelState::NewState(DisconnectingState::enter(
            after_disconnect,
            tunnel_monitor_handle,
            shared_state,
        ))
    }

    async fn handle_tunnel_close(tombstone: Tombstone, _shared_state: &mut SharedState) {
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
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

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            self.firewall_policy_params.api_endpoints = resolved_gateway_config.all_socket_addrs();
            if let Err(err) = Self::set_firewall_policy(shared_state, &self.firewall_policy_params)
            {
                trace_err_chain!(err, "failed to set firewall policy");
                return NextTunnelState::NewState(
                    ErrorState::enter(ErrorStateReason::SetFirewallPolicy, shared_state).await,
                );
            }
        }

        if !resolved_gateway_config.has_resolver_overrides() {
            tracing::warn!(
                "There are no resolver overrides, which may result in the firewall blocking API requests"
            );
        } else {
            // Tell the Account Controller about the resolver overrrides
            if let Err(e) = shared_state
                .account_command_tx
                .set_resolver_overrides(Some(
                    resolved_gateway_config
                        .nym_vpn_api_resolver_overrides
                        .clone(),
                ))
                .await
            {
                trace_err_chain!(e, "Failed to set resolver overrides for account controller");
                return NextTunnelState::NewState(
                    ErrorState::enter(
                        ErrorStateReason::Internal(
                            "Failed to set static NYM API addresses to account controller"
                                .to_owned(),
                        ),
                        shared_state,
                    )
                    .await,
                );
            }

            // Tell the Discovery Refresher about the resolver overrides and resume it
            shared_state
                .discovery_refresher_command_tx
                .send(DiscoveryRefresherCommand::UseResolverOverrides(Some(
                    Box::new(
                        resolved_gateway_config
                            .nym_vpn_api_resolver_overrides
                            .clone(),
                    ),
                )))
                .await
                .ok();

            shared_state
                .discovery_refresher_command_tx
                .send(DiscoveryRefresherCommand::Pause(false))
                .await
                .ok();
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
            shared_state.wg_keys_db.clone(),
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            shared_state.route_handler.clone(),
            #[cfg(any(target_os = "ios", target_os = "android"))]
            shared_state.tun_provider.clone(),
        );

        self.tunnel_monitor_handle = Some(tunnel_monitor_handle);

        NextTunnelState::SameState(self)
    }

    async fn handle_registered_with_gateways(
        &mut self,
        connection_data: Box<EstablishConnectionData>,
        _shared_state: &mut SharedState,
    ) -> Result<()> {
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            // Only allow entry wg endpoint in firewall when bridges are not enabled.
            // Because all bridges are already added to firewall exceptions.
            let wg_entry_endpoint = if let Some(TunnelConnectionData::Wireguard(ref wg)) =
                connection_data.tunnel
                && !_shared_state.tunnel_settings.bridges_enabled()
            {
                Some(wg.entry.endpoint)
            } else {
                None
            };
            self.firewall_policy_params.wg_entry_endpoint = wg_entry_endpoint;
            Self::set_firewall_policy(_shared_state, &self.firewall_policy_params)?;
        }

        self.connection_data = Some(*connection_data);

        Ok(())
    }

    async fn handle_interface_up(
        &mut self,
        _tunnel_interface: TunnelInterface,
        connection_data: Box<EstablishConnectionData>,
        _shared_state: &mut SharedState,
    ) -> Result<()> {
        self.connection_data = Some(*connection_data);

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            self.firewall_policy_params.tunnel_interface = Some(_tunnel_interface);
            Self::set_firewall_policy(_shared_state, &self.firewall_policy_params)?;
        }

        Ok(())
    }

    async fn handle_selected_gateways(
        &mut self,
        gateways: Box<SelectedGateways>,
        _shared_state: &mut SharedState,
    ) -> Result<()> {
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        let set_policy_result = {
            if _shared_state.tunnel_settings.bridges_enabled()
                && let Some(params) = &gateways.entry_gateway().bridge_params
            {
                self.firewall_policy_params.bridge_endpoints = params.get_addrs()
            }

            self.firewall_policy_params.ws_entry_endpoints = gateways.entry_gateway().endpoints();
            Self::set_firewall_policy(_shared_state, &self.firewall_policy_params)
        };
        self.selected_gateways = Some(*gateways);

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
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

                self.resolve_api_addrs_fut = async move {
                    nym_gateway_directory::resolve_config(&gateway_config)
                        .await
                        .map_err(Box::new)
                        .map_err(Error::ResolveApiHostnames)
                }
                .boxed()
                .fuse();

                NextTunnelState::SameState(self)
            },
            resolved_gateway_config = &mut self.resolve_api_addrs_fut => {
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
                    TunnelMonitorEvent::RegisteringWithGateways => {
                        let new_state = self.make_connecting_tunnel_state(shared_state, EstablishConnectionState::RegisteringWithGateways);
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
                    TunnelMonitorEvent::RegisteredWithGateways { connection_data, reply_tx } => {
                        let next_state = match self.handle_registered_with_gateways(connection_data, shared_state).await {
                            Ok(()) => {
                                let new_state = self.make_connecting_tunnel_state(shared_state, EstablishConnectionState::ConnectingTunnel);
                                NextTunnelState::NewState((self, new_state))
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
                            #[cfg(not(any(target_os = "android", target_os = "ios")))]
                            Self::reset_routes(shared_state).await;
                            NextTunnelState::NewState(DisconnectedState::enter(None, shared_state).await)
                        }
                    },
                    TunnelCommand::SetAllowLan(allow_lan, complete_tx) => {
                        if shared_state.set_allow_lan(allow_lan) {
                            #[cfg(not(any(target_os = "android", target_os = "ios")))]
                            {
                                self.firewall_policy_params.allow_lan = allow_lan;
                                if let Err(e) = Self::set_firewall_policy(shared_state, &self.firewall_policy_params) {
                                    trace_err_chain!(e, "failed to set firewall policy");
                                    _ = complete_tx.send(());
                                    return NextTunnelState::NewState(ErrorState::enter(ErrorStateReason::SetFirewallPolicy, shared_state).await);
                                }
                            }
                        }
                        _ = complete_tx.send(());

                        NextTunnelState::SameState(self)
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
            Some(discovery_event) = shared_state.discovery_refresher_event_rx.recv() => {
                match discovery_event {
                   DiscoveryRefresherEvent::NewNetwork(network) => {
                        shared_state.nym_config.network_env = *network;
                        NextTunnelState::SameState(self)
                    }
                    DiscoveryRefresherEvent::Error(error) => {
                        trace_err_chain!(error, "Discovery refresher reported an error");
                        NextTunnelState::SameState(self)
                    }
                }
            }
            _ = shutdown_token.cancelled() => {
                if let Some(tunnel_monitor_handle) = self.tunnel_monitor_handle {
                    Self::disconnect(PrivateActionAfterDisconnect::Nothing, tunnel_monitor_handle, shared_state).await
                } else {
                    #[cfg(not(any(target_os = "android", target_os = "ios")))]
                    Self::reset_routes(shared_state).await;
                    NextTunnelState::NewState(DisconnectedState::enter(None, shared_state).await)
                }
            }
        }
    }
}

/// Firewall policy configuration when connecting
#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[derive(Debug, Clone)]
struct ConnectingPolicyParameters {
    /// Whether IPv6 is enabled
    enable_ipv6: bool,

    /// Whether to allow LAN traffic
    allow_lan: bool,

    /// WireGuard entry endpoint
    wg_entry_endpoint: Option<SocketAddr>,

    /// Bridge endpoints
    bridge_endpoints: Vec<SocketAddr>,

    /// Entry gateway websocket endpoints
    ws_entry_endpoints: Vec<SocketAddr>,

    /// API endpoints
    api_endpoints: Vec<SocketAddr>,

    /// DNS servers
    dns_servers: Vec<IpAddr>,

    /// Tunnel interface
    tunnel_interface: Option<TunnelInterface>,
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
impl ConnectingPolicyParameters {
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

        // Allow WireGuard and entry endpoint
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

        // Allow endpoints from bridge connections to the entry gateway.
        self.bridge_endpoints
            .iter()
            .filter(|addr| addr.is_ipv4() || (self.enable_ipv6 && addr.is_ipv6()))
            .for_each(|addr| {
                let allow_bridge_endpoint = AllowedEndpoint::new(
                    Endpoint::from_socket_address(*addr, TransportProtocol::Udp),
                    #[cfg(any(target_os = "linux", target_os = "macos"))]
                    AllowedClients::Root,
                    #[cfg(target_os = "windows")]
                    AllowedClients::current_exe(),
                );
                peer_endpoints.push(allow_bridge_endpoint);
            });

        // Allow API endpoints
        let allowed_endpoints = self
            .api_endpoints
            .iter()
            .filter(|ip| ip.is_ipv4() || (self.enable_ipv6 && ip.is_ipv6()))
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

        let tunnel = self
            .tunnel_interface
            .clone()
            .map(nym_firewall::TunnelInterface::from);

        // Set non-tunnel DNS to allow api client to use those DNS servers.
        let dns_config = DnsConfig::from_addresses(&[], &self.dns_servers).resolve(
            // pass empty because we already override the config with non-tunnel addresses.
            &[],
            #[cfg(target_os = "macos")]
            53,
        );

        FirewallPolicy::Connecting {
            peer_endpoints,
            tunnel,
            allow_lan: self.allow_lan,
            dns_config,
            allowed_endpoints,
            // todo: only allow connection towards entry endpoint?
            allowed_entry_tunnel_traffic: AllowedTunnelTraffic::All,
            allowed_exit_tunnel_traffic: AllowedTunnelTraffic::All,
            // todo: split tunneling
            #[cfg(target_os = "macos")]
            redirect_interface: None,
        }
    }
}

fn wait_delay(retry_attempt: u32) -> Duration {
    // Use fast retries for the first FAST_RETRY_ATTEMPTS to handle network recovery
    // Where the network reports as "online" before DNS/routing are ready.
    if retry_attempt <= FAST_RETRY_ATTEMPTS {
        NETWORK_RECOVERY_DELAY
    } else {
        // After fast retries, use exponential backoff for persistent failures
        let multiplier = retry_attempt
            .saturating_sub(FAST_RETRY_ATTEMPTS)
            .saturating_mul(DELAY_MULTIPLIER);
        let delay = INITIAL_WAIT_DELAY.saturating_mul(multiplier);
        std::cmp::min(delay, MAX_WAIT_DELAY)
    }
}
