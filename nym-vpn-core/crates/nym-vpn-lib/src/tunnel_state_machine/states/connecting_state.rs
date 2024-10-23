// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use std::net::Ipv4Addr;

use futures::{
    future::{BoxFuture, Fuse},
    FutureExt,
};
#[cfg(any(target_os = "ios", target_os = "android"))]
use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
use nym_gateway_directory::GatewayMinPerformance;
use time::OffsetDateTime;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tun::AsyncDevice;
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use tun::Device;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use nym_ip_packet_requests::IpPair;

#[cfg(any(target_os = "ios", target_os = "android"))]
use crate::tunnel_provider;
#[cfg(target_os = "linux")]
use crate::tunnel_state_machine::default_interface::DefaultInterface;
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use crate::tunnel_state_machine::{route_handler::RoutingConfig, tun_ipv6};
use crate::tunnel_state_machine::{
    states::{ConnectedState, DisconnectingState},
    tunnel::{self, any_tunnel_handle::AnyTunnelHandle, ConnectedMixnet, MixnetConnectOptions},
    ActionAfterDisconnect, ConnectionData, Error, MixnetConnectionData, NextTunnelState, Result,
    SharedState, TunnelCommand, TunnelConnectionData, TunnelState, TunnelStateHandler, TunnelType,
    WireguardConnectionData, WireguardNode,
};

/// Default MTU for mixnet tun device.
#[cfg(not(all(target_os = "ios", target_os = "android")))]
const DEFAULT_TUN_MTU: u16 = if cfg!(any(target_os = "ios", target_os = "android")) {
    1280
} else {
    1500
};

type ConnectFut = BoxFuture<'static, tunnel::Result<ConnectedMixnet>>;

pub struct ConnectingState {
    connect_fut: Fuse<ConnectFut>,
}

impl ConnectingState {
    pub fn enter(shared_state: &mut SharedState) -> (Box<dyn TunnelStateHandler>, TunnelState) {
        let gateway_performance_options = shared_state.tunnel_settings.gateway_performance_options;

        let gateway_min_performance = GatewayMinPerformance::from_percentage_values(
            gateway_performance_options
                .mixnet_min_performance
                .map(u64::from),
            gateway_performance_options
                .vpn_min_performance
                .map(u64::from),
        );

        let mut gateway_config = shared_state.nym_config.gateway_config.clone();
        match gateway_min_performance {
            Ok(gateway_min_performance) => {
                gateway_config =
                    gateway_config.with_min_gateway_performance(gateway_min_performance);
            }
            Err(e) => {
                tracing::error!(
                    "Invalid gateway performance values. Will carry on with initial values. Error: {}"
                , e);
            }
        }

        let connect_options = MixnetConnectOptions {
            data_path: shared_state.nym_config.data_path.clone(),
            gateway_config,
            mixnet_client_config: shared_state.tunnel_settings.mixnet_client_config.clone(),
            tunnel_type: shared_state.tunnel_settings.tunnel_type,
            entry_point: shared_state.tunnel_settings.entry_point.clone(),
            exit_point: shared_state.tunnel_settings.exit_point.clone(),
            user_agent: None, // todo: provide user-agent
        };

        let connect_fut = tunnel::connect_mixnet(connect_options).boxed().fuse();

        (Box::new(Self { connect_fut }), TunnelState::Connecting)
    }

    async fn on_connect_mixnet_client(
        result: tunnel::Result<ConnectedMixnet>,
        shared_state: &mut SharedState,
    ) -> NextTunnelState {
        match result.map_err(Error::ConnectMixnetClient) {
            Ok(mut connected_mixnet) => {
                shared_state.status_listener_handle = Some(
                    connected_mixnet
                        .start_event_listener(shared_state.mixnet_event_sender.clone())
                        .await,
                );

                match Self::start_tunnel(connected_mixnet, shared_state).await {
                    Ok((conn_data, tunnel_handle)) => NextTunnelState::NewState(
                        ConnectedState::enter(conn_data, tunnel_handle, shared_state),
                    ),
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
    ) -> Result<(ConnectionData, AnyTunnelHandle)> {
        let selected_gateways = connected_mixnet.selected_gateways().clone();
        let (tunnel_conn_data, tunnel_handle) = match shared_state.tunnel_settings.tunnel_type {
            TunnelType::Mixnet => Self::start_mixnet_tunnel(connected_mixnet, shared_state).await?,
            TunnelType::Wireguard => {
                Self::start_wireguard_tunnel(connected_mixnet, shared_state).await?
            }
        };

        let conn_data = ConnectionData {
            entry_gateway: Box::new(*selected_gateways.entry.identity()),
            exit_gateway: Box::new(*selected_gateways.exit.identity()),
            connected_at: OffsetDateTime::now_utc(),
            tunnel: tunnel_conn_data,
        };

        Ok((conn_data, tunnel_handle))
    }

    async fn start_mixnet_tunnel(
        connected_mixnet: ConnectedMixnet,
        shared_state: &mut SharedState,
    ) -> Result<(TunnelConnectionData, AnyTunnelHandle)> {
        let interface_addrs = shared_state
            .tunnel_settings
            .mixnet_tunnel_options
            .interface_addrs;

        let connected_tunnel = connected_mixnet
            .connect_mixnet_tunnel(interface_addrs)
            .await
            .map_err(Error::ConnectMixnetTunnel)?;
        let assigned_addresses = connected_tunnel.assigned_addresses();

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        let enable_ipv6 = true;
        let mtu: u16 = shared_state
            .tunnel_settings
            .mixnet_tunnel_options
            .mtu
            .unwrap_or(DEFAULT_TUN_MTU);

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        let tun_device =
            Self::create_mixnet_device(assigned_addresses.interface_addresses, mtu, enable_ipv6)?;

        #[cfg(any(target_os = "ios", target_os = "android"))]
        let tun_device = {
            let packet_tunnel_settings = tunnel_provider::tunnel_settings::TunnelSettings {
                dns_servers: shared_state.tunnel_settings.dns.ip_addresses().to_vec(),
                interface_addresses: vec![
                    IpNetwork::V4(
                        Ipv4Network::new(assigned_addresses.interface_addresses.ipv4, 32)
                            .expect("ipv4/32 to ipnetwork"),
                    ),
                    IpNetwork::V6(
                        Ipv6Network::new(assigned_addresses.interface_addresses.ipv6, 128)
                            .expect("ipv6/128 addr to ipnetwork"),
                    ),
                ],
                remote_addresses: vec![assigned_addresses.entry_mixnet_gateway_ip],
                mtu,
            };

            let tun_device = Self::create_tun_device(packet_tunnel_settings, shared_state).await?;
            tracing::debug!("Created tun device");
            tun_device
        };

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        {
            let tun_name = tun_device
                .get_ref()
                .name()
                .map_err(Error::GetTunDeviceName)?;

            tracing::debug!("Created tun device: {}", tun_name);

            let routing_config = RoutingConfig::Mixnet {
                enable_ipv6,
                tun_name: tun_name.clone(),
                entry_gateway_address: assigned_addresses.entry_mixnet_gateway_ip,
                #[cfg(target_os = "linux")]
                physical_interface: DefaultInterface::current()?,
            };

            Self::set_routes(routing_config, shared_state).await?;
            Self::set_dns(&tun_name, shared_state)?;
        }

        let tunnel_conn_data = TunnelConnectionData::Mixnet(MixnetConnectionData {
            nym_address: Box::new(assigned_addresses.mixnet_client_address),
            exit_ipr: Box::new(assigned_addresses.exit_mix_addresses.0),
            ipv4: assigned_addresses.interface_addresses.ipv4,
            ipv6: assigned_addresses.interface_addresses.ipv6,
        });

        let tunnel_handle = AnyTunnelHandle::from(connected_tunnel.run(tun_device).await);

        Ok((tunnel_conn_data, tunnel_handle))
    }

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    async fn start_wireguard_tunnel(
        connected_mixnet: ConnectedMixnet,
        shared_state: &mut SharedState,
    ) -> Result<(TunnelConnectionData, AnyTunnelHandle)> {
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

        let tunnel_conn_data = TunnelConnectionData::Wireguard(WireguardConnectionData {
            entry: WireguardNode::from(conn_data.entry.clone()),
            exit: WireguardNode::from(conn_data.exit.clone()),
        });

        let tunnel_handle = connected_tunnel
            .run(
                entry_tun,
                exit_tun,
                shared_state.tunnel_settings.dns.ip_addresses().to_vec(),
            )
            .map_err(Error::RunWireguardTunnel)?;

        let any_tunnel_handle = AnyTunnelHandle::from(tunnel_handle);

        Ok((tunnel_conn_data, any_tunnel_handle))
    }

    #[cfg(any(target_os = "ios", target_os = "android"))]
    async fn start_wireguard_tunnel(
        connected_mixnet: ConnectedMixnet,
        shared_state: &mut SharedState,
    ) -> Result<(TunnelConnectionData, AnyTunnelHandle)> {
        let connected_tunnel = connected_mixnet
            .connect_wireguard_tunnel()
            .await
            .map_err(Error::ConnectWireguardTunnel)?;

        let conn_data = connected_tunnel.connection_data();

        let packet_tunnel_settings = tunnel_provider::tunnel_settings::TunnelSettings {
            dns_servers: shared_state.tunnel_settings.dns.ip_addresses().to_vec(),
            interface_addresses: vec![IpNetwork::V4(
                Ipv4Network::new(conn_data.entry.private_ipv4, 32).expect("ipv4 to ipnetwork/32"),
            )],
            remote_addresses: vec![conn_data.entry.endpoint.ip()],
            mtu: connected_tunnel.exit_mtu(),
        };

        let tun_device = Self::create_tun_device(packet_tunnel_settings, shared_state).await?;

        tracing::info!("Created tun device");

        let tunnel_conn_data = TunnelConnectionData::Wireguard(WireguardConnectionData {
            entry: WireguardNode::from(conn_data.entry.clone()),
            exit: WireguardNode::from(conn_data.exit.clone()),
        });

        let tunnel_handle = connected_tunnel
            .run(
                tun_device,
                shared_state.tunnel_settings.dns.ip_addresses().to_vec(),
                #[cfg(any(target_os = "ios", target_os = "android"))]
                shared_state.tun_provider.clone(),
            )
            .map_err(Error::RunWireguardTunnel)?;

        let any_tunnel_handle = AnyTunnelHandle::from(tunnel_handle);

        Ok((tunnel_conn_data, any_tunnel_handle))
    }

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    fn set_dns(tun_name: &str, shared_state: &mut SharedState) -> Result<()> {
        let dns_servers = shared_state.tunnel_settings.dns.ip_addresses();

        shared_state
            .dns_handler
            .set(tun_name, dns_servers)
            .map_err(Error::SetDns)
    }

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    async fn set_routes(
        routing_config: RoutingConfig,
        shared_state: &mut SharedState,
    ) -> Result<()> {
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        shared_state
            .route_handler
            .add_routes(routing_config)
            .await
            .map_err(Error::AddRoutes)?;

        Ok(())
    }

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
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

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
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

    #[cfg(any(target_os = "ios", target_os = "android"))]
    async fn create_tun_device(
        packet_tunnel_settings: tunnel_provider::tunnel_settings::TunnelSettings,
        shared_state: &mut SharedState,
    ) -> Result<AsyncDevice> {
        let mut tun_config = tun::Configuration::default();

        #[cfg(target_os = "android")]
        {
            let tun_fd = shared_state
                .tun_provider
                .configure_tunnel(packet_tunnel_settings.into_tunnel_network_settings())
                .map_err(|e| Error::ConfigureTunnelProvider(e.to_string()))?;

            tun_config.raw_fd(tun_fd);
            #[cfg(target_os = "linux")]
            tun_config.platform(|platform_config| {
                platform_config.packet_information(false);
            });
        }

        #[cfg(target_os = "ios")]
        {
            let tun_fd =
                tunnel_provider::ios::interface::get_tun_fd().ok_or(Error::LocateTunDevice)?;
            tun_config.raw_fd(tun_fd);

            shared_state
                .tun_provider
                .set_tunnel_network_settings(packet_tunnel_settings.into_tunnel_network_settings())
                .await
                .map_err(|e| Error::ConfigureTunnelProvider(e.to_string()))?
        }

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
                    TunnelCommand::SetTunnelSettings(tunnel_settings) => {
                        if shared_state.tunnel_settings == tunnel_settings {
                            NextTunnelState::SameState(self)
                        } else {
                            shared_state.tunnel_settings = tunnel_settings;
                            NextTunnelState::NewState(DisconnectingState::enter(ActionAfterDisconnect::Reconnect, None, shared_state))
                        }
                    }
                }
            }
            else => NextTunnelState::Finished
        }
    }
}
