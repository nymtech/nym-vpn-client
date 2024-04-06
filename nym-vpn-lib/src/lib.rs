// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

uniffi::setup_scaffolding!();

use crate::config::WireguardConfig;
use crate::error::{Error, Result};
use crate::mixnet_connect::setup_mixnet_client;
use crate::tunnel::{setup_route_manager, start_tunnel, Tunnel};
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
use tracing::warn;
use util::wait_for_interrupt_and_signal;

// Public re-export
pub use nym_credential_storage as credential_storage;
pub use nym_gateway_directory as gateway_directory;
pub use nym_id as id;

pub use nym_ip_packet_requests::IpPair;
pub use nym_sdk::mixnet::{NodeIdentity, Recipient, StoragePaths};
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
mod uniffi_custom_impls;
mod util;
pub mod wg_gateway_client;

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
    pub mixnet_config_path: Option<PathBuf>,

    /// Mixnet public ID of the entry gateway.
    pub entry_point: EntryPoint,

    /// Mixnet recipient address.
    pub exit_point: ExitPoint,

    /// Enable the wireguard traffic between the client and the entry gateway.
    pub enable_wireguard: bool,

    /// Associated private key.
    pub private_key: Option<String>,

    /// The IP address of the wireguard interface.
    pub wg_ip: Option<Ipv4Addr>,

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

    pub enable_credentials_mode: bool,

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
            mixnet_config_path: None,
            entry_point,
            exit_point,
            enable_wireguard: false,
            private_key: None,
            wg_ip: None,
            nym_ips: None,
            nym_mtu: None,
            disable_routing: false,
            enable_two_hop: false,
            enable_poisson_rate: false,
            disable_background_cover_traffic: false,
            enable_credentials_mode: false,
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
        let exit_gateway = exit_router.gateway().to_base58_string();
        info!("Connecting to exit gateway: {exit_gateway}");
        debug!("Connecting to exit IPR: {exit_router}");
        let our_ips = mixnet_connect::connect_to_ip_packet_router(
            mixnet_client.clone(),
            exit_router,
            self.nym_ips,
            self.enable_two_hop,
        )
        .await?;
        info!("Successfully connected to exit gateway");
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
        info!("Connecting to entry gateway: {entry_gateway}");
        let mixnet_client = timeout(
            Duration::from_secs(10),
            setup_mixnet_client(
                entry_gateway,
                &self.mixnet_config_path,
                task_manager.subscribe_named("mixnet_client_main"),
                self.enable_wireguard,
                self.enable_two_hop,
                self.enable_poisson_rate,
                self.disable_background_cover_traffic,
                self.enable_credentials_mode,
            ),
        )
        .await
        .map_err(|_| Error::StartMixnetTimeout)??;

        // Now that we have a connection, collection some info about that and return
        let nym_address = mixnet_client.nym_address().await;
        let entry_gateway = nym_address.gateway().to_base58_string();
        let our_mixnet_connection = MixnetConnectionInfo {
            nym_address,
            entry_gateway: entry_gateway.clone(),
        };

        info!("Successfully connected to entry gateway: {entry_gateway}");

        // Check that we can ping ourselves before continuing
        info!("Sending mixnet ping to ourselves to verify mixnet connection");
        connection_monitor::mixnet_beacon::self_ping_and_wait(nym_address, mixnet_client.clone())
            .await?;
        info!("Successfully mixnet pinged ourselves");

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

    async fn setup_tunnel(
        &mut self,
    ) -> Result<(
        Tunnel,
        TaskManager,
        RouteManager,
        Option<(oneshot::Receiver<()>, tokio::task::JoinHandle<Result<()>>)>,
        oneshot::Sender<()>,
        MixnetConnectionInfo,
    )> {
        // Create a gateway client that we use to interact with the entry gateway, in particular to
        // handle wireguard registration
        let gateway_client = GatewayClient::new(self.gateway_config.clone())?;
        let gateways = gateway_client
            .lookup_described_gateways_with_location()
            .await?;
        log::info!("Got gateways {:?}", gateways);

        let wg_gateway_client = WgGatewayClient::new(self.wg_gateway_config.clone())?;
        log::info!("Created wg gateway client");

        // If the entry or exit point relies on location, do a basic defensive consistency check on
        // the fetched location data. If none of the gateways have location data, we can't proceed
        // and it's likely the explorer-api isn't set correctly.
        if (self.entry_point.is_location() || self.exit_point.is_location())
            && gateways.iter().filter(|g| g.has_location()).count() == 0
        {
            return Err(Error::RequestedGatewayByLocationWithoutLocationDataAvailable);
        }

        let (entry_gateway_id, entry_location) =
            self.entry_point.lookup_gateway_identity(&gateways).await?;
        let entry_location_str = entry_location.as_deref().unwrap_or("unknown");
        log::info!("Gateway id {:?}", entry_gateway_id);

        let (exit_router_address, exit_location) =
            self.exit_point.lookup_router_address(&gateways)?;
        let exit_location_str = exit_location.as_deref().unwrap_or("unknown");
        let exit_gateway_id = exit_router_address.gateway();

        info!("Using entry gateway: {entry_gateway_id}, location: {entry_location_str}");
        info!("Using exit gateway: {exit_gateway_id}, location: {exit_location_str}");
        info!("Using exit router address {exit_router_address}");

        let wireguard_config = if self.enable_wireguard {
            let private_key = self
                .private_key
                .as_ref()
                .expect("clap should enforce value when wireguard enabled");
            let wg_ip = self
                .wg_ip
                .expect("clap should enforce value when wireguard enabled");
            let wireguard_config = init_wireguard_config(
                &gateway_client,
                &wg_gateway_client,
                &entry_gateway_id.to_base58_string(),
                private_key,
                wg_ip.into(),
            )
            .await?;
            Some(wireguard_config)
        } else {
            None
        };

        // The IP address of the gateway inside the tunnel. This will depend on if wireguard is
        // enabled
        let tunnel_gateway_ip = routing::TunnelGatewayIp::new(wireguard_config.clone());
        if self.enable_wireguard {
            info!("Wireguard tunnel gateway ip: {tunnel_gateway_ip}");
        }

        // Get the IP address of the local LAN gateway
        let default_lan_gateway_ip = routing::LanGatewayIp::get_default_interface()?;
        debug!("default_lan_gateway_ip: {default_lan_gateway_ip}");

        let task_manager = TaskManager::new(10).named("nym_vpn_lib");

        info!("Setting up route manager");
        let mut route_manager = setup_route_manager().await?;

        // let route_manager_handle = route_manager.handle()?;
        let (tunnel_close_tx, tunnel_close_rx) = oneshot::channel();

        info!("Creating tunnel");
        let mut tunnel = match Tunnel::new(
            wireguard_config.clone(),
            route_manager.handle()?,
            self.tun_provider.clone(),
        ) {
            Ok(tunnel) => tunnel,
            Err(err) => {
                error!("Failed to create tunnel: {err}");
                debug!("{err:?}");
                // Ignore if these fail since we're interesting in the original error anyway
                handle_interrupt(route_manager, None, tunnel_close_tx)
                    .await
                    .tap_err(|err| {
                        warn!("Failed to handle interrupt: {err}");
                    })
                    .ok();
                return Err(err);
            }
        };

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
        let mixnet_connection_info = match self
            .setup_tunnel_services(
                &mut route_manager,
                &entry_gateway_id,
                &exit_router_address,
                &task_manager,
                &gateway_client,
                default_lan_gateway_ip,
                tunnel_gateway_ip,
            )
            .await
        {
            Ok(mixnet_connection_info) => mixnet_connection_info,
            Err(err) => {
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
        };

        Ok((
            tunnel,
            task_manager,
            route_manager,
            wireguard_waiting,
            tunnel_close_tx,
            mixnet_connection_info,
        ))
    }

    // Start the Nym VPN client, and wait for it to shutdown. The use case is in simple console
    // applications where the main way to interact with the running process is to send SIGINT
    // (ctrl-c)
    pub async fn run(&mut self) -> Result<()> {
        let (
            mut tunnel,
            task_manager,
            route_manager,
            wireguard_waiting,
            tunnel_close_tx,
            _mixnet_connection_info,
        ) = self.setup_tunnel().await?;

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
        tunnel.firewall.reset_policy().map_err(|err| {
            error!("Failed to reset firewall policy: {err}");
            Error::FirewallError(err.to_string())
        })?;

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
        let (
            mut tunnel,
            mut task_manager,
            route_manager,
            wireguard_waiting,
            tunnel_close_tx,
            mixnet_connection_info,
        ) = self.setup_tunnel().await?;

        // Signal back that we are ready and up with all cylinders firing
        let start_status = TaskStatus::ReadyWithGateway(mixnet_connection_info.entry_gateway);
        task_manager
            .start_status_listener(vpn_status_tx, start_status)
            .await;

        // Finished starting everything, now wait for mixnet client shutdown
        let result = wait_for_interrupt_and_signal(task_manager, vpn_ctrl_rx).await;

        handle_interrupt(route_manager, wireguard_waiting, tunnel_close_tx)
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
