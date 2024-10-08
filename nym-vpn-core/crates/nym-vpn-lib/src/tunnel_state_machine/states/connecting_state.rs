// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

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
use crate::tunnel_state_machine::default_interface::DefaultInterface;

use crate::tunnel_state_machine::{
    route_handler::RoutingConfig,
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

        let tun_device = Self::create_mixnet_device(interface_addresses, mtu, enable_ipv6)?;
        let tun_name = tun_device
            .get_ref()
            .name()
            .map_err(Error::GetTunDeviceName)?;

        tracing::debug!("Created tun device: {}", tun_name);

        let routing_config = RoutingConfig::Mixnet {
            enable_ipv6,
            tun_name: tun_name.clone(),
            entry_gateway_address: connected_tunnel.entry_mixnet_gateway_ip(),
            #[cfg(target_os = "linux")]
            physical_interface: DefaultInterface::current()?,
        };

        Self::set_routes(routing_config, shared_state).await?;
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

        let entry_tun = Self::create_wireguard_device(
            conn_data.entry.private_ipv4,
            None,
            connected_tunnel.entry_mtu(),
        )?;
        let entry_tun_name = entry_tun
            .get_ref()
            .name()
            .map_err(Error::GetTunDeviceName)?;
        tracing::info!("Created entry tun device: {}", entry_tun_name);

        let exit_tun = Self::create_wireguard_device(
            conn_data.exit.private_ipv4,
            Some(conn_data.entry.private_ipv4),
            connected_tunnel.exit_mtu(),
        )?;
        let exit_tun_name = exit_tun.get_ref().name().map_err(Error::GetTunDeviceName)?;
        tracing::info!("Created exit tun device: {}", exit_tun_name);

        let routing_config = RoutingConfig::Wireguard {
            enable_ipv6,
            entry_tun_name,
            exit_tun_name: exit_tun_name.clone(),
            entry_gateway_address: conn_data.entry.endpoint.ip(),
            exit_gateway_address: conn_data.exit.endpoint.ip(),
            #[cfg(target_os = "linux")]
            physical_interface: DefaultInterface::current()?,
        };

        Self::set_routes(routing_config, shared_state).await?;
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

    async fn set_routes(
        routing_config: RoutingConfig,
        shared_state: &mut SharedState,
    ) -> Result<()> {
        shared_state
            .route_handler
            .add_routes(routing_config)
            .await
            .map_err(Error::AddRoutes)
    }

    fn create_mixnet_device(
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

    fn create_wireguard_device(
        interface_addr: Ipv4Addr,
        destination: Option<Ipv4Addr>,
        mtu: u16,
    ) -> Result<AsyncDevice> {
        let mut tun_config = tun::Configuration::default();

        tun_config
            .address(interface_addr)
            .netmask(Ipv4Addr::BROADCAST)
            .mtu(i32::from(mtu))
            .up();

        if let Some(destination) = destination {
            tun_config.destination(destination);
        }

        #[cfg(target_os = "linux")]
        tun_config.platform(|platform_config| {
            platform_config.packet_information(false);
        });

        tun::create_as_async(&tun_config).map_err(Error::CreateTunDevice)
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
