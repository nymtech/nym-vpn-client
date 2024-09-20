mod dns_handler;
mod firewall_handler;
mod route_handler;
mod states;
mod tunnel;

use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use dns_handler::DnsHandler;
use firewall_handler::FirewallHandler;
use route_handler::RouteHandler;
use states::DisconnectedState;
use tunnel::mixnet::connected_tunnel::TunnelHandle;

use crate::GenericNymVpnConfig;

#[async_trait::async_trait]
trait TunnelStateHandler: Send {
    async fn handle_event(
        mut self: Box<Self>,
        shutdown_token: &CancellationToken,
        command_rx: &'async_trait mut mpsc::UnboundedReceiver<TunnelCommand>,
        shared_state: &'async_trait mut SharedState,
    ) -> NextTunnelState;
}

enum NextTunnelState {
    NewState((Box<dyn TunnelStateHandler>, TunnelState)),
    SameState(Box<dyn TunnelStateHandler>),
    Finished,
}

#[derive(Debug)]
pub enum TunnelCommand {
    Connect,
    Disconnect,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TunnelState {
    Disconnected,
    Connecting,
    Connected,
    Disconnecting {
        after_disconnect: ActionAfterDisconnect,
    },
    Error(ErrorStateReason),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ActionAfterDisconnect {
    Nothing,
    Reconnect,
    Error(ErrorStateReason),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ErrorStateReason {
    /// Failure to configure routing
    Routing,

    /// Failure to establish mixnet connection
    EstablishMixnetConnection,

    /// Tunnel went down at runtime
    TunnelDown,
}

#[derive(Debug)]
pub enum TunnelEvent {
    NewState(TunnelState),
}

pub struct SharedState {
    route_handler: RouteHandler,
    firewall_handler: FirewallHandler,
    dns_handler: DnsHandler,
    tunnel_handle: Option<TunnelHandle>,
    config: GenericNymVpnConfig,
}

pub struct TunnelStateMachine {
    current_state_handler: Box<dyn TunnelStateHandler>,
    shared_state: SharedState,
    command_receiver: mpsc::UnboundedReceiver<TunnelCommand>,
    event_sender: mpsc::UnboundedSender<TunnelEvent>,
    shutdown_token: CancellationToken,
}

impl TunnelStateMachine {
    pub async fn spawn(
        command_receiver: mpsc::UnboundedReceiver<TunnelCommand>,
        event_sender: mpsc::UnboundedSender<TunnelEvent>,
        config: GenericNymVpnConfig,
        shutdown_token: CancellationToken,
    ) -> Result<JoinHandle<()>> {
        let (current_state_handler, _) = DisconnectedState::enter();

        let route_handler = RouteHandler::new()
            .await
            .map_err(Error::CreateRouteHandler)?;
        let dns_handler = DnsHandler::new(
            #[cfg(target_os = "linux")]
            &route_handler,
        )
        .await
        .map_err(Error::CreateDnsHandler)?;
        let firewall_handler = FirewallHandler::new().map_err(Error::CreateFirewallHandler)?;

        let shared_state = SharedState {
            route_handler,
            firewall_handler,
            dns_handler,
            tunnel_handle: None,
            config,
        };

        let tunnel_state_machine = Self {
            current_state_handler,
            shared_state,
            command_receiver,
            event_sender,
            shutdown_token,
        };

        Ok(tokio::spawn(tunnel_state_machine.run()))
    }

    async fn run(mut self) {
        loop {
            let next_state = self
                .current_state_handler
                .handle_event(
                    &self.shutdown_token,
                    &mut self.command_receiver,
                    &mut self.shared_state,
                )
                .await;

            match next_state {
                NextTunnelState::NewState((new_state_handler, new_state)) => {
                    self.current_state_handler = new_state_handler;

                    log::debug!("New tunnel state: {:?}", new_state);
                    let _ = self.event_sender.send(TunnelEvent::NewState(new_state));
                }
                NextTunnelState::SameState(same_state) => {
                    self.current_state_handler = same_state;
                }
                NextTunnelState::Finished => break,
            }
        }

        self.shared_state.route_handler.stop().await;
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to create a route handler")]
    CreateRouteHandler(#[source] route_handler::Error),

    #[error("failed to create a dns handler")]
    CreateDnsHandler(#[source] dns_handler::Error),

    #[error("failed to create firewall handler")]
    CreateFirewallHandler(#[source] firewall_handler::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
