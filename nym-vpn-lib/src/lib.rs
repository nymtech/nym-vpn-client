// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::config::WireguardConfig;
use crate::error::Result;
use crate::gateway_client::{Config, GatewayClient};
use crate::mixnet_connect::setup_mixnet_client;
use crate::mixnet_processor::IpPacketRouterAddress;
use crate::tunnel::{setup_route_manager, start_tunnel, Tunnel};
use crate::util::{handle_interrupt, wait_for_interrupt};
use futures::channel::{mpsc, oneshot};
use log::{debug, error, info};
use mixnet_connect::SharedMixnetClient;
use nym_task::TaskManager;
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use talpid_routing::RouteManager;
use tap::TapFallible;
use tracing::warn;
use util::wait_for_interrupt_and_signal;

pub use nym_task::{manager::SentStatus, StatusReceiver};

pub use nym_bin_common;
pub use nym_config;

pub mod config;
pub mod error;
pub mod gateway_client;
pub mod mixnet_connect;
pub mod mixnet_processor;
pub mod routing;
pub mod tunnel;
mod util;

async fn init_wireguard_config(
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

#[derive(Clone)]
pub struct NymVpn {
    /// Gateway configuration
    pub gateway_config: Config,

    /// Path to the data directory of a previously initialised mixnet client, where the keys reside.
    pub mixnet_client_path: Option<PathBuf>,

    /// Mixnet public ID of the entry gateway.
    pub entry_gateway: String,

    /// Mixnet recipient address.
    pub exit_router: String,

    /// Enable the wireguard traffic between the client and the entry gateway.
    pub enable_wireguard: bool,

    /// Associated private key.
    pub private_key: Option<String>,

    /// The IP address of the TUN device.
    pub ip: Option<Ipv4Addr>,

    /// The MTU of the TUN device.
    pub mtu: Option<i32>,

    /// Disable routing all traffic through the VPN TUN device.
    pub disable_routing: bool,

    /// Enable two-hop mixnet traffic. This means that traffic jumps directly from entry gateway to
    /// exit gateway.
    pub enable_two_hop: bool,

    /// Enable Poission process rate limiting of outbound traffic.
    pub enable_poisson_rate: bool,
}

impl NymVpn {
    pub fn new(entry_gateway: &str, exit_router: &str) -> Self {
        Self {
            gateway_config: gateway_client::Config::default(),
            mixnet_client_path: None,
            entry_gateway: entry_gateway.to_string(),
            exit_router: exit_router.to_string(),
            enable_wireguard: false,
            private_key: None,
            ip: None,
            mtu: None,
            disable_routing: false,
            enable_two_hop: false,
            enable_poisson_rate: false,
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn setup_post_mixnet(
        &self,
        mixnet_client: SharedMixnetClient,
        route_manager: &mut RouteManager,
        exit_router: &IpPacketRouterAddress,
        task_manager: &TaskManager,
        gateway_client: &GatewayClient,
        default_lan_gateway_ip: routing::LanGatewayIp,
        tunnel_gateway_ip: routing::TunnelGatewayIp,
    ) -> Result<()> {
        info!("Connecting to IP packet router");
        let ip = mixnet_connect::connect_to_ip_packet_router(
            mixnet_client.clone(),
            exit_router,
            self.ip,
            self.enable_two_hop,
        )
        .await?;
        info!("Sucessfully connected to IP packet router on the exit gateway!");
        info!("Using IP address: {ip}");

        // We need the IP of the gateway to correctly configure the routing table
        let mixnet_client_address = mixnet_client.nym_address().await;
        let gateway_used = mixnet_client_address.gateway().to_base58_string();
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
            route_manager,
            routing_config,
            self.enable_wireguard,
            self.disable_routing,
        )
        .await?;

        info!("Setting up mixnet processor");
        let processor_config = mixnet_processor::Config::new(*exit_router);
        debug!("Mixnet processor config: {:#?}", processor_config);
        mixnet_processor::start_processor(
            processor_config,
            mixnet_tun_dev,
            mixnet_client,
            task_manager,
            self.enable_two_hop,
        )
        .await
    }

    async fn setup_tunnel_services(
        &self,
        route_manager: &mut RouteManager,
        exit_router: &IpPacketRouterAddress,
        task_manager: &TaskManager,
        gateway_client: &GatewayClient,
        default_lan_gateway_ip: routing::LanGatewayIp,
        tunnel_gateway_ip: routing::TunnelGatewayIp,
    ) -> Result<()> {
        info!("Setting up mixnet client");
        let mixnet_client = setup_mixnet_client(
            &self.entry_gateway,
            &self.mixnet_client_path,
            task_manager.subscribe_named("mixnet_client_main"),
            self.enable_wireguard,
            self.enable_two_hop,
            self.enable_poisson_rate,
        )
        .await?;

        if let Err(err) = self
            .setup_post_mixnet(
                mixnet_client.clone(),
                route_manager,
                exit_router,
                task_manager,
                gateway_client,
                default_lan_gateway_ip,
                tunnel_gateway_ip,
            )
            .await
        {
            error!("Failed to setup post mixnet: {err}");
            debug!("{err:?}");
            mixnet_client.disconnect().await;
            return Err(err);
        };
        Ok(())
    }

    async fn setup_tunnel(
        &self,
    ) -> Result<(
        Tunnel,
        TaskManager,
        RouteManager,
        Option<(oneshot::Receiver<()>, tokio::task::JoinHandle<Result<()>>)>,
        oneshot::Sender<()>,
    )> {
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

        // Now it's time start all the stuff that needs running inside the tunnel, and that we need
        // correctly unwind if it fails
        // - Sets up mixnet client, and connects
        // - Sets up routing
        // - Starts processing packets
        if let Err(err) = self
            .setup_tunnel_services(
                &mut route_manager,
                &exit_router,
                &task_manager,
                &gateway_client,
                default_lan_gateway_ip,
                tunnel_gateway_ip,
            )
            .await
        {
            error!("Failed to setup tunnel services: {err}");
            debug!("{err:?}");
            wait_for_interrupt(task_manager).await;
            // Ignore if these fail since we're interesting in the original error anyway
            handle_interrupt(route_manager, wireguard_waiting, tunnel_close_tx)
                .await
                .tap_err(|err| {
                    warn!("Failed to handle interrupt: {err}");
                })
                .ok();
            tunnel
                .dns_monitor
                .reset()
                .tap_err(|err| {
                    warn!("Failed to reset dns monitor: {err}");
                })
                .ok();
            tunnel
                .firewall
                .reset_policy()
                .tap_err(|err| {
                    warn!("Failed to reset firewall policy: {err}");
                })
                .ok();
            return Err(err);
        }

        Ok((
            tunnel,
            task_manager,
            route_manager,
            wireguard_waiting,
            tunnel_close_tx,
        ))
    }

    // Start the Nym VPN client, and wait for it to shutdown. The use case is in simple console
    // applications where the main way to interact with the running process is to send SIGINT
    // (ctrl-c)
    pub async fn run(&self) -> Result<()> {
        let (mut tunnel, task_manager, route_manager, wireguard_waiting, tunnel_close_tx) =
            self.setup_tunnel().await?;

        // Finished starting everything, now wait for shutdown
        wait_for_interrupt(task_manager).await;
        handle_interrupt(route_manager, wireguard_waiting, tunnel_close_tx)
            .await
            .tap_err(|err| {
                error!("Failed to handle interrupt: {err}");
            })?;

        tunnel.dns_monitor.reset().tap_err(|err| {
            error!("Failed to reset dns monitor: {err}");
        })?;
        tunnel.firewall.reset_policy().tap_err(|err| {
            error!("Failed to reset firewall policy: {err}");
        })?;

        Ok(())
    }

    // Start the Nym VPN client, but also listen for external messages to e.g. disconnect as well
    // as reporting it's status on the provided channel. The usecase when the VPN is embedded in
    // another application, or running as a background process with a graphical interface remote
    // controlling it.
    pub async fn run_and_listen(
        &self,
        vpn_status_tx: nym_task::StatusSender,
        vpn_ctrl_rx: mpsc::UnboundedReceiver<NymVpnCtrlMessage>,
    ) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let (mut tunnel, mut task_manager, route_manager, wireguard_waiting, tunnel_close_tx) =
            self.setup_tunnel()
                .await
                .map_err(|err| Box::new(NymVpnExitError::generic(&err)))?;

        // Signal back that we are ready and up with all cylinders firing
        task_manager.start_status_listener(vpn_status_tx).await;

        // Finished starting everything, now wait for mixnet client shutdown
        let result = wait_for_interrupt_and_signal(task_manager, vpn_ctrl_rx).await;

        handle_interrupt(route_manager, wireguard_waiting, tunnel_close_tx)
            .await
            .map_err(|err| {
                error!("Failed to handle interrupt: {err}");
                Box::new(NymVpnExitError::generic(&err))
            })?;
        tunnel.dns_monitor.reset().map_err(|err| {
            error!("Failed to reset dns monitor: {err}");
            Box::new(NymVpnExitError::FailedToResetDnsMonitor {
                reason: err.to_string(),
            })
        })?;
        tunnel.firewall.reset_policy().map_err(|err| {
            error!("Failed to reset firewall policy: {err}");
            Box::new(NymVpnExitError::FailedToResetFirewallPolicy {
                reason: err.to_string(),
            })
        })?;

        result
    }
}

#[derive(Debug)]
pub enum NymVpnStatusMessage {
    Slow,
}

#[derive(Debug)]
pub enum NymVpnCtrlMessage {
    Stop,
}

// We are mapping all errors to a generic error since I ran into issues with the error type
// on a platform (mac) that I wasn't able to troubleshoot on in time. Basically it seemed like
// not all error cases satisfied the Sync marker trait.
#[derive(thiserror::Error, Debug)]
pub enum NymVpnExitError {
    #[error("{reason}")]
    Generic { reason: String },

    // TODO: capture the concrete error type once we have time to investigate on Mac
    #[error("failed to reset firewall policy: {reason}")]
    FailedToResetFirewallPolicy { reason: String },

    #[error("failed to reset dns monitor: {reason}")]
    FailedToResetDnsMonitor { reason: String },
}

impl NymVpnExitError {
    fn generic(err: &dyn std::error::Error) -> Self {
        NymVpnExitError::Generic {
            reason: err.to_string(),
        }
    }
}

#[derive(Debug)]
pub enum NymVpnExitStatusMessage {
    Stopped,
    Failed(Box<dyn std::error::Error + Send + Sync + 'static>),
}

/// Starts the Nym VPN client.
///
/// Examples
///
/// ```no_run
/// let mut vpn_config = nym_vpn_lib::NymVpn::new("Qwertyuiopasdfghjklzxcvbnm1234567890", "Qwertyuiopasdfghjklzxcvbnm1234567890");
/// vpn_config.enable_two_hop = true;
/// let vpn_handle = nym_vpn_lib::spawn_nym_vpn(vpn_config);
/// ```
pub fn spawn_nym_vpn(nym_vpn: NymVpn) -> Result<NymVpnHandle> {
    let (vpn_ctrl_tx, vpn_ctrl_rx) = mpsc::unbounded();

    let (vpn_status_tx, vpn_status_rx) = mpsc::channel(128);

    let (vpn_exit_tx, vpn_exit_rx) = oneshot::channel();

    std::thread::spawn(|| {
        let result = tokio::runtime::Runtime::new()
            .expect("Failed to create Tokio run time")
            .block_on(async move { nym_vpn.run_and_listen(vpn_status_tx, vpn_ctrl_rx).await });

        if let Err(err) = result {
            error!("Nym VPN returned error: {err}");
            debug!("{err:?}");
            vpn_exit_tx
                .send(NymVpnExitStatusMessage::Failed(err))
                .expect("Failed to send exit status");
            return;
        }

        log::info!("Nym VPN has shut down");
        vpn_exit_tx
            .send(NymVpnExitStatusMessage::Stopped)
            .expect("Failed to send exit status");
    });

    Ok(NymVpnHandle {
        vpn_ctrl_tx,
        vpn_status_rx,
        vpn_exit_rx,
    })
}

pub struct NymVpnHandle {
    pub vpn_ctrl_tx: mpsc::UnboundedSender<NymVpnCtrlMessage>,
    pub vpn_status_rx: nym_task::StatusReceiver,
    pub vpn_exit_rx: oneshot::Receiver<NymVpnExitStatusMessage>,
}
