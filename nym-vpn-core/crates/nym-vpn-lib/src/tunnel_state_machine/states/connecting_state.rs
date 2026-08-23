// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use std::net::SocketAddr;
use std::time::Duration;

use futures::{
    FutureExt,
    future::{BoxFuture, Fuse},
};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use nym_firewall::AllowedDns;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use crate::tunnel_state_machine::Error;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use crate::tunnel_state_machine::gateway_ext::GatewayExt;
use crate::tunnel_state_machine::{
    ErrorStateReason, NextTunnelState, PrivateActionAfterDisconnect, PrivateTunnelState, Result,
    SharedState, TunnelCommand, TunnelInterface, TunnelStateHandler,
    states::{ConnectedState, DisconnectingState, ErrorState, OfflineState},
    tunnel::{SelectedGateways, Tombstone},
    tunnel_monitor::{
        TunnelMonitor, TunnelMonitorEvent, TunnelMonitorEventReceiver, TunnelMonitorEventSender,
        TunnelMonitorHandle, TunnelParameters,
    },
};

use nym_common::trace_err_chain;
#[cfg(target_os = "macos")]
use nym_dns::DnsConfig;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use nym_firewall::{
    AllowedClients, AllowedEndpoint, AllowedTunnelTraffic, Endpoint, FirewallPolicy,
    TransportProtocol,
};
use nym_gateway_directory::ResolvedConfig;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use nym_http_api_client::HickoryDnsResolver;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use nym_vpn_lib_types::TunnelConnectionData;
use nym_vpn_lib_types::{
    AccountControllerErrorStateReason, AccountControllerState, EstablishConnectionData,
    EstablishConnectionState, GatewayLightInfo,
};

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
        // Disallow networking until firewall exceptions and resolver overrides are configured
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        shared_state.disallow_networking().await;

        // Always allow networking on mobile since there is no configurable firewall
        #[cfg(any(target_os = "android", target_os = "ios"))]
        shared_state.allow_networking().await;

        #[cfg(target_os = "macos")]
        if let Err(e) = Self::set_local_dns_resolver(shared_state).await {
            trace_err_chain!(e, "Failed to configure system to use filtering resolver");
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

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        let firewall_policy_params = {
            let mut bridge_endpoints = Vec::new();
            if shared_state.tunnel_settings.bridges_enabled()
                && let Some(gateways) = &selected_gateways
                && let Some(params) = &gateways.entry_gateway().bridge_params
            {
                bridge_endpoints = params.get_addrs();
            }

            #[cfg(target_os = "macos")]
            let redirect_interface = shared_state.split_tunnel.interface().await;

            let ws_entry_endpoints = selected_gateways
                .as_ref()
                .map(|v| v.entry_gateway().endpoints())
                .unwrap_or_default();

            let lp_entry_endpoints = selected_gateways
                .as_ref()
                .map(|v| v.entry_gateway().lp_endpoints())
                .unwrap_or_default();

            let api_endpoints = shared_state
                .resolved_api_endpoints
                .as_ref()
                .map(|r| r.all_socket_addrs())
                .unwrap_or_default();

            let firewall_policy_params = ConnectingPolicyParameters {
                enable_ipv6: shared_state.tunnel_settings.enable_ipv6,
                allow_lan: shared_state.tunnel_settings.allow_lan,
                wg_entry_endpoint: None,
                bridge_endpoints,
                ws_entry_endpoints,
                lp_entry_endpoints,
                api_endpoints,
                // Allow default DNS servers when connecting since those are used by http/client
                dns_servers: shared_state.tunnel_settings.allowed_default_dns_endpoints(),
                tunnel_interface: None,
                #[cfg(target_os = "macos")]
                redirect_interface,
            };

            if let Err(err) = Self::set_firewall_policy(shared_state, &firewall_policy_params) {
                trace_err_chain!(err, "failed to set firewall policy");
                return ErrorState::enter(ErrorStateReason::SetFirewallPolicy, shared_state).await;
            }
            firewall_policy_params
        };

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
                    entry_gateway: GatewayLightInfo::from(gateways.entry_gateway().clone()),
                    exit_gateway: GatewayLightInfo::from(gateways.exit_gateway().clone()),
                    tunnel: None,
                });

        let connecting_state = Self {
            tunnel_monitor_handle: None,
            tunnel_monitor_event_sender: Some(monitor_event_sender),
            tunnel_monitor_event_receiver: monitor_event_receiver,
            retry_attempt,
            selected_gateways,
            connection_data: initial_connection_data.clone(),
            resolve_api_addrs_fut: Fuse::terminated(),
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

        #[cfg(target_os = "linux")]
        shared_state.disable_nm_connectivity_check();

        nym_http_api_client::network_reconfigured();
        shared_state
            .firewall
            .apply_policy(policy)
            .map_err(Error::SetFirewallPolicy)
    }

    #[cfg(target_os = "macos")]
    async fn set_local_dns_resolver(shared_state: &mut SharedState) -> Result<()> {
        // Set system DNS to our local DNS resolver
        let system_dns = DnsConfig {
            addresses: vec![shared_state.filtering_resolver.listen_addr().ip()],
            port: shared_state.filtering_resolver.listen_addr().port(),
        };
        shared_state
            .dns_handler
            .set("lo", system_dns)
            .await
            .map_err(Error::SetDns)
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
        self,
        after_disconnect: PrivateActionAfterDisconnect,
        shared_state: &mut SharedState,
    ) -> NextTunnelState {
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        Self::reset_routes(shared_state).await;

        NextTunnelState::NewState(
            DisconnectingState::enter(after_disconnect, self.tunnel_monitor_handle, shared_state)
                .await,
        )
    }

    async fn handle_tunnel_close(tombstone: Tombstone, shared_state: &mut SharedState) {
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        shared_state.route_handler.remove_routes().await;

        #[cfg(not(target_os = "ios"))]
        {
            shared_state.set_socks5_proxy_tunnel_addrs(None, None);
        }

        #[cfg(any(target_os = "android", target_os = "ios"))]
        let _ = shared_state; // Avoid unused variable warning

        // drop tombstone to close tunnel devices
        let _ = tombstone;
    }

    async fn handle_reconnect_delay(
        #[allow(unused_mut)] mut self: Box<Self>,
        shared_state: &mut SharedState,
    ) -> NextTunnelState {
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            let gateway_config = shared_state.nym_config.gateway_config.clone();

            self.resolve_api_addrs_fut = async move {
                nym_gateway_directory::resolve_config(&gateway_config)
                    .await
                    .map_err(|err| Error::ResolveApiHostnames(Box::new(err)))
            }
            .boxed()
            .fuse();

            NextTunnelState::SameState(self)
        }

        #[cfg(any(target_os = "android", target_os = "ios"))]
        {
            // Start tunnel monitor immediately since there is no configurable firewall on mobile
            self.start_tunnel_monitor(shared_state).await
        }
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
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

        // Set the resolved addresses as static in the default (shared) DNS resolver. Any http
        // client based on `nym-http-api-client::Client` that not modified to be independent will
        // use these overrides automatically.
        HickoryDnsResolver::shared().set_static_preresolve(resolved_gateway_config.addr_map());

        self.firewall_policy_params.api_endpoints = resolved_gateway_config.all_socket_addrs();

        // Store resolved API endpoints for reuse in error state to enable API communication while blocking network.
        shared_state.resolved_api_endpoints = Some(resolved_gateway_config);

        if let Err(err) = Self::set_firewall_policy(shared_state, &self.firewall_policy_params) {
            trace_err_chain!(err, "failed to set firewall policy");
            return NextTunnelState::NewState(
                ErrorState::enter(ErrorStateReason::SetFirewallPolicy, shared_state).await,
            );
        }

        // Allow networking now when firewall and resolver overrides are configured.
        shared_state.allow_networking().await;

        Self::force_account_refresh_if_time_desynced(self.retry_attempt, shared_state).await;

        self.start_tunnel_monitor(shared_state).await
    }

    #[cfg(any(target_os = "android", target_os = "ios"))]
    async fn handle_resolved_gateway_config(
        self: Box<Self>,
        _resolver_result: Result<ResolvedConfig>,
        shared_state: &mut SharedState,
    ) -> NextTunnelState {
        Self::force_account_refresh_if_time_desynced(self.retry_attempt, shared_state).await;

        NextTunnelState::NewState(
            ErrorState::enter(
                ErrorStateReason::Internal(
                    "DNS resolution must not be performed on mobile. This is a logical error."
                        .to_owned(),
                ),
                shared_state,
            )
            .await,
        )
    }

    async fn start_tunnel_monitor(
        mut self: Box<Self>,
        shared_state: &mut SharedState,
    ) -> NextTunnelState {
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

        let tunnel_parameters = TunnelParameters {
            nym_config: shared_state.nym_config.clone(),
            tunnel_settings: shared_state.tunnel_settings.clone(),
            tunnel_constants: shared_state.tunnel_constants,
            selected_gateways: self.selected_gateways.clone(),
            user_agent: shared_state.user_agent.clone(),
            #[cfg(not(target_os = "android"))]
            filtering_resolver_addr: shared_state.filtering_resolver.listen_addr(),
        };
        #[cfg(target_os = "android")]
        let dns_filter = if shared_state.tunnel_settings.enable_ad_blocking {
            Some(shared_state.adblocker.get_dns_filter())
        } else {
            None
        };

        let tunnel_monitor_handle = TunnelMonitor::start(
            tunnel_parameters,
            shared_state.account_controller_state.clone(),
            shared_state.account_command_tx.clone(),
            shared_state.bandwidth_command_tx.clone(),
            shared_state.skew_manager.clone(),
            shared_state.gateway_provider.clone(),
            shared_state.topology_service.clone(),
            tunnel_monitor_event_sender,
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            shared_state.route_handler.clone(),
            #[cfg(any(target_os = "ios", target_os = "android"))]
            shared_state.tun_provider.clone(),
            #[cfg(target_os = "android")]
            dns_filter,
        );

        self.tunnel_monitor_handle = Some(tunnel_monitor_handle);

        NextTunnelState::SameState(self)
    }

    /// Requests account summary refresh on the very first connection attempt if account controller is in error state due to time desync issue.
    /// This provides an escape hatch making it possible to disconnect and reconnect the tunnel without ending up in the error state indefinitely.
    async fn force_account_refresh_if_time_desynced(
        retry_attempt: u32,
        shared_state: &SharedState,
    ) {
        if retry_attempt == 0
            && let AccountControllerState::Error(
                AccountControllerErrorStateReason::DeviceTimeDesynced,
            ) = shared_state.account_controller_state.get_state()
        {
            tracing::info!("Forcing account state refresh due to device time being desynced");
            if let Err(err) = shared_state
                .account_command_tx
                .refresh_account_state(true)
                .await
            {
                trace_err_chain!(
                    err,
                    "failed to request background refresh for account state"
                );
            }
        }
    }

    async fn handle_registered_with_gateways(
        &mut self,
        connection_data: Box<EstablishConnectionData>,
        shared_state: &mut SharedState,
    ) -> Result<()> {
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            // Only allow entry wg endpoint in firewall when bridges are not enabled.
            // Because all bridges are already added to firewall exceptions.
            let wg_entry_endpoint = if let Some(TunnelConnectionData::Wireguard(ref wg)) =
                connection_data.tunnel
                && !shared_state.tunnel_settings.bridges_enabled()
            {
                Some(wg.entry.endpoint)
            } else {
                None
            };
            self.firewall_policy_params.wg_entry_endpoint = wg_entry_endpoint;
            Self::set_firewall_policy(shared_state, &self.firewall_policy_params)?;
        }

        #[cfg(any(target_os = "android", target_os = "ios"))]
        let _ = shared_state; // Avoid unused variable warning

        self.connection_data = Some(*connection_data);

        Ok(())
    }

    async fn handle_interface_up(
        mut self: Box<Self>,
        tunnel_interface: TunnelInterface,
        connection_data: Box<EstablishConnectionData>,
        shared_state: &mut SharedState,
    ) -> NextTunnelState {
        self.connection_data = Some(*connection_data);

        #[cfg(any(target_os = "macos", target_os = "windows"))]
        if let Err(st_error_cause) = shared_state
            .enable_split_tunnel(tunnel_interface.exit_tunnel_metadata())
            .await
        {
            let after_disconnect = match st_error_cause {
                #[cfg(target_os = "macos")]
                nym_split_tunnel::SplitTunnelErrorCause::NeedFullDiskPermissions => {
                    PrivateActionAfterDisconnect::Error(ErrorStateReason::NeedFullDiskPermissions)
                }
                #[cfg(target_os = "macos")]
                nym_split_tunnel::SplitTunnelErrorCause::IsOffline => {
                    PrivateActionAfterDisconnect::Offline {
                        reconnect: true,
                        gateways: self.selected_gateways.clone(),
                    }
                }
                nym_split_tunnel::SplitTunnelErrorCause::Other => {
                    PrivateActionAfterDisconnect::Error(ErrorStateReason::SplitTunnel)
                }
            };

            return self.disconnect(after_disconnect, shared_state).await;
        }

        #[cfg(not(target_os = "ios"))]
        {
            let (tunnel_v4_addr, tunnel_v6_addr) =
                tunnel_interface.exit_tunnel_metadata().get_addresses();
            shared_state.set_socks5_proxy_tunnel_addrs(tunnel_v4_addr, tunnel_v6_addr);
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        let _ = tunnel_interface; // Avoid "unused" warning

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            self.firewall_policy_params.tunnel_interface = Some(tunnel_interface);

            if let Err(err) = Self::set_firewall_policy(shared_state, &self.firewall_policy_params)
            {
                trace_err_chain!(err, "Failed to set firewall policy");
                return self
                    .disconnect(
                        PrivateActionAfterDisconnect::Error(ErrorStateReason::SetFirewallPolicy),
                        shared_state,
                    )
                    .await;
            }
        }

        let state = self
            .make_connecting_tunnel_state(shared_state, EstablishConnectionState::ConnectingTunnel);

        NextTunnelState::NewState((self, state))
    }

    async fn handle_selected_gateways(
        &mut self,
        gateways: Box<SelectedGateways>,
        shared_state: &mut SharedState,
    ) -> Result<()> {
        let entry_gateway = gateways.entry_gateway();
        let exit_gateway = gateways.exit_gateway();
        tracing::debug!(
            "Using entry public key: {}",
            gateways.entry_keypair().public_key()
        );
        tracing::debug!(
            "Using exit public key: {}",
            gateways.exit_keypair().public_key()
        );

        tracing::info!(
            "Using entry gateway: {}, location: {}, performance: {}",
            entry_gateway.identity(),
            entry_gateway
                .two_letter_iso_country_code()
                .map_or_else(|| "unknown".to_string(), |code| code.to_string()),
            entry_gateway
                .mixnet_performance
                .map_or_else(|| "unknown".to_string(), |perf| perf.to_string()),
        );
        tracing::info!(
            "Using exit gateway: {}, location: {}, performance: {}",
            exit_gateway.identity(),
            exit_gateway
                .two_letter_iso_country_code()
                .map_or_else(|| "unknown".to_string(), |code| code.to_string()),
            exit_gateway
                .mixnet_performance
                .map_or_else(|| "unknown".to_string(), |perf| perf.to_string()),
        );
        tracing::info!(
            "Using exit router address {}",
            exit_gateway
                .ipr_address
                .map_or_else(|| "none".to_string(), |ipr| ipr.to_string())
        );

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        let set_policy_result = {
            if shared_state.tunnel_settings.bridges_enabled()
                && let Some(params) = &gateways.entry_gateway().bridge_params
            {
                self.firewall_policy_params.bridge_endpoints = params.get_addrs()
            }

            self.firewall_policy_params.ws_entry_endpoints = gateways.entry_gateway().endpoints();
            self.firewall_policy_params.lp_entry_endpoints =
                gateways.entry_gateway().lp_endpoints();
            Self::set_firewall_policy(shared_state, &self.firewall_policy_params)
        };
        self.selected_gateways = Some(*gateways);

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            set_policy_result
        }

        #[cfg(any(target_os = "ios", target_os = "android"))]
        {
            let _ = shared_state; // Avoid unused variable warning
            Ok(())
        }
    }

    fn make_connecting_tunnel_state(
        &self,
        shared_state: &SharedState,
        state: EstablishConnectionState,
    ) -> PrivateTunnelState {
        PrivateTunnelState::Connecting {
            retry_attempt: self.retry_attempt,
            state,
            tunnel_type: shared_state.tunnel_settings.tunnel_type_used(),
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
                self.handle_reconnect_delay(shared_state).await
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
                    TunnelMonitorEvent::AwaitingCredentialsAvailability => {
                        let new_state = self.make_connecting_tunnel_state(shared_state, EstablishConnectionState::AwaitingCredentialsAvailability);
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
                                NextTunnelState::NewState(DisconnectingState::enter(
                                    PrivateActionAfterDisconnect::Error(ErrorStateReason::SetFirewallPolicy),
                                    self.tunnel_monitor_handle.take(),
                                    shared_state
                                ).await)
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
                                NextTunnelState::NewState(DisconnectingState::enter(
                                    PrivateActionAfterDisconnect::Error(ErrorStateReason::SetFirewallPolicy),
                                    self.tunnel_monitor_handle.take(),
                                    shared_state
                                ).await)
                            }
                        };
                        _ = reply_tx.send(());
                        next_state
                    }
                    TunnelMonitorEvent::InterfaceUp {
                        tunnel_interface, connection_data, reply_tx
                    }  => {
                        let next_state = self.handle_interface_up(tunnel_interface, connection_data, shared_state).await;
                        _ = reply_tx.send(());
                        next_state
                    }
                    TunnelMonitorEvent::Up { tunnel_interface, connection_data } => {
                        shared_state.entry_blame.clear();
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
                                self.tunnel_monitor_handle.take(),
                                shared_state
                            ).await)
                        } else {
                            if let Some(tunnel_monitor_handle) = self.tunnel_monitor_handle.take() {
                                let tombstone = tunnel_monitor_handle.wait().await;
                                Self::handle_tunnel_close(tombstone, shared_state).await;
                            }

                            tracing::info!("Tunnel closed");

                            self.reconnect(shared_state).await
                        }
                    }
                    TunnelMonitorEvent::ConnectionFailed { entry_gateway_id, exit_gateway_id, exit_handshake_completed } => {
                        // WG handshake timed out or connectivity probe failed without a healthy
                        // metadata path; blacklist the exit so a different one is selected.
                        shared_state.gateway_provider.add_blacklisted_gateway(exit_gateway_id).await;
                        // A failure before the exit handshake ever completed may equally be the
                        // entry gateway's fault. Once the same entry accumulates enough
                        // pre-handshake failures while exits rotate, blacklist it too.
                        if shared_state.entry_blame.record_failure(entry_gateway_id, exit_handshake_completed) {
                            tracing::warn!(
                                "Blacklisted entry gateway {entry_gateway_id} after repeated connection failures without a completed exit handshake"
                            );
                            shared_state.gateway_provider.add_blacklisted_gateway(entry_gateway_id).await;
                        }
                        self.selected_gateways = None;
                        NextTunnelState::SameState(self)
                    }
                    TunnelMonitorEvent::RegistrationFailed { gateway_id } => {
                        // Registration with the entry gateway failed; blacklist it to avoid
                        // re-selecting the same failing gateway.
                        shared_state.gateway_provider.add_blacklisted_gateway(gateway_id).await;
                        self.selected_gateways = None;
                        NextTunnelState::SameState(self)
                    }
                }
           }
            Some(command) = command_rx.recv() => {
                tracing::debug!("ConnectingState received command: {command:?}");
                match command {
                    TunnelCommand::Connect => {
                        // Skip disconnecting state if tunnel monitor isn't running yet
                        if self.tunnel_monitor_handle.is_some() {
                            self.disconnect(PrivateActionAfterDisconnect::Reconnect, shared_state).await
                        } else {
                            NextTunnelState::NewState(ConnectingState::enter(self.retry_attempt, None, shared_state).await)
                        }
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
                                                    gateways: self.selected_gateways.clone(),
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
                                    return self.disconnect(PrivateActionAfterDisconnect::Error(ErrorStateReason::SetFirewallPolicy), shared_state).await;
                                }
                            }
                        }

                        // Not all changes require the tunnel to be reconnected
                        if diff.should_reconnect(shared_state.tunnel_settings.tunnel_type_used()) {
                            // Skip disconnecting state if tunnel monitor isn't running yet
                            if self.tunnel_monitor_handle.is_some() {
                                self.disconnect(PrivateActionAfterDisconnect::Reconnect, shared_state).await
                            } else {
                                let next_gateways = if diff.entry_point_changed() || diff.exit_point_changed() || diff.quic_changed() {
                                    None
                                } else {
                                    self.selected_gateways
                                };
                                NextTunnelState::NewState(ConnectingState::enter(self.retry_attempt, next_gateways, shared_state).await)
                            }
                        } else {
                             NextTunnelState::SameState(self)
                        }
                    }
                    TunnelCommand::Block(reason) => {
                        self.disconnect(PrivateActionAfterDisconnect::Error(reason), shared_state).await
                    }
                }
            }
            Some(connectivity) = shared_state.connectivity_handle.next() => {
                if connectivity.is_offline() {
                    let gateways = self.selected_gateways.clone();
                    self.disconnect(PrivateActionAfterDisconnect::Offline {
                        reconnect: true,
                        gateways,
                    }, shared_state).await
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

/// Firewall policy configuration when connecting
#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[derive(Debug, Clone, Eq, PartialEq)]
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

    /// Entry gateway LP control endpoints
    lp_entry_endpoints: Vec<SocketAddr>,

    /// API endpoints
    api_endpoints: Vec<SocketAddr>,

    /// Non-tunnel DNS servers
    dns_servers: Vec<Endpoint>,

    /// Tunnel interface
    tunnel_interface: Option<TunnelInterface>,

    /// Split tunnel redirect interface
    #[cfg(target_os = "macos")]
    redirect_interface: Option<String>,
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

        // Allow WireGuard and entry endpoint
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

        // Allow endpoints from bridge connections to the entry gateway.
        self.bridge_endpoints
            .iter()
            .filter(|addr| addr.is_ipv4() || (self.enable_ipv6 && addr.is_ipv6()))
            .for_each(|addr| {
                let allow_bridge_endpoint = AllowedEndpoint::new(
                    Endpoint::from_socket_address(*addr, TransportProtocol::Udp),
                    #[cfg(target_os = "linux")]
                    // On Linux, All is needed so the mangle chain rule sets fwmark for outbound traffic
                    AllowedClients::All,
                    #[cfg(target_os = "macos")]
                    AllowedClients::Root,
                    #[cfg(target_os = "windows")]
                    AllowedClients::current_exe(),
                );
                peer_endpoints.push(allow_bridge_endpoint);
            });

        // Allow API endpoints
        let mut allowed_endpoints = self
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

        // Allow LP control endpoints for LP-based registration.
        allowed_endpoints.extend(
            self.lp_entry_endpoints
                .iter()
                .filter(|addr| addr.is_ipv4() || (self.enable_ipv6 && addr.is_ipv6()))
                .map(|addr| {
                    tracing::info!("In policy lp ep: {addr}");
                    AllowedEndpoint::new(
                        Endpoint::from_socket_address(*addr, TransportProtocol::Tcp),
                        #[cfg(any(target_os = "linux", target_os = "macos"))]
                        AllowedClients::Root,
                        #[cfg(target_os = "windows")]
                        AllowedClients::current_exe(),
                    )
                }),
        );

        let tunnel = self
            .tunnel_interface
            .clone()
            .map(nym_firewall::TunnelInterface::from);

        let dns_config = AllowedDns::new(vec![], self.dns_servers.clone());

        FirewallPolicy::Connecting {
            peer_endpoints,
            tunnel,
            allow_lan: self.allow_lan,
            dns_config,
            allowed_endpoints,
            // todo: only allow connection towards entry endpoint?
            allowed_entry_tunnel_traffic: AllowedTunnelTraffic::All,
            allowed_exit_tunnel_traffic: AllowedTunnelTraffic::All,
            #[cfg(target_os = "macos")]
            redirect_interface: self.redirect_interface.clone(),
        }
    }
}

fn wait_delay(retry_attempt: u32) -> Duration {
    // Use fast retries for the first FAST_RETRY_ATTEMPTS to handle network recovery
    // where the network reports as "online" before DNS/routing are ready.
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn wait_delay_sequence() {
        let retry_attempt_values: Vec<u32> = (0..10).collect();
        let expected_delays: [Duration; 10] = [
            NETWORK_RECOVERY_DELAY,
            NETWORK_RECOVERY_DELAY,
            NETWORK_RECOVERY_DELAY,
            Duration::from_secs(4),
            Duration::from_secs(8),
            Duration::from_secs(12),
            MAX_WAIT_DELAY,
            MAX_WAIT_DELAY,
            MAX_WAIT_DELAY,
            MAX_WAIT_DELAY,
        ];

        let delay_values: Vec<Duration> = retry_attempt_values
            .iter()
            .map(|i| wait_delay(*i))
            .collect();
        assert_eq!(delay_values, expected_delays);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_connecting_peer_endpoints_allow_all_for_fwmark() {
        use std::net::{IpAddr, Ipv4Addr, SocketAddr};

        let params = ConnectingPolicyParameters {
            enable_ipv6: false,
            allow_lan: false,
            wg_entry_endpoint: Some(SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4)),
                51822,
            )),
            bridge_endpoints: vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::new(5, 6, 7, 8)), 4443)],
            ws_entry_endpoints: vec![SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(9, 10, 11, 12)),
                9000,
            )],
            lp_entry_endpoints: vec![],
            api_endpoints: vec![],
            dns_servers: vec![],
            tunnel_interface: None,
        };

        let policy = params.as_policy();

        for endpoint in policy.peer_endpoints() {
            assert!(
                endpoint.clients.allow_all(),
                "Linux connecting peer endpoints must use AllowedClients::All so nftables mangle sets fwmark"
            );
        }
    }
}
