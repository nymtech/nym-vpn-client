use std::{collections::HashSet, net::IpAddr};

use default_net::interface::get_default_interface;
use ipnetwork::IpNetwork;
use talpid_routing::{Node, RequiredRoute, RouteManager};
use tracing::{debug, info};
use tun::Device;

use crate::WireguardConfig;

const GATEWAY_ALLOWED_IPS: &str = "10.0.0.2";

#[derive(Debug)]
pub struct RoutingConfig {
    mixnet_tun_config: tun::Configuration,
    entry_mixnet_gateway_ip: IpAddr,
    lan_gateway_ip: LanGatewayIp,
    tunnel_gateway_ip: TunnelGatewayIp,
}

impl RoutingConfig {
    pub fn new(
        entry_mixnet_gateway_ip: IpAddr,
        lan_gateway_ip: LanGatewayIp,
        tunnel_gateway_ip: TunnelGatewayIp,
    ) -> Self {
        let mut mixnet_tun_config = tun::Configuration::default();
        mixnet_tun_config.address(GATEWAY_ALLOWED_IPS);
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
    pub ipv4: String,
    pub ipv6: Option<String>,
}

impl TunnelGatewayIp {
    pub fn new(wireguard_config: Option<WireguardConfig>) -> Self {
        let ipv4 = wireguard_config
            .as_ref()
            .map(|c| c.0.ipv4_gateway.to_string())
            .unwrap_or("10.1.0.1".to_string());
        let ipv6 = wireguard_config
            .as_ref()
            .map(|c| c.0.ipv6_gateway.map(|ip| ip.to_string()))
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
pub struct LanGatewayIp(pub IpAddr);

impl LanGatewayIp {
    pub fn get_default_interface() -> Result<Self, crate::error::Error> {
        Ok(Self(
            get_default_interface()
                .map_err(|_| crate::error::Error::DefaultInterfaceGatewayError)?
                .gateway
                .map_or(
                    Err(crate::error::Error::DefaultInterfaceGatewayError),
                    |g| Ok(g.ip_addr),
                )?,
        ))
    }
}

impl std::fmt::Display for LanGatewayIp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
fn get_tunnel_nodes(
    iface_name: &str,
    ipv4_gateway: String,
    ipv6_gateway: Option<String>,
) -> (Node, Node) {
    #[cfg(windows)]
    {
        let v4 = routing::Node::new(ipv4_gateway.clone().into(), iface_name.to_string());
        let v6 = if let Some(ipv6_gateway) = ipv6_gateway.as_ref() {
            routing::Node::new(ipv6_gateway.clone().into(), iface_name.to_string())
        } else {
            routing::Node::device(iface_name.to_string())
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
) -> Result<tun::AsyncDevice, crate::error::Error> {
    let dev = tun::create_as_async(&config.mixnet_tun_config)?;
    let device_name = dev.get_ref().name().to_string();
    info!("Opened tun device {}", device_name);

    if disable_routing {
        info!("Routing is disabled, skipping adding routes");
        return Ok(dev);
    }

    let (node_v4, node_v6) = get_tunnel_nodes(
        &device_name,
        config.tunnel_gateway_ip.ipv4.clone(),
        config.tunnel_gateway_ip.ipv6.clone(),
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
    if !enable_wireguard {
        let entry_mixnet_gateway_ip = config.entry_mixnet_gateway_ip.to_string();
        let default_node = Node::address(config.lan_gateway_ip.0);
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
