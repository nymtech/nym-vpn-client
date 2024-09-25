#[cfg(not(target_os = "linux"))]
use std::net::IpAddr;
use std::net::Ipv4Addr;

use futures::{
    future::{BoxFuture, Fuse},
    FutureExt,
};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tun::{AsyncDevice, Device};

use nym_ip_packet_requests::IpPair;

#[cfg(target_os = "linux")]
use crate::mixnet::SharedMixnetClient;

use crate::tunnel_state_machine::{
    states::{ConnectedState, DisconnectingState},
    tun_ipv6,
    tunnel::{self, any_tunnel_handle::AnyTunnelHandle, ConnectedMixnet},
    ActionAfterDisconnect, Error, NextTunnelState, Result, SharedState, TunnelCommand, TunnelState,
    TunnelStateHandler,
};

const DEFAULT_TUN_MTU: u16 = 1500;

type ConnectFut = BoxFuture<'static, tunnel::Result<ConnectedMixnet>>;

pub struct ConnectingState {
    connect_fut: Fuse<ConnectFut>,
}

impl ConnectingState {
    pub fn enter(shared_state: &mut SharedState) -> (Box<dyn TunnelStateHandler>, TunnelState) {
        let config = shared_state.config.clone();

        let connect_fut = tunnel::connect_mixnet(config, shared_state.enable_wireguard)
            .boxed()
            .fuse();

        (Box::new(Self { connect_fut }), TunnelState::Connecting)
    }

    async fn on_connect_mixnet_client(
        result: tunnel::Result<ConnectedMixnet>,
        shared_state: &mut SharedState,
    ) -> NextTunnelState {
        match result.map_err(Error::ConnectMixnetClient) {
            Ok(connected_mixnet) => {
                match Self::start_tunnel(connected_mixnet, shared_state).await {
                    Ok(tunnel_handle) => NextTunnelState::NewState(ConnectedState::enter(
                        tunnel_handle,
                        shared_state,
                    )),
                    Err(e) => {
                        tracing::error!("Failed to start the tunnel: {}", e);
                        NextTunnelState::NewState(DisconnectingState::enter(
                            ActionAfterDisconnect::Error(e.error_state_reason()),
                            None,
                            shared_state,
                        ))
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to connect mixnet client: {}", e);
                NextTunnelState::NewState(DisconnectingState::enter(
                    ActionAfterDisconnect::Error(e.error_state_reason()),
                    None,
                    shared_state,
                ))
            }
        }
    }

    async fn start_tunnel(
        connected_mixnet: ConnectedMixnet,
        shared_state: &mut SharedState,
    ) -> Result<AnyTunnelHandle> {
        #[cfg(target_os = "linux")]
        Self::set_mixnet_client_fwmark(connected_mixnet.mixnet_client.clone()).await;

        if shared_state.enable_wireguard {
            Self::start_wireguard_tunnel(connected_mixnet, shared_state).await
        } else {
            Self::start_mixnet_tunnel(connected_mixnet, shared_state).await
        }
    }

    async fn start_mixnet_tunnel(
        connected_mixnet: ConnectedMixnet,
        shared_state: &mut SharedState,
    ) -> Result<AnyTunnelHandle> {
        let connected_tunnel = connected_mixnet
            .connect_mixnet_tunnel(shared_state.config.nym_ips.clone())
            .await
            .map_err(Error::ConnectMixnetTunnel)?;

        let enable_ipv6 = true;
        let mtu = shared_state.config.nym_mtu.unwrap_or(DEFAULT_TUN_MTU);
        let interface_addresses = connected_tunnel.interface_addresses();

        let tun_device: AsyncDevice = Self::create_device(interface_addresses, mtu, enable_ipv6)?;
        let tun_name = tun_device
            .get_ref()
            .name()
            .map_err(Error::GetTunDeviceName)?;

        tracing::debug!("Created tun device: {}", tun_name);

        Self::set_routes(
            &tun_name,
            #[cfg(not(target_os = "linux"))]
            connected_tunnel.entry_mixnet_gateway_ip(),
            enable_ipv6,
            shared_state,
        )
        .await?;
        Self::set_dns(&tun_name, shared_state)?;

        Ok(AnyTunnelHandle::from(
            connected_tunnel.run(tun_device).await,
        ))
    }

    async fn start_wireguard_tunnel(
        connected_mixnet: ConnectedMixnet,
        shared_state: &mut SharedState,
    ) -> Result<AnyTunnelHandle> {
        let connected_tunnel = connected_mixnet
            .connect_wireguard_tunnel()
            .await
            .map_err(Error::ConnectWireguardTunnel)?;

        let enable_ipv6 = false;
        let conn_data = connected_tunnel.connection_data();

        let mut entry_config = tun::Configuration::default();
        entry_config
            .address(conn_data.entry.gateway.private_ipv4)
            .netmask(Ipv4Addr::BROADCAST)
            .mtu(i32::from(connected_tunnel.entry_mtu()))
            .up();
        let entry_tun = tun::create_as_async(&entry_config).map_err(Error::CreateTunDevice)?;
        let entry_tun_name = entry_tun
            .get_ref()
            .name()
            .map_err(Error::GetTunDeviceName)?;
        tracing::info!("Created entry tun device: {}", entry_tun_name);

        let mut exit_config = tun::Configuration::default();
        exit_config
            .address(conn_data.exit.gateway.private_ipv4)
            .netmask(Ipv4Addr::BROADCAST)
            .destination(conn_data.entry.gateway.private_ipv4)
            .mtu(i32::from(connected_tunnel.exit_mtu()))
            .up();
        let exit_tun = tun::create_as_async(&exit_config).map_err(Error::CreateTunDevice)?;
        let exit_tun_name = exit_tun.get_ref().name().map_err(Error::GetTunDeviceName)?;
        tracing::info!("Created exit tun device: {}", exit_tun_name);

        Self::set_routes(
            &exit_tun_name,
            #[cfg(not(target_os = "linux"))]
            conn_data.entry.gateway.endpoint.ip(),
            enable_ipv6,
            shared_state,
        )
        .await?;
        Self::set_dns(&exit_tun_name, shared_state)?;

        Ok(AnyTunnelHandle::from(
            connected_tunnel
                .run(entry_tun, exit_tun)
                .map_err(Error::RunWireguardTunnel)?,
        ))
    }

    fn set_dns(tun_name: &str, shared_state: &mut SharedState) -> Result<()> {
        let dns_servers = shared_state
            .config
            .dns
            .map(|ip_addr| vec![ip_addr])
            .unwrap_or_else(|| crate::DEFAULT_DNS_SERVERS.to_vec());

        shared_state
            .dns_handler
            .set(tun_name, &dns_servers)
            .map_err(Error::SetDns)
    }

    #[cfg(target_os = "linux")]
    async fn set_mixnet_client_fwmark(shared_mixnet_client: SharedMixnetClient) {
        use nix::sys::socket::{self, sockopt::Mark};
        use std::os::fd::BorrowedFd;

        let gateway_ws_fd = shared_mixnet_client
            .lock()
            .await
            .as_ref()
            .and_then(|mixnet_client| mixnet_client.gateway_connection().gateway_ws_fd);

        if let Some(gateway_ws_fd) = gateway_ws_fd {
            let borrowed_fd = unsafe { BorrowedFd::borrow_raw(gateway_ws_fd) };
            socket::setsockopt(
                &borrowed_fd,
                Mark,
                &crate::tunnel_state_machine::route_handler::TUNNEL_FWMARK,
            )
            .expect("failed to set fwmark")
        }
    }

    async fn set_routes(
        tun_name: &str,
        #[cfg(not(target_os = "linux"))] entry_gateway_ip: IpAddr,
        enable_ipv6: bool,
        shared_state: &mut SharedState,
    ) -> Result<()> {
        shared_state
            .route_handler
            .add_routes(
                tun_name,
                #[cfg(not(target_os = "linux"))]
                entry_gateway_ip,
                enable_ipv6,
            )
            .await
            .map_err(Error::AddRoutes)
    }

    fn create_device(
        interface_addresses: IpPair,
        mtu: u16,
        enable_ipv6: bool,
    ) -> Result<AsyncDevice> {
        let mut tun_config = tun::Configuration::default();

        tun_config
            .address(interface_addresses.ipv4)
            .mtu(i32::from(mtu))
            .up();

        #[cfg(target_os = "linux")]
        tun_config.platform(|platform_config| {
            platform_config.packet_information(false);
        });

        let tun_device = tun::create_as_async(&tun_config).map_err(Error::CreateTunDevice)?;

        if enable_ipv6 {
            let tun_name = tun_device
                .get_ref()
                .name()
                .map_err(Error::GetTunDeviceName)?;

            tun_ipv6::set_ipv6_addr(&tun_name, interface_addresses.ipv6)
                .map_err(Error::SetTunDeviceIpv6Addr)?;
        }

        Ok(tun_device)
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
                NextTunnelState::NewState(DisconnectingState::enter(ActionAfterDisconnect::Nothing, None, shared_state))
            }
            result = &mut self.connect_fut => {
                Self::on_connect_mixnet_client(result, shared_state).await
            }
            Some(command) = command_rx.recv() => {
                match command {
                    TunnelCommand::Connect => NextTunnelState::SameState(self),
                    TunnelCommand::Disconnect => {
                        NextTunnelState::NewState(DisconnectingState::enter(ActionAfterDisconnect::Nothing, None, shared_state))
                    },
                }
            }
            else => NextTunnelState::Finished
        }
    }
}
