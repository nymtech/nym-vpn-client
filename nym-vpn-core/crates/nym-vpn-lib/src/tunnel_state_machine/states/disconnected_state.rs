// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::tunnel_state_machine::{
    NextTunnelState, PrivateTunnelState, SharedState, TunnelCommand, TunnelStateHandler,
    states::{ConnectingState, OfflineState},
    tunnel::Tombstone,
};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use nym_common::trace_err_chain;
use nym_http_api_client::HickoryDnsResolver;

pub struct DisconnectedState;

impl DisconnectedState {
    pub async fn enter(
        tombstone: Option<Tombstone>,
        shared_state: &mut SharedState,
    ) -> (Box<dyn TunnelStateHandler>, PrivateTunnelState) {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        if let Err(err) = shared_state.split_tunnel.reset_tunnel().await {
            trace_err_chain!(err, "failed to reset split tunnel");
        }

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        Self::reset_dns(shared_state).await;

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        Self::reset_firewall_policy(shared_state);

        // Drop tombstone to close tunnel devices.
        drop(tombstone);

        // Clear addresses from the pre-resolve table in the (shared) DNS resolver.
        HickoryDnsResolver::shared().clear_preresolve();

        shared_state.allow_networking().await;
        shared_state
            .gateway_provider
            .set_active_geo_location(true)
            .await;

        // Notify the SOCKS5 proxy subprocess that the VPN tunnel is down
        #[cfg(not(target_os = "ios"))]
        {
            shared_state.set_socks5_proxy_tunnel_addrs(None, None);
        }

        (Box::new(Self), PrivateTunnelState::Disconnected)
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    fn reset_firewall_policy(shared_state: &mut SharedState) {
        if let Err(e) = shared_state.firewall.reset_policy() {
            trace_err_chain!(e, "Failed to reset firewall policy");
        }

        #[cfg(target_os = "linux")]
        shared_state.restore_nm_connectivity_check();

        nym_http_api_client::network_reconfigured();
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    async fn reset_dns(shared_state: &mut SharedState) {
        if let Err(error) = shared_state.dns_handler.reset().await {
            trace_err_chain!(error, "Failed to reset DNS");
        }
    }
}

#[async_trait::async_trait]
impl TunnelStateHandler for DisconnectedState {
    async fn handle_event(
        mut self: Box<Self>,
        shutdown_token: &CancellationToken,
        command_rx: &'async_trait mut mpsc::UnboundedReceiver<TunnelCommand>,
        shared_state: &'async_trait mut SharedState,
    ) -> NextTunnelState {
        tokio::select! {
            Some(command) = command_rx.recv() => {
                tracing::debug!("DisconnectedState received command: {command:?}");
                match command {
                    TunnelCommand::Connect => {
                        NextTunnelState::NewState(ConnectingState::enter(0, None, shared_state).await)
                    },
                    TunnelCommand::Disconnect => NextTunnelState::SameState(self),
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

                        NextTunnelState::SameState(self)
                    }
                    TunnelCommand::Block(_reason) => {
                        NextTunnelState::SameState(self)
                    }
                }
            }
            Some(connectivity) = shared_state.connectivity_handle.next() => {
                if connectivity.is_offline() {
                    NextTunnelState::NewState(OfflineState::enter(false, None, shared_state).await)
                } else {
                    NextTunnelState::SameState(self)
                }
            }
            _ = shutdown_token.cancelled() => {
                #[cfg(not(any(target_os = "android", target_os = "ios")))]
                Self::reset_dns(shared_state).await;
                NextTunnelState::Finished
            }
        }
    }
}
