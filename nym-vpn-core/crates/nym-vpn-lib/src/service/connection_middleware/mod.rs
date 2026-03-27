// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Middleware interface between the user and the tunnel state machine
//!
//! This layer tries to reflect the user's connection desire, and it's currently
//! composed of the following possibilities:
//!
//! 1. The user whishes to connect to a specific group of gateways and with a
//!    specific hop mode. In this case, this layer acts only as a forwarder for
//!    commands from user and events from tunnel state machine.
//!
//! 2. (WIP) The user whishes to connect and remain connected, without specifying
//!    more information. In this case, this layer will forward the relevant commands
//!    and events and even create tunnel state machine commands on its own, such that
//!    connection retries are transparent to the user and the connection is maintained
//!    as much as possible.

use nym_vpn_lib_types::{TargetState, TunnelEvent};
use tokio::{
    sync::{broadcast, mpsc},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use crate::service::{
    VpnServiceCommand,
    connection_middleware::state_machine::{
        ConnectionMiddlewareCommand, ConnectionMiddlewareStateMachine,
        disconnected::DisconnectedState,
    },
};

mod state_machine;

pub(super) struct ConnectionMiddleware {
    connection_middleware_sm_handle: JoinHandle<()>,
    conn_middleware_command_tx: mpsc::UnboundedSender<ConnectionMiddlewareCommand>,

    vpn_command_rx: mpsc::UnboundedReceiver<VpnServiceCommand>,
    forwarded_vpn_command_tx: mpsc::UnboundedSender<VpnServiceCommand>,

    forwarded_tunnel_event_tx: broadcast::Sender<TunnelEvent>,
    tunnel_event_rx: broadcast::Receiver<TunnelEvent>,

    shutdown_token: CancellationToken,
    sm_shutdown_token: CancellationToken,
}

impl ConnectionMiddleware {
    pub(super) fn new(
        vpn_command_rx: mpsc::UnboundedReceiver<VpnServiceCommand>,
        forwarded_tunnel_event_tx: broadcast::Sender<TunnelEvent>,
        shutdown_token: CancellationToken,
    ) -> (
        Self,
        mpsc::UnboundedReceiver<VpnServiceCommand>,
        broadcast::Sender<TunnelEvent>,
    ) {
        let (forwarded_vpn_command_tx, forwarded_vpn_command_rx) = mpsc::unbounded_channel();
        let (tunnel_event_tx, tunnel_event_rx) = broadcast::channel(10);
        let (conn_middleware_command_tx, conn_middleware_command_rx) = mpsc::unbounded_channel();
        let sm_shutdown_token = CancellationToken::new();

        let (current_state_handler, initial_state) = DisconnectedState::enter();
        tracing::info!("Initial ConnectionMiddleware state: {initial_state:?}");

        let connection_middleware_sm_handle = ConnectionMiddlewareStateMachine::spawn(
            current_state_handler,
            conn_middleware_command_rx,
            tunnel_event_tx.subscribe(),
            sm_shutdown_token.clone(),
        );

        (
            Self {
                connection_middleware_sm_handle,
                conn_middleware_command_tx,
                vpn_command_rx,
                forwarded_vpn_command_tx,
                forwarded_tunnel_event_tx,
                tunnel_event_rx,
                shutdown_token,
                sm_shutdown_token,
            },
            forwarded_vpn_command_rx,
            tunnel_event_tx,
        )
    }

    fn handle_command(&self, command: VpnServiceCommand) {
        if let VpnServiceCommand::SetTargetState(_, target_state) = &command {
            match target_state {
                TargetState::Unsecured => {
                    if self
                        .conn_middleware_command_tx
                        .send(ConnectionMiddlewareCommand::Stop)
                        .is_err()
                    {
                        tracing::error!("ConnectionMiddleware state machine died")
                    }
                }
                TargetState::Secured => {
                    if self
                        .conn_middleware_command_tx
                        .send(ConnectionMiddlewareCommand::Start)
                        .is_err()
                    {
                        tracing::error!("ConnectionMiddleware state machine died")
                    }
                }
            }
        }

        if self.forwarded_vpn_command_tx.send(command).is_err() {
            tracing::error!("Command channel is closed");
        }
    }

    fn handle_event(&self, event: TunnelEvent) {
        if self.forwarded_tunnel_event_tx.send(event).is_err() {
            tracing::error!("Failed to send tunnel event");
        }
    }

    pub(super) async fn run(mut self) {
        loop {
            tokio::select! {
                biased;
                _ = self.shutdown_token.cancelled() => {
                    tracing::debug!("ConnectionMiddleware : Received cancellation signal");
                    self.sm_shutdown_token.cancel();
                    if let Err(err) = self.connection_middleware_sm_handle.await {
                        tracing::warn!("ConnectionMiddleware state machine didn't finish correctly: {err:?}");
                    }
                    break;
                },
                Some(command) = self.vpn_command_rx.recv() => {
                    self.handle_command(command);
                }
                Ok(event) = self.tunnel_event_rx.recv() => {
                    self.handle_event(event);
                }
            }
        }
    }
}
