// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[cfg(target_os = "macos")]
use crate::tunnel_state_machine::ErrorStateReason;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use crate::tunnel_state_machine::{Error, Result, states::error_state::BlockedPolicyParameters};
use crate::tunnel_state_machine::{
    NextTunnelState, PrivateTunnelState, SharedState, TunnelCommand, TunnelStateHandler,
    states::{ConnectingState, DisconnectedState, ErrorState},
    tunnel::SelectedGateways,
};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use nym_common::trace_err_chain;
#[cfg(target_os = "macos")]
use nym_dns::DnsConfig;

pub struct OfflineState {
    /// Whether to connect the tunnel once online
    reconnect: bool,

    /// Gateways to which the tunnel will reconnect to once online
    selected_gateways: Option<SelectedGateways>,

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    firewall_policy_params: BlockedPolicyParameters,
}

impl OfflineState {
    pub async fn enter(
        reconnect: bool,
        selected_gateways: Option<SelectedGateways>,
        shared_state: &mut SharedState,
    ) -> (Box<dyn TunnelStateHandler>, PrivateTunnelState) {
        shared_state.disallow_networking().await;

        #[cfg(target_os = "macos")]
        if Self::set_local_dns_resolver(shared_state).await.is_err() {
            return Box::pin(ErrorState::enter(ErrorStateReason::SetDns, shared_state)).await;
        }

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        let firewall_policy_params = BlockedPolicyParameters {
            enable_ipv6: shared_state.tunnel_settings.enable_ipv6,
            allow_lan: shared_state.tunnel_settings.allow_lan,
            api_endpoints: vec![],
        };

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        if let Err(e) = Self::set_firewall_policy(shared_state, &firewall_policy_params) {
            trace_err_chain!(e, "Failed to apply firewall policy for blocked state");
        }

        (
            Box::new(Self {
                reconnect,
                selected_gateways,
                #[cfg(not(any(target_os = "android", target_os = "ios")))]
                firewall_policy_params,
            }),
            PrivateTunnelState::Offline { reconnect },
        )
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
            trace_err_chain!(error, "Unable to reset DNS");
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
}

#[async_trait::async_trait]
impl TunnelStateHandler for OfflineState {
    async fn handle_event(
        mut self: Box<Self>,
        shutdown_token: &CancellationToken,
        command_rx: &'async_trait mut mpsc::UnboundedReceiver<TunnelCommand>,
        shared_state: &'async_trait mut SharedState,
    ) -> NextTunnelState {
        tokio::select! {
            Some(command) = command_rx.recv() => {
                tracing::debug!("OfflineState received command: {command:?}");
                match command {
                    TunnelCommand::Connect => {
                        if self.reconnect {
                            NextTunnelState::SameState(self)
                        } else {
                            self.reconnect = true;
                            let new_state = PrivateTunnelState::Offline { reconnect: self.reconnect };
                            NextTunnelState::NewState((self, new_state))
                        }
                    },
                    TunnelCommand::Disconnect => {
                        if self.reconnect {
                            self.reconnect = false;
                            let new_state = PrivateTunnelState::Offline { reconnect: self.reconnect };
                            NextTunnelState::NewState((self, new_state))
                        } else {
                            NextTunnelState::SameState(self)
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
                            shared_state
                                .start_or_stop_socks5_proxy()
                                .await;
                        }

                        if diff.enable_ad_blocking_changed() {
                            shared_state.enable_ad_blocking(shared_state.tunnel_settings.enable_ad_blocking).await;
                        }

                        #[cfg(not(any(target_os = "android", target_os = "ios")))]
                        {
                            if diff.allow_lan_changed() {
                                self.firewall_policy_params.allow_lan = shared_state.tunnel_settings.allow_lan;

                                if let Err(e) = Self::set_firewall_policy(shared_state, &self.firewall_policy_params) {
                                    trace_err_chain!(e, "failed to set firewall policy");
                                }
                            }
                        }

                        if diff.entry_point_changed() || diff.exit_point_changed() || diff.quic_changed() {
                            self.selected_gateways = None;
                        };

                        NextTunnelState::SameState(self)
                    }
                    TunnelCommand::Block(reason) => {
                        NextTunnelState::NewState(ErrorState::enter(reason, shared_state).await)
                    }
                }
            }
            Some(connectivity) = shared_state.connectivity_handle.next() => {
                if connectivity.is_offline() {
                    NextTunnelState::SameState(self)
                } else {
                    #[cfg(any(target_os = "linux", target_os = "windows"))]
                    Self::reset_dns(shared_state).await;

                    if self.reconnect {
                        NextTunnelState::NewState(ConnectingState::enter(0, self.selected_gateways, shared_state).await)
                    } else {
                        NextTunnelState::NewState(DisconnectedState::enter(None, shared_state).await)
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
