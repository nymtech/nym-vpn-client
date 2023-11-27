// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>

mod commands;
mod error;
mod gateway_client;
mod mixnet_processor;
mod routing;
mod tunnel;

use crate::{
    error::Result,
    gateway_client::{Config as GatewayConfig, GatewayData},
    mixnet_processor::IpPacketRouterAddress,
    tunnel::{start_tunnel, Tunnel},
};

use clap::Parser;
use futures::channel::oneshot;
use gateway_client::GatewayClient;
use log::{debug, error, info};
use nym_config::defaults::{setup_env, NymNetworkDetails};
use nym_sdk::mixnet::{MixnetClientBuilder, StoragePaths};
use nym_task::TaskManager;
use std::{collections::HashSet, path::PathBuf};
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use talpid_routing::RouteManager;
use talpid_types::net::wireguard::{
    ConnectionConfig, PeerConfig, PrivateKey, PublicKey, TunnelConfig, TunnelOptions,
};
use talpid_types::net::GenericTunnelOptions;
#[cfg(target_os = "linux")]
use talpid_types::ErrorExt;

pub fn setup_logging() {
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
        .from_env()
        .unwrap()
        .add_directive("hyper::proto=info".parse().unwrap())
        .add_directive("netlink_proto=info".parse().unwrap());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .compact()
        .init();
}

#[derive(Clone)]
pub struct WireguardConfig(pub talpid_wireguard::config::Config);

impl WireguardConfig {
    fn new(
        tunnel: TunnelConfig,
        peers: Vec<PeerConfig>,
        connection_config: &ConnectionConfig,
        wg_options: &TunnelOptions,
        generic_options: &GenericTunnelOptions,
    ) -> Result<Self> {
        Ok(Self(talpid_wireguard::config::Config::new(
            tunnel,
            peers,
            connection_config,
            wg_options,
            generic_options,
            None,
        )?))
    }

    fn init(private_key: &str, gateway_data: &GatewayData) -> Result<Self> {
        let tunnel = TunnelConfig {
            private_key: PrivateKey::from(
                *PublicKey::from_base64(private_key)
                    .map_err(|_| error::Error::InvalidWireGuardKey)?
                    .as_bytes(),
            ),
            addresses: vec![gateway_data.private_ip],
        };
        let peers = vec![PeerConfig {
            public_key: gateway_data.public_key.clone(),
            allowed_ips: vec!["10.1.0.1".parse().unwrap()],
            endpoint: gateway_data.endpoint,
            psk: None,
        }];
        let connection_config = ConnectionConfig {
            tunnel: tunnel.clone(),
            peer: peers[0].clone(),
            exit_peer: None,
            ipv4_gateway: Ipv4Addr::from_str(&gateway_data.private_ip.to_string())?,
            ipv6_gateway: None,
            #[cfg(target_os = "linux")]
            fwmark: None,
        };
        let generic_options = GenericTunnelOptions { enable_ipv6: true };
        let wg_options = TunnelOptions::default();
        let config = Self::new(
            tunnel,
            peers,
            &connection_config,
            &wg_options,
            &generic_options,
        )?;
        Ok(config)
    }
}

impl std::fmt::Display for WireguardConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "tunnel:")?;
        writeln!(f, "  mtu: {}", self.0.mtu)?;
        #[cfg(target_os = "linux")]
        writeln!(f, "  enable_ipv6: {}", self.0.enable_ipv6)?;
        writeln!(f, "  addresses:")?;
        for address in &self.0.tunnel.addresses {
            writeln!(f, "    - {}", address)?;
        }
        writeln!(f, "peers:")?;
        for peer in &self.0.peers {
            writeln!(f, "  - public_key: {}", peer.public_key)?;
            writeln!(f, "    allowed_ips:")?;
            for allowed_ip in &peer.allowed_ips {
                writeln!(f, "      - {}", allowed_ip)?;
            }
            writeln!(f, "    endpoint: {}", peer.endpoint)?;
        }
        writeln!(f, "connection:")?;
        writeln!(f, "  ipv4_gateway: {}", self.0.ipv4_gateway)?;
        if let Some(ipv6_gateway) = &self.0.ipv6_gateway {
            writeln!(f, "  ipv6_gateway: {}", ipv6_gateway)?;
        }
        #[cfg(target_os = "linux")]
        if let Some(fwmark) = &self.0.fwmark {
            writeln!(f, "  fwmark: {}", fwmark)?;
        }
        Ok(())
    }
}

pub async fn setup_route_manager() -> Result<RouteManager> {
    #[cfg(target_os = "linux")]
    let route_manager = {
        let fwmark = 0;
        let table_id = 0;
        RouteManager::new(HashSet::new(), fwmark, table_id).await?
    };

    #[cfg(not(target_os = "linux"))]
    let route_manager = RouteManager::new(HashSet::new()).await?;

    Ok(route_manager)
}

pub async fn setup_mixnet_client(
    mixnet_entry_gateway: &str,
    mixnet_client_key_storage_path: &Option<PathBuf>,
    task_client: nym_task::TaskClient,
    enable_wireguard: bool,
) -> Result<nym_sdk::mixnet::MixnetClient> {
    // Disable Poisson rate limiter by default
    let mut debug_config = nym_client_core::config::DebugConfig::default();
    debug_config
        .traffic
        .disable_main_poisson_packet_distribution = true;

    debug!("mixnet client has wireguard_mode={enable_wireguard}");
    let mixnet_client = if let Some(path) = mixnet_client_key_storage_path {
        debug!("Using custom key storage path: {:?}", path);
        let key_storage_path = StoragePaths::new_from_dir(path)?;
        MixnetClientBuilder::new_with_default_storage(key_storage_path)
            .await?
            .with_wireguard_mode(enable_wireguard)
            .request_gateway(mixnet_entry_gateway.to_string())
            .network_details(NymNetworkDetails::new_from_env())
            .debug_config(debug_config)
            .custom_shutdown(task_client)
            .build()?
            .connect_to_mixnet()
            .await?
    } else {
        debug!("Using ephemeral key storage");
        MixnetClientBuilder::new_ephemeral()
            .with_wireguard_mode(enable_wireguard)
            .request_gateway(mixnet_entry_gateway.to_string())
            .network_details(NymNetworkDetails::new_from_env())
            .debug_config(debug_config)
            .custom_shutdown(task_client)
            .build()?
            .connect_to_mixnet()
            .await?
    };

    Ok(mixnet_client)
}

async fn wait_for_interrupt(task_manager: nym_task::TaskManager) {
    if let Err(e) = task_manager.catch_interrupt().await {
        error!("Could not wait for interrupts anymore - {e}. Shutting down the tunnel.");
    }
}

async fn handle_interrupt(
    mut route_manager: RouteManager,
    wireguard_waiting: Option<(oneshot::Receiver<()>, tokio::task::JoinHandle<Result<()>>)>,
    tunnel_close_tx: oneshot::Sender<()>,
) -> Result<()> {
    let sig_handle = tokio::task::spawn_blocking(move || -> Result<()> {
        debug!("Received interrupt signal");
        route_manager.clear_routes()?;
        #[cfg(target_os = "linux")]
        if let Err(error) =
            tokio::runtime::Handle::current().block_on(route_manager.clear_routing_rules())
        {
            error!(
                "{}",
                error.display_chain_with_msg("Failed to clear routing rules")
            );
        }
        tunnel_close_tx
            .send(())
            .map_err(|_| error::Error::OneshotSendError)?;
        Ok(())
    });

    if let Some((finished_shutdown_rx, tunnel_handle)) = wireguard_waiting {
        tunnel_handle.await??;
        sig_handle.await??;
        finished_shutdown_rx.await?;
    } else {
        sig_handle.await??;
    }
    Ok(())
}

async fn run() -> Result<()> {
    setup_logging();
    let args = commands::CliArgs::parse();
    setup_env(args.config_env_file.as_ref());

    // Setup gateway configuration
    let gateway_config = GatewayConfig::override_from_env(&args, GatewayConfig::default());
    info!("nym-api: {}", gateway_config.api_url);

    // Create a gateway client that we use to interact with the entry gateway, in particular to
    // handle wireguard registeration
    let gateway_client = GatewayClient::new(gateway_config)?;

    let wireguard_config = if args.enable_wireguard {
        // First we need to register with the gateway to setup keys and IP assignment
        info!("Registering with wireguard gateway");
        let wg_gateway_data = gateway_client
            .register_wireguard(&args.entry_gateway)
            .await?;
        debug!("Received wireguard gateway data: {wg_gateway_data:?}");

        // It's ok to unwrap, since clap enforces that this is non-zero when enable_wireguard is
        // true
        let private_key = args.private_key.as_ref().unwrap();
        let wireguard_config = WireguardConfig::init(private_key, &wg_gateway_data)?;
        info!("Wireguard config: \n{wireguard_config}");
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
    let exit_router = IpPacketRouterAddress::try_from_base58_string(&args.exit_router)?;
    info!("exit_router: {exit_router}");

    let task_manager = TaskManager::new(10);

    info!("Setting up route manager");
    let mut route_manager = setup_route_manager().await?;

    // let route_manager_handle = route_manager.handle()?;
    let (tunnel_close_tx, tunnel_close_rx) = oneshot::channel();

    info!("Creating tunnel");
    let mut tunnel = Tunnel::new(wireguard_config, route_manager.handle()?)?;

    let wireguard_waiting = if args.enable_wireguard {
        info!("Starting wireguard tunnel");
        let (finished_shutdown_tx, finished_shutdown_rx) = oneshot::channel();
        let tunnel_handle = start_tunnel(&tunnel, tunnel_close_rx, finished_shutdown_tx)?;
        Some((finished_shutdown_rx, tunnel_handle))
    } else {
        info!("Wireguard is disabled");
        None
    };

    info!("Setting up mixnet client");
    let mixnet_client = setup_mixnet_client(
        &args.entry_gateway,
        &args.mixnet_client_path,
        task_manager.subscribe_named("mixnet_client_main"),
        args.enable_wireguard,
    )
    .await?;

    // We need the IP of the gateway to correctly configure the routing table
    let gateway_used = mixnet_client.nym_address().gateway().to_base58_string();
    info!("Using gateway: {gateway_used}");
    let entry_mixnet_gateway_ip: IpAddr = gateway_client.lookup_gateway_ip(&gateway_used).await?;
    debug!("Gateway ip resolves to: {entry_mixnet_gateway_ip}");

    info!("Setting up routing");
    let routing_config = routing::RoutingConfig::new(
        args.ip.into(),
        entry_mixnet_gateway_ip,
        default_lan_gateway_ip,
        tunnel_gateway_ip,
    );
    debug!("Routing config: {:#?}", routing_config);
    let mixnet_tun_dev = routing::setup_routing(
        &mut route_manager,
        routing_config,
        args.enable_wireguard,
        args.disable_routing,
    )
    .await?;

    info!("Setting up mixnet processor");
    let processor_config = mixnet_processor::Config::new(exit_router);
    debug!("Mixnet processor config: {:#?}", processor_config);
    if let Err(err) = mixnet_processor::start_processor(
        processor_config,
        mixnet_tun_dev,
        mixnet_client,
        &task_manager,
    )
    .await {
        error!("Failed to start mixnet processor: {err}");
        // we let exucution continue as we still want to try to clean up the tunnel
        // TODO: make cleanup code always run on any failure, not just this one
    }

    // Finished starting everything, now wait for shutdown
    wait_for_interrupt(task_manager).await;
    handle_interrupt(route_manager, wireguard_waiting, tunnel_close_tx).await?;

    tunnel.dns_monitor.reset()?;
    tunnel.firewall.reset_policy()?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(err) = run().await {
        error!("Exit with error: {err}");
        eprintln!("An error occurred: {err}");
        std::process::exit(1)
    }
    Ok(())
}
