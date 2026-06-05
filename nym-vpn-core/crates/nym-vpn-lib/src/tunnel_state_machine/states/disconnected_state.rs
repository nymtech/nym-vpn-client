// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::tunnel_state_machine::{
    NextTunnelState, PrivateTunnelState, SharedState, TunnelCommand, TunnelSettings,
    TunnelSettingsDiff, TunnelStateHandler,
    states::{AccountPreflightState, OfflineState},
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
        Self::reset_to_unrestricted_networking(tombstone, shared_state).await;

        (Box::new(Self), PrivateTunnelState::Disconnected)
    }

    pub(super) async fn reset_to_unrestricted_networking(
        tombstone: Option<Tombstone>,
        shared_state: &mut SharedState,
    ) {
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
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    fn reset_firewall_policy(shared_state: &mut SharedState) {
        if let Err(e) = shared_state.firewall.reset_policy() {
            trace_err_chain!(e, "Failed to reset firewall policy");
        }
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    async fn reset_dns(shared_state: &mut SharedState) {
        if let Err(error) = shared_state.dns_handler.reset().await {
            trace_err_chain!(error, "Failed to reset DNS");
        }
    }

    /// Apply updated tunnel settings, running the platform-specific side
    /// effects for whichever fields changed.
    ///
    /// Returns `None` when the new settings are identical to the current ones
    /// (nothing applied), or `Some(diff)` describing the fields that changed.
    /// Callers that cache selected gateways (e.g. the reconnect paths feeding
    /// `AccountPreflightState`) must inspect the diff and drop their cache when
    /// `entry_point`, `exit_point`, or `quic` changed, otherwise a subsequent
    /// reconnect would use stale gateways.
    pub(super) async fn apply_tunnel_settings(
        tunnel_settings: TunnelSettings,
        shared_state: &mut SharedState,
    ) -> Option<TunnelSettingsDiff> {
        let diff = shared_state.tunnel_settings.diff(&tunnel_settings);
        if diff.is_empty() {
            return None;
        }

        shared_state.set_tunnel_settings(tunnel_settings).await;

        #[cfg(any(target_os = "macos", target_os = "windows"))]
        if diff.split_tunnel_changed() || diff.geo_exclusion_enabled_changed() {
            let _ = shared_state.set_split_tunnel_exclude_paths().await;
        }

        #[cfg(not(target_os = "ios"))]
        if diff.geo_exclusion_enabled_changed() {
            shared_state.start_or_stop_socks5_proxy().await;
        }

        if diff.enable_ad_blocking_changed() {
            shared_state
                .enable_ad_blocking(shared_state.tunnel_settings.enable_ad_blocking)
                .await;
        }

        Some(diff)
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
                        NextTunnelState::NewState(AccountPreflightState::enter(None, shared_state).await)
                    },
                    TunnelCommand::Disconnect => NextTunnelState::SameState(self),
                    TunnelCommand::SetTunnelSettings(tunnel_settings) => {
                        Self::apply_tunnel_settings(tunnel_settings, shared_state).await;
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
