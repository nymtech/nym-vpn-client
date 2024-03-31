// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use netdev::Interface;
use std::fmt::{Display, Formatter};
use std::net::{Ipv4Addr, Ipv6Addr};
#[cfg(target_os = "android")]
use std::os::fd::{AsRawFd, RawFd};
#[cfg(target_os = "android")]
use std::sync::{Arc, Mutex};
use std::{collections::HashSet, net::IpAddr};

use ipnetwork::IpNetwork;
use netdev::interface::get_default_interface;
use nym_ip_packet_requests::IpPair;
use talpid_routing::{Node, RequiredRoute, RouteManager};
#[cfg(target_os = "android")]
use talpid_tunnel::tun_provider::TunProvider;
use tap::TapFallible;
use tokio::runtime::Runtime;
use tracing::{debug, error, info, trace};
use tun2::AbstractDevice;

use crate::config::WireguardConfig;
use crate::error::Result;
use crate::NymVpn;

const DEFAULT_TUN_MTU: u16 = 1500;

#[derive(Clone)]
pub struct RoutingConfig {
    pub(crate) mixnet_tun_config: tun2::Configuration,
    // In case we need them, as they're not read-accessible in the tun2 config
    pub(crate) tun_ips: IpPair,
    pub(crate) mtu: u16,

    pub(crate) entry_mixnet_gateway_ip: IpAddr,
    pub(crate) lan_gateway_ip: LanGatewayIp,
    pub(crate) tunnel_gateway_ip: TunnelGatewayIp,
    pub(crate) enable_wireguard: bool,
    pub(crate) disable_routing: bool,
    #[cfg(target_os = "android")]
    pub(crate) gateway_ws_fd: Option<RawFd>,
    #[cfg(target_os = "android")]
    pub(crate) tun_provider: Arc<Mutex<TunProvider>>,
}

impl Display for RoutingConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "mixnet_tun_config: {:?}\ntun_ips: {:?}\nmtu: {}\nentry_mixnet_gateway_ip: {:?}\nlan_gateway_ip: {:?}\ntunnel_gateway_ip: {:?}\nenable_wireguard: {:?}\ndisable_routing: {:?}",
            self.mixnet_tun_config,
            self.tun_ips,
            self.mtu,
            self.entry_mixnet_gateway_ip,
            self.lan_gateway_ip,
            self.tunnel_gateway_ip,
            self.enable_wireguard,
            self.disable_routing
        )
    }
}

impl RoutingConfig {
    pub fn new(
        vpn: &NymVpn,
        tun_ips: IpPair,
        entry_mixnet_gateway_ip: IpAddr,
        lan_gateway_ip: LanGatewayIp,
        tunnel_gateway_ip: TunnelGatewayIp,
        #[cfg(target_os = "android")] gateway_ws_fd: Option<RawFd>,
    ) -> Self {
        debug!("TUN device IPs: {}", tun_ips);
        let mut mixnet_tun_config = tun2::Configuration::default();
        let mtu = vpn.nym_mtu.unwrap_or(DEFAULT_TUN_MTU);
        // only IPv4 is supported by tun2 for now
        mixnet_tun_config.address(tun_ips.ipv4);
        mixnet_tun_config.mtu(mtu);
        mixnet_tun_config.up();

        #[cfg(target_os = "linux")]
        mixnet_tun_config.platform_config(|config| {
            config.ensure_root_privileges(true);
        });

        Self {
            mixnet_tun_config,
            tun_ips,
            mtu,
            entry_mixnet_gateway_ip,
            lan_gateway_ip,
            tunnel_gateway_ip,
            enable_wireguard: vpn.enable_wireguard,
            disable_routing: vpn.disable_routing,
            #[cfg(target_os = "android")]
            gateway_ws_fd,
            #[cfg(target_os = "android")]
            tun_provider: vpn.tun_provider.clone(),
        }
    }

    pub fn tun_ips(&self) -> IpPair {
        self.tun_ips
    }

    pub fn mtu(&self) -> u16 {
        self.mtu
    }

    pub fn entry_mixnet_gateway_ip(&self) -> IpAddr {
        self.entry_mixnet_gateway_ip
    }

    pub fn enable_wireguard(&self) -> bool {
        self.enable_wireguard
    }
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct LanGatewayIp(pub Interface);

impl LanGatewayIp {
    pub fn get_default_interface() -> Result<Self> {
        trace!("Getting default interface");
        let default_interface = get_default_interface().map_err(|err| {
            error!("Failed to get default interface: {}", err);
            crate::error::Error::DefaultInterfaceError
        })?;
        info!("Default network interface: {}", default_interface.name);
        debug!("Default network interface: {:?}", default_interface);
        Ok(LanGatewayIp(default_interface))
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

pub fn setup_routing(
    route_manager: &mut RouteManager,
    config: RoutingConfig,
    #[cfg(target_os = "ios")] ios_tun_provider: std::sync::Arc<
        dyn crate::platform::swift::OSTunProvider,
    >,
) -> Result<tun2::AsyncDevice> {
    debug!("Creating tun device");
    let mixnet_tun_config = config.mixnet_tun_config.clone();
    #[cfg(target_os = "android")]
    let mixnet_tun_config = {
        let mut tun_config = talpid_tunnel::tun_provider::TunConfig::default();
        let tun_ips = config.tun_ips();
        tun_config.addresses = vec![tun_ips.ipv4.into(), tun_ips.ipv6.into()];
        let fd = {
            let mut tun_provider = config
                .tun_provider
                .lock()
                .expect("access should not be passed to mullvad yet");

            let fd = tun_provider.get_tun(tun_config)?.as_raw_fd();
            if !config.enable_wireguard {
                if let Some(raw_fd) = config.gateway_ws_fd {
                    tun_provider.bypass(raw_fd)?;
                }
            }
            fd
        };
        let mut mixnet_tun_config = mixnet_tun_config.clone();
        mixnet_tun_config.raw_fd(fd);
        mixnet_tun_config
    };
    #[cfg(target_os = "ios")]
    let mixnet_tun_config = {
        let fd = ios_tun_provider.configure_nym(config.clone().into())?;
        let mut mixnet_tun_config = mixnet_tun_config.clone();
        mixnet_tun_config.raw_fd(fd);
        mixnet_tun_config
    };
    let dev = tun2::create_as_async(&mixnet_tun_config)
        .tap_err(|err| error!("Failed to create tun device: {}", err))?;
    let device_name = dev.as_ref().tun_name().unwrap().to_string();
    info!(
        "Created tun device {device_name} with ip={device_ip:?}",
        device_name = device_name,
        device_ip = dev
            .as_ref()
            .address()
            .map(|ip| ip.to_string())
            .unwrap_or("None".to_string())
    );
    debug!("Created tun device {device_name}: ip={device_ip:?}, broadcast={device_broadcast:?}, netmask={device_netmask:?}, destination={device_destination:?}, mtu={device_mtu:?}",
        device_name = device_name,
        device_ip = dev.as_ref().address(),
        device_broadcast = dev.as_ref().broadcast(),
        device_netmask = dev.as_ref().netmask(),
        device_destination = dev.as_ref().destination(),
        device_mtu = dev.as_ref().mtu(),
    );

    let _ipv6_addr = config.tun_ips.ipv6.to_string();
    #[cfg(target_os = "linux")]
    std::process::Command::new("ip")
        .args(["-6", "addr", "add", &_ipv6_addr, "dev", &device_name])
        .output()?;

    #[cfg(target_os = "macos")]
    std::process::Command::new("ifconfig")
        .args([&device_name, "inet6", "add", &_ipv6_addr])
        .output()?;

    if config.disable_routing {
        info!("Routing is disabled, skipping adding routes");
        return Ok(dev);
    }

    let (node_v4, node_v6) = get_tunnel_nodes(
        &device_name,
        config.tunnel_gateway_ip.ipv4,
        config.tunnel_gateway_ip.ipv6,
    );
    debug!("Using node_v4: {:?}", node_v4);
    debug!("Using node_v6: {:?}", node_v6);

    let mut routes = [
        ("0.0.0.0/0".to_string(), node_v4),
        ("::/0".to_string(), node_v6),
    ]
    .to_vec();

    // If wireguard is not enabled, and we are not tunneling the connection to the gateway through
    // it, we need to add an exception route for the gateway to the routing table.
    //
    // NOTE: On windows it seems like it's not necessary to add the default route.
    // BUG: The name of the device is not correctly set on windows. If this section is to be
    // re-enabled then config.lan_gateway_ip.0.name needs to be set correctly on Windows. The
    // correct one should be something along the lines of "Ethernet" or "Wi-Fi". Check the name
    // with `netsh interface show interfaces`
    if (!config.enable_wireguard && cfg!(not(target_os = "windows"))) || cfg!(target_os = "linux") {
        let entry_mixnet_gateway_ip = config.entry_mixnet_gateway_ip.to_string();
        let default_node = if let Some(addr) = config.lan_gateway_ip.0.gateway.and_then(|g| {
            g.ipv4
                .first()
                .map(|a| IpAddr::from(*a))
                .or(g.ipv6.first().map(|a| IpAddr::from(*a)))
        }) {
            Node::new(addr, config.lan_gateway_ip.0.name)
        } else {
            Node::device(config.lan_gateway_ip.0.name)
        };
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

    let mut rt = Runtime::new().unwrap();

    rt.block_on(route_manager.add_routes(routes.collect()))?;

    Ok(dev)
}
