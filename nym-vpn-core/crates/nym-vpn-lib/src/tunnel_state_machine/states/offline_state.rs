// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use nym_common::ErrorExt;
#[cfg(target_os = "macos")]
use nym_dns::DnsConfig;
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use nym_firewall::FirewallPolicy;

#[cfg(target_os = "macos")]
use crate::tunnel_state_machine::{states::ErrorState, ErrorStateReason};
use crate::tunnel_state_machine::{
    states::{ConnectingState, DisconnectedState},
    tunnel::SelectedGateways,
    NextTunnelState, PrivateTunnelState, SharedState, TunnelCommand, TunnelStateHandler,
};
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use crate::tunnel_state_machine::{Error, Result};

pub struct OfflineState {
    /// Whether to connect the tunnel once online
    reconnect: bool,

    /// Last known retry attempt before entering offline state.
    retry_attempt: u32,

    /// Gateways to which the tunnel will reconnect to once online
    selected_gateways: Option<SelectedGateways>,
}

impl OfflineState {
    pub async fn enter(
        reconnect: bool,
        retry_attempt: u32,
        selected_gateways: Option<SelectedGateways>,
        _shared_state: &mut SharedState,
    ) -> (Box<dyn TunnelStateHandler>, PrivateTunnelState) {
        #[cfg(target_os = "macos")]
        if Self::set_local_dns_resolver(_shared_state).await.is_err() {
            return Box::pin(ErrorState::enter(ErrorStateReason::Dns, _shared_state)).await;
        }

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        if let Err(e) = Self::set_firewall_policy(_shared_state) {
            log::error!(
                "{}",
                e.display_chain_with_msg("Failed to apply firewall policy for blocked state")
            );
        }

        (
            Box::new(Self {
                reconnect,
                retry_attempt,
                selected_gateways,
            }),
            PrivateTunnelState::Offline { reconnect },
        )
    }

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    fn set_firewall_policy(shared_state: &mut SharedState) -> Result<()> {
        let policy = FirewallPolicy::Blocked {
            // todo: fetch from config
            allow_lan: true,
            allowed_endpoints: Vec::new(),
            #[cfg(target_os = "macos")]
            dns_redirect_port: shared_state.filtering_resolver.listening_port(),
        };

        shared_state
            .firewall
            .apply_policy(policy)
            .map_err(Error::ApplyFirewallPolicy)
    }

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    fn reset_firewall_policy(shared_state: &mut SharedState) {
        if let Err(e) = shared_state.firewall.reset_policy() {
            tracing::error!(
                "{}",
                e.display_chain_with_msg("Failed to reset firewall policy")
            );
        }
    }

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    async fn reset_dns(shared_state: &mut SharedState) {
        if let Err(error) = shared_state.dns_handler.reset().await {
            log::error!("{}", error.display_chain_with_msg("Unable to reset DNS"));
        }
    }

    #[cfg(target_os = "macos")]
    async fn set_local_dns_resolver(shared_state: &mut SharedState) -> Result<()> {
        // Set system DNS to our local DNS resolver
        let system_dns = DnsConfig::default().resolve(
            &[std::net::Ipv4Addr::LOCALHOST.into()],
            shared_state.filtering_resolver.listening_port(),
        );
        shared_state
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
                    TunnelCommand::SetTunnelSettings(tunnel_settings) => {
                        shared_state.tunnel_settings = tunnel_settings;
                        NextTunnelState::SameState(self)
                    }
                }
            }
            Some(connectivity) = shared_state.offline_monitor.next() => {
                if connectivity.is_offline() {
                    NextTunnelState::SameState(self)
                } else if self.reconnect {
                    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
                    Self::reset_dns(shared_state).await;

                    NextTunnelState::NewState(ConnectingState::enter(
                        self.retry_attempt,
                        self.selected_gateways,
                        shared_state
                    ).await)
                } else {
                    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
                    Self::reset_dns(shared_state).await;

                    NextTunnelState::NewState(DisconnectedState::enter(shared_state).await)
                }
            }
            _ = shutdown_token.cancelled() => {
                #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
                {
                    Self::reset_dns(shared_state).await;
                    Self::reset_firewall_policy(shared_state);
                }
                NextTunnelState::Finished
            }
        }
    }
}
