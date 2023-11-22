use std::{collections::HashSet, net::IpAddr};

use ipnetwork::IpNetwork;
use talpid_routing::{Node, RequiredRoute, RouteManager};
use tracing::{debug, info};
use tun::Device;

const GATEWAY_ALLOWED_IPS: &str = "10.0.0.2";

#[derive(Debug)]
pub struct RoutingConfig {
    mixnet_tun_config: tun::Configuration,
    entry_mixnet_gateway_ip: IpAddr,
    default_node_address: IpAddr,
    ipv4_gateway: String,
    ipv6_gateway: Option<String>,
}

impl RoutingConfig {
    pub fn new(
        entry_mixnet_gateway_ip: IpAddr,
        default_node_address: IpAddr,
        ipv4_gateway: String,
        ipv6_gateway: Option<String>,
    ) -> Self {
        let mut mixnet_tun_config = tun::Configuration::default();
        mixnet_tun_config.address(GATEWAY_ALLOWED_IPS);
        mixnet_tun_config.up();

        Self {
            mixnet_tun_config,
            entry_mixnet_gateway_ip,
            default_node_address,
            ipv4_gateway,
            ipv6_gateway,
        }
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
) -> Result<tun::AsyncDevice, crate::error::Error> {
    let dev = tun::create_as_async(&config.mixnet_tun_config)?;
    let device_name = dev.get_ref().name().to_string();
    info!("Opened tun device {}", device_name);

    let (node_v4, node_v6) = get_tunnel_nodes(
        &device_name,
        config.ipv4_gateway.clone(),
        config.ipv6_gateway.clone(),
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
        let default_node = Node::address(config.default_node_address);
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
