// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>

use futures::{SinkExt, StreamExt};
use ipnetwork::IpNetwork;
use log::*;
use nym_sdk::mixnet::{IncludedSurbs, MixnetClient, MixnetMessageSender, Recipient};
use nym_task::{TaskClient, TaskManager};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, net::IpAddr};
use talpid_routing::{Node, RequiredRoute, RouteManager};
use tun::{AsyncDevice, Device, TunPacket};

const GATEWAY_ALLOWED_IPS: &str = "10.0.0.2";

#[derive(Debug)]
pub struct Config {
    pub default_node_address: IpAddr,
    pub mixnet_tun_config: tun::Configuration,
    pub entry_mixnet_gateway_ip: IpAddr,
    pub recipient: Recipient,
    pub ipv4_gateway: String,
    pub ipv6_gateway: Option<String>,
}

impl Config {
    pub fn new(
        default_node_address: IpAddr,
        entry_mixnet_gateway_ip: IpAddr,
        recipient: Recipient,
        ipv4_gateway: String,
        ipv6_gateway: Option<String>,
    ) -> Self {
        let mut mixnet_tun_config = tun::Configuration::default();
        mixnet_tun_config.address(GATEWAY_ALLOWED_IPS);
        mixnet_tun_config.up();

        Config {
            default_node_address,
            mixnet_tun_config,
            entry_mixnet_gateway_ip,
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

#[derive(Serialize, Deserialize)]
pub struct TaggedPacket {
    packet: bytes::Bytes,
    return_address: Recipient,
    return_mix_hops: Option<u8>,
}

impl TaggedPacket {
    fn new(packet: bytes::Bytes, return_address: Recipient, return_mix_hops: Option<u8>) -> Self {
        TaggedPacket {
            packet,
            return_address,
            return_mix_hops,
        }
    }
    fn to_tagged_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        use bincode::Options;
        let bincode_serializer = make_bincode_serializer();
        let packet: Vec<u8> = bincode_serializer.serialize(self)?;
        Ok(packet)
    }
}

fn make_bincode_serializer() -> impl bincode::Options {
    use bincode::Options;
    bincode::DefaultOptions::new()
        .with_big_endian()
        .with_varint_encoding()
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
        let (mut sink, mut stream) = self.device.into_framed().split();
        let sender = self.mixnet_client.split_sender();
        let recipient = self.recipient;
        let mut mixnet_stream = self
            .mixnet_client
            .map(|reconstructed_message| Ok(TunPacket::new(reconstructed_message.message.clone())));

        while !shutdown.is_shutdown() {
            tokio::select! {
                _ = shutdown.recv() => {
                    trace!("MixnetProcessor: Received shutdown");
                }
                Some(Ok(packet)) = stream.next() => {
                    // TODO: properly investigate the binary format here and the overheard
                    let Ok(packet) = TaggedPacket::new(packet.into_bytes(), recipient, None).to_tagged_bytes() else {
                        error!("Failed to serialize packet");
                        continue;
                    };

                    // The enum here about IncludedSurbs and ExposeSelfAddress is misleading. It is
                    // not being used. Basically IncludedSurbs::ExposeSelfAddress just omits the
                    // surbs, assuming that it is exposed in side the message. (This is the case
                    // for SOCKS5 too).
                    let ret = sender.send_message(recipient, &packet, IncludedSurbs::ExposeSelfAddress).await;
                    if ret.is_err() {
                        error!("Could not forward IP packet to the mixnet. The packet will be dropped.");
                    }
                }
                res = sink.send_all(&mut mixnet_stream) => {
                    warn!("Mixnet stream finished. This may mean that the gateway was shut down");
                    if let Err(e) = res {
                        error!("Could not forward mixnet traffic to the client - {:?}", e);
                    }
                    break;
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
    task_manager: &TaskManager,
    enable_wireguard: bool,
) -> Result<(), crate::error::Error> {
    let dev = tun::create_as_async(&config.mixnet_tun_config)?;
    let device_name = dev.get_ref().name().to_string();
    info!("Opened tun device {}", device_name);

    let (node_v4, node_v6) =
        get_tunnel_nodes(&device_name, config.ipv4_gateway, config.ipv6_gateway);
    info!("Using node_v4: {:?}", node_v4);
    info!("Using node_v6: {:?}", node_v6);

    let mut routes = [("0.0.0.0/0".to_string(), node_v4), ("::/0".to_string(), node_v6)].to_vec();

    // If wireguard is not enabled, and we are not tunneling the connection to the gateway through
    // it, we need to add an exception route for the gateway to the routing table.
    if !enable_wireguard {
        let entry_mixnet_gateway_ip = config.entry_mixnet_gateway_ip.to_string();
        let default_node = Node::address(config.default_node_address);
        info!("Add extra route: [{:?}, {:?}]", entry_mixnet_gateway_ip, default_node.clone());
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

    info!("Creating mixnet processor");
    let processor = MixnetProcessor::new(dev, mixnet_client, config.recipient);
    let shutdown_listener = task_manager.subscribe();
    tokio::spawn(processor.run(shutdown_listener));
    Ok(())
}
