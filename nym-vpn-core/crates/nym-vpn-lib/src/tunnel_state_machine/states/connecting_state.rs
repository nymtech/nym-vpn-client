use futures::{
    future::{BoxFuture, Fuse},
    FutureExt,
};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tun2::AsyncDevice;

use nym_ip_packet_requests::IpPair;

use crate::tunnel_state_machine::{
    states::{ConnectedState, DisconnectingState},
    tunnel::{self, mixnet::connected_tunnel::TunnelHandle, ConnectedMixnet},
    ActionAfterDisconnect, ErrorStateReason, NextTunnelState, SharedState, TunnelCommand,
    TunnelState, TunnelStateHandler,
};

type ConnectFut = BoxFuture<'static, tunnel::Result<ConnectedMixnet>>;

pub struct ConnectingState {
    connect_fut: Fuse<ConnectFut>,
}

impl ConnectingState {
    pub fn enter(shared_state: &mut SharedState) -> (Box<dyn TunnelStateHandler>, TunnelState) {
        let config = shared_state.config.clone();

        let connect_fut = tunnel::connect_mixnet(config, false).boxed().fuse();

        (Box::new(Self { connect_fut }), TunnelState::Connecting)
    }

    async fn on_connect_mixnet_client(
        result: tunnel::Result<ConnectedMixnet>,
        shared_state: &mut SharedState,
    ) -> NextTunnelState {
        match result {
            Ok(connected_mixnet) => {
                match Self::start_mixnet_tunnel(shared_state, connected_mixnet).await {
                    Ok(tunnel_handle) => NextTunnelState::NewState(ConnectedState::enter(
                        shared_state,
                        tunnel_handle,
                    )),
                    Err(e) => {
                        tracing::error!("Failed to start mixnet tunnel: {}", e);
                        NextTunnelState::NewState(DisconnectingState::enter(
                            ActionAfterDisconnect::Error(e.error_state_reason()),
                            shared_state,
                        ))
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to connect mixnet client: {}", e);
                NextTunnelState::NewState(DisconnectingState::enter(
                    ActionAfterDisconnect::Error(e.error_state_reason()),
                    shared_state,
                ))
            }
        }
    }

    async fn start_mixnet_tunnel(
        shared_state: &mut SharedState,
        connected_mixnet: ConnectedMixnet,
    ) -> tunnel::Result<TunnelHandle> {
        let connected_tunnel = connected_mixnet
            .connect_tunnel(shared_state.config.nym_ips.clone())
            .await?;

        let entry_mixnet_gateway_ip = connected_tunnel.entry_mixnet_gateway_ip();
        let mtu = shared_state.config.nym_mtu.unwrap_or(1500);
        let tun_device = Self::create_device(connected_tunnel.interface_addresses(), mtu);

        Ok(connected_tunnel.run(tun_device).await)
    }

    fn create_device(interface_addresses: IpPair, mtu: u16) -> tunnel::Result<AsyncDevice> {
        let mut tun_config = tun2::Configuration::default();
        tun_config.address(interface_addresses.ipv4).mtu(mtu).up();

        tun2::create_as_async(&tun_config).map_err(tunnel::Error::CreateTunDevice)
    }
}

#[async_trait::async_trait]
impl TunnelStateHandler for ConnectingState {
    async fn handle_event(
        mut self: Box<Self>,
        shutdown_token: &CancellationToken,
        command_rx: &'async_trait mut mpsc::UnboundedReceiver<TunnelCommand>,
        shared_state: &'async_trait mut SharedState,
    ) -> NextTunnelState {
        tokio::select! {
            _ = shutdown_token.cancelled() => {
                NextTunnelState::NewState(DisconnectingState::enter(ActionAfterDisconnect::Nothing, shared_state))
            }
            result = &mut self.connect_fut => {
                Self::on_connect_mixnet_client(result, shared_state).await
            }
            Some(command) = command_rx.recv() => {
                match command {
                    TunnelCommand::Connect => NextTunnelState::SameState(self),
                    TunnelCommand::Disconnect => {
                        NextTunnelState::NewState(DisconnectingState::enter(ActionAfterDisconnect::Nothing, shared_state))
                    },
                }
            }
            else => NextTunnelState::Finished
        }
    }
}

// todo: move impl
impl tunnel::Error {
    fn error_state_reason(&self) -> ErrorStateReason {
        // todo: map errors to reason
        ErrorStateReason::EstablishMixnetConnection
    }
}
