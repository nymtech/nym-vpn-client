// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use std::net::SocketAddr;
#[cfg(target_os = "ios")]
use std::{net::IpAddr, sync::Arc};

#[cfg(target_os = "macos")]
use nym_dns::DnsConfig;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[cfg(any(
    target_os = "linux",
    target_os = "macos",
    target_os = "windows",
    target_os = "ios",
    target_os = "android"
))]
use nym_common::trace_err_chain;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use nym_firewall::{AllowedClients, AllowedEndpoint, Endpoint, FirewallPolicy, TransportProtocol};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use nym_http_api_client::HickoryDnsResolver;

#[cfg(target_os = "ios")]
use crate::tunnel_provider::OSTunProvider;
#[cfg(target_os = "ios")]
use crate::tunnel_state_machine::blocking_tun::blocking_tunnel_settings;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use crate::tunnel_state_machine::{Error, Result};
use crate::tunnel_state_machine::{
    ErrorStateReason, NextTunnelState, PrivateTunnelState, SharedState, TunnelCommand,
    TunnelStateHandler,
    states::{ConnectingState, DisconnectedState, OfflineState},
};

pub struct ErrorState {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    firewall_policy_params: BlockedPolicyParameters,
}

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
            Self::set_blocking_network_settings(
                shared_state.tun_provider.clone(),
                shared_state.filtering_resolver.listen_addr().ip(),
            )
            .await;
        }

        #[cfg(target_os = "android")]
        if let Err(err) = shared_state.ensure_android_blocking_tun() {
            trace_err_chain!(err, "Failed to install Android blocking TUN in error state");
        }

        // Mobile: allow discovery/account networking; device traffic is covered by blocking TUN
        // (Android) or NE blocking settings (iOS). Control-plane sockets use bypass() on Android.
        #[cfg(any(target_os = "android", target_os = "ios"))]
        shared_state.allow_networking().await;

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        let firewall_policy_params = {
            let api_endpoints =
                if let Some(resolved_config) = shared_state.resolved_api_endpoints.as_ref() {
                    // Set the resolved addresses as static in the default (shared) DNS resolver. Any http
                    // client based on `nym-http-api-client::Client` that not modified to be independent will
                    // use these overrides automatically.
                    HickoryDnsResolver::shared().set_static_preresolve(resolved_config.addr_map());

                    resolved_config.all_socket_addrs()
                } else {
                    vec![]
                };

            let firewall_policy_params = BlockedPolicyParameters {
                enable_ipv6: shared_state.tunnel_settings.enable_ipv6,
                allow_lan: shared_state.tunnel_settings.allow_lan,
                api_endpoints,
            };

            if let Err(err) = Self::set_firewall_policy(shared_state, &firewall_policy_params) {
                trace_err_chain!(err, "failed to set firewall policy");
            }

            if firewall_policy_params.api_endpoints.is_empty() {
                tracing::warn!(
                    "No API endpoints were resolved. API communication will be blocked!"
                );
                shared_state.disallow_networking().await;
            } else {
                shared_state.allow_networking().await;
                shared_state
                    .gateway_provider
                    .set_active_geo_location(true)
                    .await;
            }

            firewall_policy_params
        };

        let blocked_state = Self {
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            firewall_policy_params,
        };

        // Notify the SOCKS5 proxy subprocess that the VPN tunnel is down.
        #[cfg(not(target_os = "ios"))]
        {
            shared_state.set_socks5_proxy_tunnel_addrs(None, None)
        }

        (Box::new(blocked_state), PrivateTunnelState::Error(reason))
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    fn set_firewall_policy(
        shared_state: &mut SharedState,
        params: &BlockedPolicyParameters,
    ) -> Result<()> {
        let policy = params.as_policy();

        nym_http_api_client::network_reconfigured();
        shared_state
            .firewall
            .apply_policy(policy)
            .map_err(Error::SetFirewallPolicy)
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    fn reset_firewall_policy(shared_state: &mut SharedState) {
        #[cfg(target_os = "linux")]
        shared_state.restore_nm_connectivity_check();

        if let Err(e) = shared_state.firewall.reset_policy() {
            trace_err_chain!(e, "Failed to reset firewall policy");
        }
        nym_http_api_client::network_reconfigured();
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
        let system_dns = DnsConfig {
            addresses: vec![shared_state.filtering_resolver.listen_addr().ip()],
            port: shared_state.filtering_resolver.listen_addr().port(),
        };

        shared_state
            .dns_handler
            .set("lo", system_dns)
            .await
            .inspect_err(|err| {
                trace_err_chain!(err, "Failed to configure system to use filtering resolver");
            })
            .map_err(Error::SetDns)
    }

    /// Configure tunnel with network settings blocking all traffic
    #[cfg(target_os = "ios")]
    async fn set_blocking_network_settings(
        tun_provider: Arc<dyn OSTunProvider>,
        dns_server: IpAddr,
    ) {
        let tunnel_network_settings = blocking_tunnel_settings(dns_server);

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
                tracing::debug!("ErrorState received command: {command:?}");
                match command {
                    TunnelCommand::Connect => {
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
                    TunnelCommand::SetTunnelSettings(tunnel_settings) => {
                        let diff = shared_state.tunnel_settings.diff(&tunnel_settings);
                        if diff.is_empty() {
                            return NextTunnelState::SameState(self);
                        }

                        shared_state.set_tunnel_settings(tunnel_settings).await;

                        #[cfg(any(target_os = "macos", target_os = "windows"))]
                        if diff.split_tunnel_changed() || diff.geo_exclusion_enabled_changed() {
                            let _ = shared_state.set_split_tunnel_exclude_paths().await;
                        }

                        #[cfg(not(target_os = "ios"))]
                        if diff.geo_exclusion_enabled_changed() {
                            shared_state.start_or_stop_socks5_proxy().await;
                        } else if diff.geo_exclusion_excluded_countries_changed() {
                            shared_state.set_socks5_proxy_excluded_countries();
                        }

                        if diff.enable_ad_blocking_changed() {
                            shared_state.enable_ad_blocking(shared_state.tunnel_settings.enable_ad_blocking).await;
                        }

                        #[cfg(not(any(target_os = "android", target_os = "ios")))]
                        if diff.allow_lan_changed() {
                            self.firewall_policy_params.allow_lan = shared_state.tunnel_settings.allow_lan;

                            if let Err(e) = Self::set_firewall_policy(shared_state, &self.firewall_policy_params) {
                                trace_err_chain!(e, "failed to set firewall policy");
                            }
                        }

                        NextTunnelState::SameState(self)
                    }
                    TunnelCommand::Block(reason) => {
                        NextTunnelState::NewState(ErrorState::enter(reason, shared_state).await)
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

// Firewall policy configuration when blocked
#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[derive(Debug, Clone)]
pub struct BlockedPolicyParameters {
    /// Whether IPv6 is enabled
    pub enable_ipv6: bool,

    /// Whether to allow LAN traffic
    pub allow_lan: bool,

    /// API endpoints
    pub api_endpoints: Vec<SocketAddr>,
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
impl BlockedPolicyParameters {
    pub fn as_policy(&self) -> FirewallPolicy {
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

        FirewallPolicy::Blocked {
            allow_lan: self.allow_lan,
            allowed_endpoints,
        }
    }
}
