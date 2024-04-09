// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

uniffi::setup_scaffolding!();

use crate::config::WireguardConfig;
use crate::error::{Error, Result};
use crate::mixnet_connect::setup_mixnet_client;
use crate::util::{handle_interrupt, wait_for_interrupt};
use crate::wg_gateway_client::{WgConfig, WgGatewayClient};
use futures::channel::{mpsc, oneshot};
use log::{debug, error, info};
use mixnet_connect::SharedMixnetClient;
use nym_gateway_directory::{Config, EntryPoint, ExitPoint, GatewayClient, IpPacketRouterAddress};
use nym_task::TaskManager;
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use talpid_routing::RouteManager;
use tap::TapFallible;
use tokio::time::timeout;
use tunnel_setup::{setup_tunnel, AllTunnelsSetup, TunnelSetup};
use util::wait_for_interrupt_and_signal;

// Public reexport onder gateway_directory name
pub use nym_gateway_directory as gateway_directory;

pub use nym_ip_packet_requests::IpPair;
pub use nym_sdk::mixnet::{NodeIdentity, Recipient};
pub use nym_task::{
    manager::{SentStatus, TaskStatus},
    StatusReceiver,
};

#[cfg(target_os = "ios")]
use crate::platform::swift::OSTunProvider;
pub use nym_bin_common;
pub use nym_config;
use talpid_tunnel::tun_provider::TunProvider;
use tokio::task::JoinHandle;
use tun2::AsyncDevice;

pub mod config;
mod connection_monitor;
pub mod error;
pub mod mixnet_connect;
pub mod mixnet_processor;
mod platform;
pub mod routing;
pub mod tunnel;
mod tunnel_setup;
mod uniffi_custom_impls;
mod util;
pub mod wg_gateway_client;
mod wireguard_setup;

async fn init_wireguard_config(
    gateway_client: &GatewayClient,
    wg_gateway_client: &WgGatewayClient,
    entry_gateway_identity: &str,
    wireguard_private_key: &str,
    wireguard_ip: IpAddr,
) -> Result<WireguardConfig> {
    // First we need to register with the gateway to setup keys and IP assignment
    info!("Registering with wireguard gateway");
    let entry_gateway_identity = gateway_client
        .lookup_gateway_ip(entry_gateway_identity)
        .await?;
    let wg_gateway_data = wg_gateway_client
        .register_wireguard(entry_gateway_identity, wireguard_ip)
        .await?;
    debug!("Received wireguard gateway data: {wg_gateway_data:?}");

    let wireguard_config = WireguardConfig::init(wireguard_private_key, &wg_gateway_data)?;
    info!("Wireguard config: \n{wireguard_config}");
    Ok(wireguard_config)
}

struct ShadowHandle {
    _inner: Option<JoinHandle<Result<AsyncDevice>>>,
}

pub struct NymVpn {
    /// Gateway configuration
    pub gateway_config: Config,

    /// Wireguard Gateway configuration
    pub wg_gateway_config: WgConfig,

    /// Path to the data directory of a previously initialised mixnet client, where the keys reside.
    pub mixnet_client_path: Option<PathBuf>,

    /// Mixnet public ID of the entry gateway.
    pub entry_point: EntryPoint,

    /// Mixnet recipient address.
    pub exit_point: ExitPoint,

    /// Enable the wireguard traffic between the client and the entry gateway.
    pub enable_wireguard: bool,

    /// Associated private key.
    pub private_key: Option<String>,

    /// The IP address of the entry wireguard interface.
    pub entry_wg_ip: Option<Ipv4Addr>,

    /// The IP address of the exit wireguard interface.
    pub exit_wg_ip: Option<Ipv4Addr>,

    /// The IP addresses of the TUN device.
    pub nym_ips: Option<IpPair>,

    /// The MTU of the TUN device.
    pub nym_mtu: Option<u16>,

    /// Disable routing all traffic through the VPN TUN device.
    pub disable_routing: bool,

    /// Enable two-hop mixnet traffic. This means that traffic jumps directly from entry gateway to
    /// exit gateway.
    pub enable_two_hop: bool,

    /// Enable Poission process rate limiting of outbound traffic.
    pub enable_poisson_rate: bool,

    /// Disable constant rate background loop cover traffic
    pub disable_background_cover_traffic: bool,

    tun_provider: Arc<Mutex<TunProvider>>,

    #[cfg(target_os = "ios")]
    ios_tun_provider: Arc<dyn OSTunProvider>,

    // Necessary so that the device doesn't get closed before cleanup has taken place
    shadow_handle: ShadowHandle,
}

pub struct MixnetConnectionInfo {
    pub nym_address: Recipient,
    pub entry_gateway: String,
}

impl NymVpn {
    pub fn new(
        entry_point: EntryPoint,
        exit_point: ExitPoint,
        #[cfg(target_os = "android")] android_context: talpid_types::android::AndroidContext,
        #[cfg(target_os = "ios")] ios_tun_provider: Arc<dyn OSTunProvider>,
    ) -> Self {
        let tun_provider = Arc::new(Mutex::new(TunProvider::new(
            #[cfg(target_os = "android")]
            android_context,
            #[cfg(target_os = "android")]
            false,
            #[cfg(target_os = "android")]
            None,
            #[cfg(target_os = "android")]
            vec![],
        )));

        Self {
            gateway_config: nym_gateway_directory::Config::default(),
            wg_gateway_config: wg_gateway_client::WgConfig::default(),
            mixnet_client_path: None,
            entry_point,
            exit_point,
            enable_wireguard: false,
            private_key: None,
            entry_wg_ip: None,
            exit_wg_ip: None,
            nym_ips: None,
            nym_mtu: None,
            disable_routing: false,
            enable_two_hop: false,
            enable_poisson_rate: false,
            disable_background_cover_traffic: false,
            tun_provider,
            #[cfg(target_os = "ios")]
            ios_tun_provider,
            shadow_handle: ShadowHandle { _inner: None },
        }
    }

    pub(crate) fn set_shadow_handle(&mut self, shadow_handle: JoinHandle<Result<AsyncDevice>>) {
        self.shadow_handle = ShadowHandle {
            _inner: Some(shadow_handle),
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn setup_post_mixnet(
        &mut self,
        mixnet_client: SharedMixnetClient,
        route_manager: &mut RouteManager,
        exit_router: &IpPacketRouterAddress,
        task_manager: &TaskManager,
        gateway_client: &GatewayClient,
        default_lan_gateway_ip: routing::LanGatewayIp,
        tunnel_gateway_ip: routing::TunnelGatewayIp,
    ) -> Result<()> {
        info!("Connecting to IP packet router");
        let our_ips = mixnet_connect::connect_to_ip_packet_router(
            mixnet_client.clone(),
            exit_router,
            self.nym_ips,
            self.enable_two_hop,
        )
        .await?;
        info!("Successfully connected to IP packet router!");
        info!("Using mixnet VPN IP addresses: {our_ips}");

        // We need the IP of the gateway to correctly configure the routing table
        let mixnet_client_address = mixnet_client.nym_address().await;
        let gateway_used = mixnet_client_address.gateway().to_base58_string();
        debug!("Entry gateway used for setting up routing table: {gateway_used}");
        let entry_mixnet_gateway_ip: IpAddr =
            gateway_client.lookup_gateway_ip(&gateway_used).await?;
        debug!("Gateway ip resolves to: {entry_mixnet_gateway_ip}");

        info!("Setting up routing");
        let routing_config = routing::RoutingConfig::new(
            self,
            our_ips,
            entry_mixnet_gateway_ip,
            default_lan_gateway_ip,
            tunnel_gateway_ip,
            #[cfg(target_os = "android")]
            mixnet_client.gateway_ws_fd().await,
        );
        debug!("Routing config: {}", routing_config);
        let mixnet_tun_dev = routing::setup_routing(
            route_manager,
            routing_config,
            #[cfg(target_os = "ios")]
            self.ios_tun_provider.clone(),
        )
        .await?;

        info!("Setting up mixnet processor");
        let processor_config = mixnet_processor::Config::new(*exit_router);
        debug!("Mixnet processor config: {:#?}", processor_config);

        // For other components that will want to send mixnet packets
        let mixnet_client_sender = mixnet_client.split_sender().await;

        // Setup connection monitor shared tag and channels
        let connection_monitor = connection_monitor::ConnectionMonitorTask::setup();

        let shadow_handle = mixnet_processor::start_processor(
            processor_config,
            mixnet_tun_dev,
            mixnet_client,
            task_manager,
            self.enable_two_hop,
            our_ips,
            &connection_monitor,
        )
        .await;
        self.set_shadow_handle(shadow_handle);

        connection_monitor.start(
            mixnet_client_sender,
            mixnet_client_address,
            our_ips,
            exit_router,
            task_manager,
        );

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn setup_tunnel_services(
        &mut self,
        route_manager: &mut RouteManager,
        entry_gateway: &NodeIdentity,
        exit_router: &IpPacketRouterAddress,
        task_manager: &TaskManager,
        gateway_client: &GatewayClient,
        default_lan_gateway_ip: routing::LanGatewayIp,
        tunnel_gateway_ip: routing::TunnelGatewayIp,
    ) -> Result<MixnetConnectionInfo> {
        info!("Setting up mixnet client");
        let mixnet_client = timeout(
            Duration::from_secs(10),
            setup_mixnet_client(
                entry_gateway,
                &self.mixnet_client_path,
                task_manager.subscribe_named("mixnet_client_main"),
                self.enable_wireguard,
                self.enable_two_hop,
                self.enable_poisson_rate,
                self.disable_background_cover_traffic,
            ),
        )
        .await
        .map_err(|_| Error::StartMixnetTimeout)??;

        // Now that we have a connection, collection some info about that and return
        let nym_address = mixnet_client.nym_address().await;
        let entry_gateway = nym_address.gateway().to_base58_string();
        let our_mixnet_connection = MixnetConnectionInfo {
            nym_address,
            entry_gateway,
        };

        // Check that we can ping ourselves before continuing
        info!("Sending mixnet ping to ourselves");
        connection_monitor::mixnet_beacon::self_ping_and_wait(nym_address, mixnet_client.clone())
            .await?;
        info!("Successfully pinged ourselves");

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
        Ok(our_mixnet_connection)
    }

    // Start the Nym VPN client, and wait for it to shutdown. The use case is in simple console
    // applications where the main way to interact with the running process is to send SIGINT
    // (ctrl-c)
    pub async fn run(&mut self) -> Result<()> {
        let tunnels = setup_tunnel(self).await?;

        // Finished starting everything, now wait for mixnet client shutdown
        match tunnels {
            AllTunnelsSetup::Mix(TunnelSetup {
                route_manager,
                tunnel_close_tx,
                specific_setup,
                ..
            }) => {
                wait_for_interrupt(specific_setup.task_manager).await;
                handle_interrupt(route_manager, None, tunnel_close_tx)
                    .await
                    .tap_err(|err| {
                        error!("Failed to handle interrupt: {err}");
                    })?;
            }
            AllTunnelsSetup::Wg { entry, exit } => {
                wait_for_interrupt(TaskManager::new(10)).await;
                for TunnelSetup {
                    mut tunnel,
                    route_manager,
                    tunnel_close_tx,
                    specific_setup,
                } in [entry, exit]
                {
                    handle_interrupt(
                        route_manager,
                        Some((specific_setup.receiver, specific_setup.handle)),
                        tunnel_close_tx,
                    )
                    .await
                    .tap_err(|err| {
                        error!("Failed to handle interrupt: {err}");
                    })?;

                    tunnel.dns_monitor.reset().tap_err(|err| {
                        error!("Failed to reset dns monitor: {err}");
                    })?;
                    tunnel.firewall.reset_policy().map_err(|err| {
                        error!("Failed to reset firewall policy: {err}");
                        Error::FirewallError(err.to_string())
                    })?;
                }
            }
        }

        Ok(())
    }

    // Start the Nym VPN client, but also listen for external messages to e.g. disconnect as well
    // as reporting it's status on the provided channel. The usecase when the VPN is embedded in
    // another application, or running as a background process with a graphical interface remote
    // controlling it.
    pub async fn run_and_listen(
        &mut self,
        vpn_status_tx: nym_task::StatusSender,
        vpn_ctrl_rx: mpsc::UnboundedReceiver<NymVpnCtrlMessage>,
    ) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let tunnels = setup_tunnel(self).await?;

        // Finished starting everything, now wait for mixnet client shutdown
        match tunnels {
            AllTunnelsSetup::Mix(TunnelSetup {
                route_manager,
                tunnel_close_tx,
                mut specific_setup,
                ..
            }) => {
                // Signal back that mixnet is ready and up with all cylinders firing
                let start_status = TaskStatus::ReadyWithGateway(
                    specific_setup.mixnet_connection_info.entry_gateway.clone(),
                );
                specific_setup
                    .task_manager
                    .start_status_listener(vpn_status_tx, start_status)
                    .await;
                let result =
                    wait_for_interrupt_and_signal(Some(specific_setup.task_manager), vpn_ctrl_rx)
                        .await;
                handle_interrupt(route_manager, None, tunnel_close_tx)
                    .await
                    .map_err(|err| {
                        error!("Failed to handle interrupt: {err}");
                        Box::new(NymVpnExitError::Generic { reason: err })
                    })?;
                result
            }
            AllTunnelsSetup::Wg { entry, exit } => {
                let result = wait_for_interrupt_and_signal(None, vpn_ctrl_rx).await;
                for TunnelSetup {
                    mut tunnel,
                    route_manager,
                    tunnel_close_tx,
                    specific_setup,
                } in [entry, exit]
                {
                    handle_interrupt(
                        route_manager,
                        Some((specific_setup.receiver, specific_setup.handle)),
                        tunnel_close_tx,
                    )
                    .await
                    .map_err(|err| {
                        error!("Failed to handle interrupt: {err}");
                        Box::new(NymVpnExitError::Generic { reason: err })
                    })?;
                    tunnel.dns_monitor.reset().map_err(|err| {
                        error!("Failed to reset dns monitor: {err}");
                        NymVpnExitError::FailedToResetDnsMonitor {
                            reason: err.to_string(),
                        }
                    })?;
                    tunnel.firewall.reset_policy().map_err(|err| {
                        error!("Failed to reset firewall policy: {err}");
                        NymVpnExitError::FailedToResetFirewallPolicy {
                            reason: err.to_string(),
                        }
                    })?;
                }
                result
            }
        }
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
    Generic { reason: Error },

    // TODO: capture the concrete error type once we have time to investigate on Mac
    #[error("failed to reset firewall policy: {reason}")]
    FailedToResetFirewallPolicy { reason: String },

    #[error("failed to reset dns monitor: {reason}")]
    FailedToResetDnsMonitor { reason: String },
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
/// use nym_vpn_lib::gateway_directory::{EntryPoint, ExitPoint};
/// use nym_vpn_lib::NodeIdentity;
///
/// let mut vpn_config = nym_vpn_lib::NymVpn::new(EntryPoint::Gateway { identity: NodeIdentity::from_base58_string("Qwertyuiopasdfghjklzxcvbnm1234567890").unwrap()},
/// ExitPoint::Gateway { identity: NodeIdentity::from_base58_string("Qwertyuiopasdfghjklzxcvbnm1234567890".to_string()).unwrap()});
/// vpn_config.enable_two_hop = true;
/// let vpn_handle = nym_vpn_lib::spawn_nym_vpn(vpn_config);
/// ```
pub fn spawn_nym_vpn(nym_vpn: NymVpn) -> Result<NymVpnHandle> {
    let (vpn_ctrl_tx, vpn_ctrl_rx) = mpsc::unbounded();
    let (vpn_status_tx, vpn_status_rx) = mpsc::channel(128);
    let (vpn_exit_tx, vpn_exit_rx) = oneshot::channel();

    tokio::spawn(run_nym_vpn(
        nym_vpn,
        vpn_status_tx,
        vpn_ctrl_rx,
        vpn_exit_tx,
    ));

    Ok(NymVpnHandle {
        vpn_ctrl_tx,
        vpn_status_rx,
        vpn_exit_rx,
    })
}

/// Starts the Nym VPN client, in a separate tokio runtime.
///
/// Examples
///
/// ```no_run
/// use nym_vpn_lib::gateway_directory::{EntryPoint, ExitPoint};
/// use nym_vpn_lib::NodeIdentity;
///
/// let mut vpn_config = nym_vpn_lib::NymVpn::new(EntryPoint::Gateway { identity: NodeIdentity::from_base58_string("Qwertyuiopasdfghjklzxcvbnm1234567890").unwrap()},
/// ExitPoint::Gateway { identity: NodeIdentity::from_base58_string("Qwertyuiopasdfghjklzxcvbnm1234567890".to_string()).unwrap()});
/// vpn_config.enable_two_hop = true;
/// let vpn_handle = nym_vpn_lib::spawn_nym_vpn_with_new_runtime(vpn_config);
/// ```
pub fn spawn_nym_vpn_with_new_runtime(nym_vpn: NymVpn) -> Result<NymVpnHandle> {
    let (vpn_ctrl_tx, vpn_ctrl_rx) = mpsc::unbounded();
    let (vpn_status_tx, vpn_status_rx) = mpsc::channel(128);
    let (vpn_exit_tx, vpn_exit_rx) = oneshot::channel();

    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio run time");
        rt.block_on(run_nym_vpn(
            nym_vpn,
            vpn_status_tx,
            vpn_ctrl_rx,
            vpn_exit_tx,
        ));
    });

    Ok(NymVpnHandle {
        vpn_ctrl_tx,
        vpn_status_rx,
        vpn_exit_rx,
    })
}

async fn run_nym_vpn(
    mut nym_vpn: NymVpn,
    vpn_status_tx: nym_task::StatusSender,
    vpn_ctrl_rx: mpsc::UnboundedReceiver<NymVpnCtrlMessage>,
    vpn_exit_tx: oneshot::Sender<NymVpnExitStatusMessage>,
) {
    let result = nym_vpn.run_and_listen(vpn_status_tx, vpn_ctrl_rx).await;
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
}

pub struct NymVpnHandle {
    pub vpn_ctrl_tx: mpsc::UnboundedSender<NymVpnCtrlMessage>,
    pub vpn_status_rx: nym_task::StatusReceiver,
    pub vpn_exit_rx: oneshot::Receiver<NymVpnExitStatusMessage>,
}
