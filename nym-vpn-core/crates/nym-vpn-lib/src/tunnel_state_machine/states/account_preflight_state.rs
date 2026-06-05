// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::{AccountControllerState, EstablishConnectionState};
use tokio::sync::{mpsc, watch};
use tokio_util::sync::CancellationToken;

use crate::tunnel_state_machine::{
    NextTunnelState, PrivateTunnelState, SharedState, TunnelCommand, TunnelStateHandler,
    states::{ConnectingState, DisconnectedState, ErrorState, OfflineState},
    tunnel::SelectedGateways,
};

enum AccountPreflight {
    Ready,
    Wait,
    Block,
}

pub struct AccountPreflightState {
    selected_gateways: Option<SelectedGateways>,
    account_state_rx: watch::Receiver<AccountControllerState>,
}

impl AccountPreflightState {
    fn account_preflight(shared_state: &SharedState) -> AccountPreflight {
        match shared_state.account_controller_state.get_state() {
            AccountControllerState::ReadyToConnect
            | AccountControllerState::Decentralised
            | AccountControllerState::UpgradeMode => AccountPreflight::Ready,
            AccountControllerState::Syncing
            | AccountControllerState::RequestingZkNyms
            | AccountControllerState::PendingSubscription => AccountPreflight::Wait,
            AccountControllerState::Offline
            | AccountControllerState::LoggedOut
            | AccountControllerState::Error(_) => AccountPreflight::Block,
        }
    }

    pub async fn enter(
        selected_gateways: Option<SelectedGateways>,
        shared_state: &mut SharedState,
    ) -> (Box<dyn TunnelStateHandler>, PrivateTunnelState) {
        match Self::account_preflight(shared_state) {
            AccountPreflight::Ready => {
                return ConnectingState::enter(0, selected_gateways, shared_state).await;
            }
            AccountPreflight::Block => {
                return DisconnectedState::enter(None, shared_state).await;
            }
            AccountPreflight::Wait => {}
        }

        DisconnectedState::reset_to_unrestricted_networking(None, shared_state).await;
        let account_state_rx = shared_state.account_controller_state.subscribe();

        match Self::account_preflight(shared_state) {
            AccountPreflight::Ready => {
                ConnectingState::enter(0, selected_gateways, shared_state).await
            }
            AccountPreflight::Block => DisconnectedState::enter(None, shared_state).await,
            AccountPreflight::Wait => {
                let state = Self::make_preflight_tunnel_state(shared_state);
                (
                    Box::new(Self {
                        selected_gateways,
                        account_state_rx,
                    }),
                    state,
                )
            }
        }
    }

    fn make_preflight_tunnel_state(shared_state: &SharedState) -> PrivateTunnelState {
        PrivateTunnelState::Connecting {
            retry_attempt: 0,
            state: EstablishConnectionState::AwaitingAccountReadiness,
            tunnel_type: shared_state.tunnel_settings.tunnel_type_used(),
            connection_data: None,
        }
    }
}

#[async_trait::async_trait]
impl TunnelStateHandler for AccountPreflightState {
    async fn handle_event(
        mut self: Box<Self>,
        shutdown_token: &CancellationToken,
        command_rx: &'async_trait mut mpsc::UnboundedReceiver<TunnelCommand>,
        shared_state: &'async_trait mut SharedState,
    ) -> NextTunnelState {
        tokio::select! {
            Some(command) = command_rx.recv() => {
                tracing::debug!("AccountPreflightState received command: {command:?}");
                match command {
                    TunnelCommand::Connect => NextTunnelState::SameState(self),
                    TunnelCommand::Disconnect => {
                        NextTunnelState::NewState(DisconnectedState::enter(None, shared_state).await)
                    }
                    TunnelCommand::SetTunnelSettings(tunnel_settings) => {
                        if DisconnectedState::apply_tunnel_settings(tunnel_settings, shared_state).await {
                            let new_state = Self::make_preflight_tunnel_state(shared_state);
                            NextTunnelState::NewState((self, new_state))
                        } else {
                            NextTunnelState::SameState(self)
                        }
                    }
                    TunnelCommand::Block(reason) => {
                        NextTunnelState::NewState(ErrorState::enter(reason, shared_state).await)
                    }
                }
            }
            account_state_changed = self.account_state_rx.changed() => {
                if account_state_changed.is_err() {
                    return NextTunnelState::NewState(DisconnectedState::enter(None, shared_state).await);
                }

                match Self::account_preflight(shared_state) {
                    AccountPreflight::Ready => {
                        NextTunnelState::NewState(ConnectingState::enter(0, self.selected_gateways.take(), shared_state).await)
                    }
                    AccountPreflight::Block => {
                        NextTunnelState::NewState(DisconnectedState::enter(None, shared_state).await)
                    }
                    AccountPreflight::Wait => NextTunnelState::SameState(self),
                }
            }
            Some(connectivity) = shared_state.connectivity_handle.next() => {
                if connectivity.is_offline() {
                    NextTunnelState::NewState(OfflineState::enter(true, self.selected_gateways.take(), shared_state).await)
                } else {
                    NextTunnelState::SameState(self)
                }
            }
            _ = shutdown_token.cancelled() => {
                NextTunnelState::Finished
            }
        }
    }
}
