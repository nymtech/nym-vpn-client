// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>

use crate::config::WireguardConfig;
use crate::error::Result;
use crate::gateway_client::{Config, GatewayClient};
use crate::mixnet_connect::setup_mixnet_client;
use crate::mixnet_processor::IpPacketRouterAddress;
use crate::tunnel::{setup_route_manager, start_tunnel, Tunnel};
use crate::util::{handle_interrupt, wait_for_interrupt};
use futures::channel::oneshot;
use log::*;
use nym_task::TaskManager;
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;

pub mod config;
pub mod error;
pub mod gateway_client;
pub mod mixnet_connect;
pub mod mixnet_processor;
pub mod routing;
pub mod tunnel;
mod util;

pub async fn init_wireguard_config(
    gateway_client: &GatewayClient,
    entry_gateway_identity: &str,
    wireguard_private_key: &str,
) -> Result<WireguardConfig> {
    // First we need to register with the gateway to setup keys and IP assignment
    info!("Registering with wireguard gateway");
    let wg_gateway_data = gateway_client
        .register_wireguard(entry_gateway_identity)
        .await?;
    debug!("Received wireguard gateway data: {wg_gateway_data:?}");

    let wireguard_config = WireguardConfig::init(wireguard_private_key, &wg_gateway_data)?;
    info!("Wireguard config: \n{wireguard_config}");
    Ok(wireguard_config)
}

pub struct NymVPN {
    /// Gateway configuration
    pub gateway_config: Config,

    /// Enable the wireguard traffic between the client and the entry gateway.
    pub enable_wireguard: bool,

    /// Path to the data directory of a previously initialised mixnet client, where the keys reside.
    pub mixnet_client_path: Option<PathBuf>,

    /// Mixnet public ID of the entry gateway.
    pub entry_gateway: String,

    /// Mixnet recipient address.
    pub exit_router: String,

    /// Associated private key.
    pub private_key: Option<String>,

    /// The IP address of the TUN device.
    pub ip: Option<Ipv4Addr>,

    /// Disable routing all traffic through the VPN TUN device.
    pub disable_routing: bool,

    /// The MTU of the TUN device.
    pub mtu: Option<i32>,
}

impl NymVPN {
    pub async fn run(&self) -> Result<()> {
        // Create a gateway client that we use to interact with the entry gateway, in particular to
        // handle wireguard registration
        let gateway_client = GatewayClient::new(self.gateway_config.clone())?;

        let wireguard_config = if self.enable_wireguard {
            let private_key = self
                .private_key
                .as_ref()
                .expect("clap should enforce value when wireguard enabled");
            let wireguard_config =
                init_wireguard_config(&gateway_client, &self.entry_gateway, private_key).await?;
            Some(wireguard_config)
        } else {
            None
        };

        // The IP address of the gateway inside the tunnel. This will depend on if wireguard is
        // enabled
        let tunnel_gateway_ip = routing::TunnelGatewayIp::new(wireguard_config.clone());
        info!("tunnel_gateway_ip: {tunnel_gateway_ip}");

        // Get the IP address of the local LAN gateway
        let default_lan_gateway_ip = routing::LanGatewayIp::get_default_interface()?;
        info!("default_lane_gateway: {default_lan_gateway_ip}");

        // The address of the ip packet router running on the exit gateway
        let exit_router = IpPacketRouterAddress::try_from_base58_string(&self.exit_router)?;
        info!("exit_router: {exit_router}");

        let task_manager = TaskManager::new(10);

        info!("Setting up route manager");
        let mut route_manager = setup_route_manager().await?;

        // let route_manager_handle = route_manager.handle()?;
        let (tunnel_close_tx, tunnel_close_rx) = oneshot::channel();

        info!("Creating tunnel");
        let mut tunnel = Tunnel::new(wireguard_config.clone(), route_manager.handle()?)?;

        let wireguard_waiting = if self.enable_wireguard {
            info!("Starting wireguard tunnel");
            let (finished_shutdown_tx, finished_shutdown_rx) = oneshot::channel();
            let tunnel_handle = start_tunnel(&tunnel, tunnel_close_rx, finished_shutdown_tx)?;
            Some((finished_shutdown_rx, tunnel_handle))
        } else {
            info!("Wireguard is disabled");
            None
        };

        info!("Setting up mixnet client");
        let mut mixnet_client = setup_mixnet_client(
            &self.entry_gateway,
            &self.mixnet_client_path,
            task_manager.subscribe_named("mixnet_client_main"),
            self.enable_wireguard,
        )
        .await?;

        info!("Connecting to IP packet router");
        let ip = match mixnet_connect::connect_to_ip_packet_router(
            &mut mixnet_client,
            exit_router,
            self.ip,
        )
        .await
        {
            Ok(ip) => {
                info!("Connected to IP packet router on the exit gateway!");
                info!("Using IP address: {ip}");
                ip
            }
            Err(err) => {
                // TODO: we should handle shutdown gracefully in all cases, not just this one.
                error!("Failed to connect to IP packet router: {err}");
                debug!("{err:?}");
                mixnet_client.disconnect().await;
                wait_for_interrupt(task_manager).await;
                handle_interrupt(route_manager, wireguard_waiting, tunnel_close_tx).await?;
                return Err(err);
            }
        };
        info!("Connected to IP packet router on the exit gateway!");

        // We need the IP of the gateway to correctly configure the routing table
        let gateway_used = mixnet_client.nym_address().gateway().to_base58_string();
        info!("Using gateway: {gateway_used}");
        let entry_mixnet_gateway_ip: IpAddr =
            gateway_client.lookup_gateway_ip(&gateway_used).await?;
        debug!("Gateway ip resolves to: {entry_mixnet_gateway_ip}");

        info!("Setting up routing");
        let routing_config = routing::RoutingConfig::new(
            ip,
            entry_mixnet_gateway_ip,
            default_lan_gateway_ip,
            tunnel_gateway_ip,
            self.mtu,
        );
        debug!("Routing config: {:#?}", routing_config);
        let mixnet_tun_dev = routing::setup_routing(
            &mut route_manager,
            routing_config,
            self.enable_wireguard,
            self.disable_routing,
        )
        .await?;

        info!("Setting up mixnet processor");
        let processor_config = mixnet_processor::Config::new(exit_router);
        debug!("Mixnet processor config: {:#?}", processor_config);
        mixnet_processor::start_processor(
            processor_config,
            mixnet_tun_dev,
            mixnet_client,
            &task_manager,
        )
        .await?;

        // Finished starting everything, now wait for shutdown
        wait_for_interrupt(task_manager).await;
        handle_interrupt(route_manager, wireguard_waiting, tunnel_close_tx).await?;

        tunnel.dns_monitor.reset()?;
        tunnel.firewall.reset_policy()?;

        Ok(())
    }
}
