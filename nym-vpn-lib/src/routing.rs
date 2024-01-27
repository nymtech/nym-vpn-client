// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use default_net::Interface;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::process::Command;
use std::{collections::HashSet, net::IpAddr};

use default_net::interface::get_default_interface;
use ipnetwork::IpNetwork;
use talpid_routing::{Node, RequiredRoute, RouteManager};
use tap::TapFallible;
use tracing::{debug, error, info, trace};
use tun::Device;

use crate::config::WireguardConfig;
use crate::error::Result;

const DEFAULT_TUN_MTU: i32 = 1500;

#[derive(Debug)]
pub struct RoutingConfig {
    mixnet_tun_config: tun::Configuration,
    entry_mixnet_gateway_ip: IpAddr,
    lan_gateway_ip: LanGatewayIp,
    tunnel_gateway_ip: TunnelGatewayIp,
}

impl RoutingConfig {
    pub fn new(
        tun_ip: IpAddr,
        entry_mixnet_gateway_ip: IpAddr,
        lan_gateway_ip: LanGatewayIp,
        tunnel_gateway_ip: TunnelGatewayIp,
        mtu: Option<i32>,
    ) -> Self {
        debug!("TUN device IP: {}", tun_ip);
        let mut mixnet_tun_config = tun::Configuration::default();
        mixnet_tun_config.address(tun_ip);
        mixnet_tun_config.mtu(mtu.unwrap_or(DEFAULT_TUN_MTU));
        mixnet_tun_config.up();

        Self {
            mixnet_tun_config,
            entry_mixnet_gateway_ip,
            lan_gateway_ip,
            tunnel_gateway_ip,
        }
    }
}

#[derive(Debug)]
pub struct TunnelGatewayIp {
    pub ipv4: Ipv4Addr,
    pub ipv6: Option<Ipv6Addr>,
}

impl TunnelGatewayIp {
    pub fn new(wireguard_config: Option<WireguardConfig>) -> Self {
        let ipv4 = wireguard_config
            .as_ref()
            .map(|c| c.0.ipv4_gateway)
            .unwrap_or(Ipv4Addr::new(10, 1, 0, 1));
        let ipv6 = wireguard_config
            .as_ref()
            .map(|c| c.0.ipv6_gateway)
            .unwrap_or(None);
        Self { ipv4, ipv6 }
    }
}

impl std::fmt::Display for TunnelGatewayIp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ipv6) = &self.ipv6 {
            write!(f, "ipv4: {}, ipv6: {}", self.ipv4, ipv6)
        } else {
            write!(f, "ipv4: {}", self.ipv4)
        }
    }
}

#[derive(Debug)]
pub struct LanGatewayIp(pub Interface);

impl LanGatewayIp {
    pub fn get_default_interface() -> Result<Self> {
        trace!("Getting default interface");
        let default_interface = get_default_interface().map_err(|err| {
            error!("Failed to get default interface: {}", err);
            crate::error::Error::DefaultInterfaceError
        })?;
        info!("Default interface: {}", default_interface.name);
        debug!("Default interface: {:?}", default_interface);
        if default_interface.gateway.is_none() {
            error!(
                "The default interface `{}` reports no gateway",
                default_interface.name
            );
            Err(crate::error::Error::DefaultInterfaceGatewayError(
                default_interface.name,
            ))
        } else {
            Ok(LanGatewayIp(default_interface))
        }
    }
}

impl std::fmt::Display for LanGatewayIp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
fn get_tunnel_nodes(
    iface_name: &str,
    ipv4_gateway: Ipv4Addr,
    ipv6_gateway: Option<Ipv6Addr>,
) -> (Node, Node) {
    #[cfg(windows)]
    {
        let v4 = Node::new(ipv4_gateway.into(), iface_name.to_string());
        let v6 = if let Some(ipv6_gateway) = ipv6_gateway.as_ref() {
            Node::new((*ipv6_gateway).into(), iface_name.to_string())
        } else {
            Node::device(iface_name.to_string())
        };
        (v4, v6)
    }

    #[cfg(not(windows))]
    {
        let node = Node::device(iface_name.to_string());
        (node.clone(), node)
    }
}

/// Replace default (0-prefix) routes with more specific routes.
fn replace_default_prefixes(network: IpNetwork) -> Vec<IpNetwork> {
    #[cfg(not(target_os = "linux"))]
    if network.prefix() == 0 {
        if network.is_ipv4() {
            vec!["0.0.0.0/1".parse().unwrap(), "128.0.0.0/1".parse().unwrap()]
        } else {
            vec!["8000::/1".parse().unwrap(), "::/1".parse().unwrap()]
        }
    } else {
        vec![network]
    }

    #[cfg(target_os = "linux")]
    vec![network]
}

pub async fn setup_routing(
    route_manager: &mut RouteManager,
    config: RoutingConfig,
    enable_wireguard: bool,
    disable_routing: bool,
) -> Result<tun::AsyncDevice> {
    info!("Creating tun device");
    let dev = tun::create_as_async(&config.mixnet_tun_config)
        .tap_err(|err| error!("Failed to create tun device: {}", err))?;
    let device_name = dev.get_ref().name().unwrap().to_string();
    info!(
        "Created tun device {device_name} with ip={device_ip:?}",
        device_name = device_name,
        device_ip = dev
            .get_ref()
            .address()
            .map(|ip| ip.to_string())
            .unwrap_or("None".to_string())
    );
    debug!("Created tun device {device_name}: ip={device_ip:?}, broadcast={device_broadcast:?}, netmask={device_netmask:?}, destination={device_destination:?}, mtu={device_mtu:?}",
        device_name = device_name,
        device_ip = dev.get_ref().address(),
        device_broadcast = dev.get_ref().broadcast(),
        device_netmask = dev.get_ref().netmask(),
        device_destination = dev.get_ref().destination(),
        device_mtu = dev.get_ref().mtu(),
    );

    #[cfg(target_os = "linux")]
    Command::new("ip")
        .args([
            "-6",
            "addr",
            "add",
            "fda7:576d:ac1a::1/48",
            "dev",
            &device_name,
        ])
        .output()?;

    #[cfg(target_os = "macos")]
    Command::new("ifconfig")
        .args([&device_name, "inet6", "add", "fda7:576d:ac1a::1/48"])
        .output()?;

    if disable_routing {
        info!("Routing is disabled, skipping adding routes");
        return Ok(dev);
    }

    let (node_v4, node_v6) = get_tunnel_nodes(
        &device_name,
        config.tunnel_gateway_ip.ipv4,
        config.tunnel_gateway_ip.ipv6,
    );
    info!("Using node_v4: {:?}", node_v4);
    info!("Using node_v6: {:?}", node_v6);

    let mut routes = [
        ("0.0.0.0/0".to_string(), node_v4),
        ("::/0".to_string(), node_v6),
    ]
    .to_vec();

    // If wireguard is not enabled, and we are not tunneling the connection to the gateway through
    // it, we need to add an exception route for the gateway to the routing table.
    if !enable_wireguard || cfg!(target_os = "linux") {
        let entry_mixnet_gateway_ip = config.entry_mixnet_gateway_ip.to_string();
        let default_node = Node::new(
            config
                .lan_gateway_ip
                .0
                .gateway
                .expect("This value was already checked to exist")
                .ip_addr,
            config.lan_gateway_ip.0.name,
        );
        info!(
            "Add extra route: [{:?}, {:?}]",
            entry_mixnet_gateway_ip,
            default_node.clone()
        );
        routes.extend([(entry_mixnet_gateway_ip, default_node.clone())]);
    };

    let routes = routes.into_iter().flat_map(|(network, node)| {
        replace_default_prefixes(network.parse().unwrap())
            .into_iter()
            .map(move |ip| RequiredRoute::new(ip, node.clone()))
    });
    #[cfg(target_os = "linux")]
    let routes = routes.map(|route| route.use_main_table(false));

    info!("Adding routes to route manager");
    debug!("Routes: {:#?}", routes.clone().collect::<HashSet<_>>());
    route_manager.add_routes(routes.collect()).await?;

    Ok(dev)
}
