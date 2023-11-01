// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>

use futures::StreamExt;
use ipnetwork::IpNetwork;
use log::*;
use nym_sdk::mixnet::{IncludedSurbs, MixnetClient, MixnetMessageSender, Recipient};
use nym_task::{TaskClient, TaskManager};
use talpid_routing::{Node, RequiredRoute, RouteManager};
use tun::{AsyncDevice, Device};

pub struct Config {
    pub mixnet_tun_config: tun::Configuration,
    pub recipient: Recipient,
    pub ipv4_gateway: String,
    pub ipv6_gateway: Option<String>,
}

impl Config {
    pub fn new(recipient: Recipient, ipv4_gateway: String, ipv6_gateway: Option<String>) -> Self {
        let mut mixnet_tun_config = tun::Configuration::default();
        mixnet_tun_config.up();

        Config {
            mixnet_tun_config,
            recipient,
            ipv4_gateway,
            ipv6_gateway,
        }
    }
}

pub struct MixnetProcessor {
    device: AsyncDevice,
    mixnet_client: MixnetClient,
    recipient: Recipient,
}

impl MixnetProcessor {
    pub fn new(device: AsyncDevice, mixnet_client: MixnetClient, recipient: Recipient) -> Self {
        MixnetProcessor {
            device,
            mixnet_client,
            recipient,
        }
    }

    pub async fn run(self, mut shutdown: TaskClient) {
        info!(
            "Opened mixnet processor on tun device {}",
            self.device.get_ref().name()
        );
        let mut stream = self.device.into_framed();
        while !shutdown.is_shutdown() {
            tokio::select! {
                _ = shutdown.recv() => {
                    trace!("MixnetProcessor: Received shutdown");
                }
                Some(Ok(packet)) = stream.next() => {
                    let ret = self.mixnet_client.send_message(self.recipient, packet.get_bytes(), IncludedSurbs::ExposeSelfAddress).await;
                    if ret.is_err() {
                        error!("Could not forward datagram to the mixnet. The packet will be dropped.");
                    }
                }
            }
        }
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

pub async fn start_processor(
    config: Config,
    mixnet_client: MixnetClient,
    route_manager: &mut RouteManager,
    shutdown: &TaskManager,
) -> Result<(), crate::error::Error> {
    let dev = tun::create_as_async(&config.mixnet_tun_config)?;
    let device_name = dev.get_ref().name().to_string();
    let (node_v4, node_v6) =
        get_tunnel_nodes(&device_name, config.ipv4_gateway, config.ipv6_gateway);
    let routes = ["0.0.0.0/0", "::/0"]
        .into_iter()
        .flat_map(|network| replace_default_prefixes(network.parse().unwrap()))
        .map(move |allowed_ip| {
            if allowed_ip.is_ipv4() {
                RequiredRoute::new(allowed_ip, node_v4.clone())
            } else {
                RequiredRoute::new(allowed_ip, node_v6.clone())
            }
        });
    #[cfg(target_os = "linux")]
    let routes = routes.map(|route| route.use_main_table(false));
    route_manager.add_routes(routes.collect()).await?;
    let processor = MixnetProcessor::new(dev, mixnet_client, config.recipient);
    let shutdown_listener = shutdown.subscribe();
    tokio::spawn(async move {
        processor.run(shutdown_listener).await;
    });
    Ok(())
}
