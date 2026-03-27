// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! ```text
//! Connection middleware's state machine.
//! It tries to follow the user's desired target state and only uses the tunnel
//! event data to determine if the user's desired target is reached
//!
//!                tunnel connects by itself for some reason (tunnel connected happens)
//!        | =================================================================================== |
//!        ||                                                                                   ||
//!        ||                                                                                   ||
//!        ||        user wants to connect                    tunnel manages to connect         ||
//!        ||         (connect is clicked)                (tunnel connected event happens)      v
//! +--------------+  ==================>  +------------+  =============================> +-----------+
//! | Disconnected |                       | Connecting |                                 | Connected |
//! +--------------+  <==================  +------------+  <============================= +-----------+
//!        A          user doesn't want to                         tunnel disconnects           ||
//!        ||            connect anymore                             for some reason            ||
//!        ||       (disconnect is clicked)              (tunnel non-connected event happens)   ||
//!        ||                                                                                   ||
//!        ||                                                                                   ||
//!        | =================================================================================== |
//!                  user doesn't want to be connected anymore (disconnect is clicked)
//! ```

use nym_vpn_lib_types::TunnelEvent;
use tokio::{
    sync::{broadcast, mpsc},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

pub mod connected;
pub mod connecting;
pub mod disconnected;

#[derive(Debug)]
pub(super) enum ConnectionMiddlewareCommand {
    /// Start trying to connect the tunnel.
    Start,

    /// Stop trying to connect the tunnel or disconnect if already connected.
    Stop,
}

pub(super) struct SharedState {
    tunnel_event_rx: broadcast::Receiver<TunnelEvent>,
}

pub(super) enum NextConnectionMiddlewareState {
    NewState(
        (
            Box<dyn ConnectionMiddlewareStateHandler>,
            PrivateConnectionMiddlewareState,
        ),
    ),
    SameState(Box<dyn ConnectionMiddlewareStateHandler>),
    Finished,
}

#[async_trait::async_trait]
pub(super) trait ConnectionMiddlewareStateHandler: Send {
    async fn handle_event(
        mut self: Box<Self>,
        shutdown_token: &CancellationToken,
        command_rx: &'async_trait mut mpsc::UnboundedReceiver<ConnectionMiddlewareCommand>,
        shared_state: &'async_trait mut SharedState,
    ) -> NextConnectionMiddlewareState;
}

pub(super) struct ConnectionMiddlewareStateMachine {
    current_state_handler: Box<dyn ConnectionMiddlewareStateHandler>,
    command_rx: mpsc::UnboundedReceiver<ConnectionMiddlewareCommand>,
    shutdown_token: CancellationToken,
    shared_state: SharedState,
}

impl ConnectionMiddlewareStateMachine {
    pub fn spawn(
        current_state_handler: Box<dyn ConnectionMiddlewareStateHandler>,
        command_rx: mpsc::UnboundedReceiver<ConnectionMiddlewareCommand>,
        tunnel_event_rx: broadcast::Receiver<TunnelEvent>,
        shutdown_token: CancellationToken,
    ) -> JoinHandle<()> {
        let shared_state = SharedState { tunnel_event_rx };
        let conn_middlware_state_machine = ConnectionMiddlewareStateMachine {
            current_state_handler,
            command_rx,
            shutdown_token,
            shared_state,
        };

        tokio::spawn(conn_middlware_state_machine.run())
    }

    async fn run(mut self) {
        loop {
            let next_state = self
                .current_state_handler
                .handle_event(
                    &self.shutdown_token,
                    &mut self.command_rx,
                    &mut self.shared_state,
                )
                .await;

            match next_state {
                NextConnectionMiddlewareState::NewState((new_state_handler, new_state)) => {
                    self.current_state_handler = new_state_handler;
                    tracing::info!("New connection middleware state: {new_state:?}");
                }
                NextConnectionMiddlewareState::SameState(same_state) => {
                    self.current_state_handler = same_state;
                }
                NextConnectionMiddlewareState::Finished => break,
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(super) enum PrivateConnectionMiddlewareState {
    Disconnected,
    Connecting,
    Connected,
}
