// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use nym_common::trace_err_chain;
#[cfg(target_os = "macos")]
use nym_dns::DnsConfig;

#[cfg(target_os = "macos")]
use crate::tunnel_state_machine::resolver::LOCAL_DNS_RESOLVER;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use crate::tunnel_state_machine::{Error, Result, states::error_state::BlockedPolicyParameters};
#[cfg(target_os = "macos")]
use crate::tunnel_state_machine::{ErrorStateReason, states::ErrorState};
use crate::tunnel_state_machine::{
    NextTunnelState, PrivateTunnelState, SharedState, TunnelCommand, TunnelStateHandler,
    states::{ConnectingState, DisconnectedState},
    tunnel::SelectedGateways,
};

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
        #[cfg(target_os = "macos")]
        if Self::set_local_dns_resolver(shared_state).await.is_err() {
            return Box::pin(ErrorState::enter(ErrorStateReason::SetDns, shared_state)).await;
        }

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        let firewall_policy_params = BlockedPolicyParameters {
            allow_lan: shared_state.tunnel_settings.allow_lan,
        };

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        if let Err(e) = Self::set_firewall_policy(shared_state, &firewall_policy_params) {
            trace_err_chain!(e, "Failed to apply firewall policy for blocked state");
        }

        let _ = shared_state
            .account_command_tx
            .set_vpn_api_firewall_up()
            .await;

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

        shared_state
            .firewall
            .apply_policy(policy)
            .map_err(Error::SetFirewallPolicy)
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
            trace_err_chain!(error, "Unable to reset DNS");
        }
    }

    #[cfg(target_os = "macos")]
    async fn set_local_dns_resolver(shared_state: &mut SharedState) -> Result<()> {
        // Set system DNS to our local DNS resolver
        let system_dns = DnsConfig::default().resolve(
            &[shared_state.filtering_resolver.listen_addr().ip()],
            shared_state.filtering_resolver.listen_addr().port(),
        );
        shared_state
            .dns_handler
            .set("lo".to_owned(), system_dns)
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
                    TunnelCommand::SetAllowLan(allow_lan, complete_tx) => {
                        if shared_state.set_allow_lan(allow_lan) {
                            #[cfg(not(any(target_os = "android", target_os = "ios")))]
                            {
                                self.firewall_policy_params.allow_lan = allow_lan;
                                if let Err(err) = Self::set_firewall_policy(shared_state, &self.firewall_policy_params) {
                                    trace_err_chain!(err, "Failed to apply firewall policy for blocked state");
                                }
                            }
                        }

                        _ = complete_tx.send(());
                        NextTunnelState::SameState(self)
                    },
                    TunnelCommand::SetTunnelSettings(tunnel_settings) => {
                        shared_state.tunnel_settings = tunnel_settings;
                        NextTunnelState::SameState(self)
                    }
                }
            }
            Some(connectivity) = shared_state.connectivity_handle.next() => {
                // See: https://github.com/rust-lang/rust-clippy/issues/14799
                #[allow(clippy::collapsible_else_if)]
                if connectivity.is_offline() {
                    NextTunnelState::SameState(self)
                } else {
                    tracing::info!("Network connectivity restored, preparing to reconnect");

                    // Reset DNS to allow ConnectingState to resolve API hostnames.
                    // The local filtering resolver will be reconfigured by ConnectingState once connection is established.
                    #[cfg(not(any(target_os = "android", target_os = "ios")))]
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
