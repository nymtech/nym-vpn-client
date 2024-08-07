// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

uniffi::setup_scaffolding!();

use crate::config::WireguardConfig;
use crate::error::{Error, Result};
use crate::mixnet_connect::setup_mixnet_client;
use crate::tunnel::setup_route_manager;
use crate::util::{handle_interrupt, wait_for_interrupt};
use crate::wg_gateway_client::WgGatewayClient;
use error::GatewayDirectoryError;
use futures::channel::{mpsc, oneshot};
use futures::SinkExt;
use log::{debug, error, info};
use mixnet_connect::SharedMixnetClient;
use nym_connection_monitor::ConnectionMonitorTask;
use nym_gateway_directory::{
    Config as GatewayDirectoryConfig, EntryPoint, ExitPoint, GatewayClient, IpPacketRouterAddress,
};
use nym_ip_packet_client::IprClient;
use nym_task::TaskManager;
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use talpid_core::dns::DnsMonitor;
use talpid_routing::RouteManager;
use tunnel_setup::{init_firewall_dns, setup_tunnel, AllTunnelsSetup, TunnelSetup};
use util::wait_for_interrupt_and_signal;

// Public re-export
pub use nym_connection_monitor as connection_monitor;
pub use nym_credential_storage as credential_storage;
pub use nym_gateway_directory as gateway_directory;
pub use nym_id as id;

pub use nym_ip_packet_requests::IpPair;
pub use nym_sdk::mixnet::{NodeIdentity, Recipient, StoragePaths};
pub use nym_sdk::UserAgent;
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

mod bandwidth_controller;
mod platform;
mod tunnel_setup;
mod uniffi_custom_impls;

pub mod config;
pub mod credentials;
pub mod error;
pub mod keys;
pub mod mixnet_connect;
pub mod mixnet_processor;
pub mod routing;
pub mod tunnel;
pub mod util;
pub mod wg_gateway_client;
mod wireguard_setup;

const MIXNET_CLIENT_STARTUP_TIMEOUT_SECS: u64 = 30;
pub const SHUTDOWN_TIMER_SECS: u64 = 10;

async fn init_wireguard_config(
    gateway_client: &GatewayClient,
    wg_gateway_client: &mut WgGatewayClient,
    wg_gateway: Option<IpAddr>,
    mtu: u16,
) -> Result<(WireguardConfig, IpAddr)> {
    // First we need to register with the gateway to setup keys and IP assignment
    info!("Registering with wireguard gateway");
    let gateway_auth_recipient = wg_gateway_client
        .auth_recipient()
        .gateway()
        .to_base58_string();
    let gateway_host = gateway_client
        .lookup_gateway_ip(&gateway_auth_recipient)
        .await
        .map_err(|source| GatewayDirectoryError::FailedToLookupGatewayIp {
            gateway_id: gateway_auth_recipient,
            source,
        })?;
    let wg_gateway_data = wg_gateway_client.register_wireguard(gateway_host).await?;
    debug!("Received wireguard gateway data: {wg_gateway_data:?}");

    let wireguard_config = WireguardConfig::init(
        wg_gateway_client.keypair(),
        &wg_gateway_data,
        wg_gateway,
        mtu,
    )?;
    Ok((wireguard_config, gateway_host))
}

struct ShadowHandle {
    _inner: Option<JoinHandle<Result<AsyncDevice>>>,
}

pub struct MixnetVpn {}

pub struct WireguardVpn {}

pub trait Vpn {}

impl Vpn for MixnetVpn {}
impl Vpn for WireguardVpn {}

pub enum SpecificVpn {
    Wg(NymVpn<WireguardVpn>),
    Mix(NymVpn<MixnetVpn>),
}

impl From<NymVpn<WireguardVpn>> for SpecificVpn {
    fn from(value: NymVpn<WireguardVpn>) -> Self {
        Self::Wg(value)
    }
}

impl From<NymVpn<MixnetVpn>> for SpecificVpn {
    fn from(value: NymVpn<MixnetVpn>) -> Self {
        Self::Mix(value)
    }
}

#[derive(Clone, Debug)]
pub struct MixnetClientConfig {
    /// Enable Poission process rate limiting of outbound traffic.
    pub enable_poisson_rate: bool,

    /// Disable constant rate background loop cover traffic
    pub disable_background_cover_traffic: bool,

    /// Enable the credentials mode between the client and the entry gateway.
    pub enable_credentials_mode: bool,

    /// The minimum performance of mixnodes to use.
    pub min_mixnode_performance: Option<u8>,

    /// The minimum performance of gateways to use.
    pub min_gateway_performance: Option<u8>,
}

pub struct NymVpn<T: Vpn> {
    pub mixnet_client_config: MixnetClientConfig,

    /// Path to the data directory, where keys reside.
    pub data_path: Option<PathBuf>,

    /// Gateway configuration
    pub gateway_config: GatewayDirectoryConfig,

    /// Mixnet public ID of the entry gateway.
    pub entry_point: EntryPoint,

    /// Mixnet recipient address.
    pub exit_point: ExitPoint,

    /// Enable two-hop mixnet traffic. This means that traffic jumps directly from entry gateway to
    /// exit gateway.
    pub enable_two_hop: bool,

    /// VPN configuration, depending on the type used
    pub vpn_config: T,

    /// The IP addresses of the TUN device.
    pub nym_ips: Option<IpPair>,

    /// The MTU of the TUN device.
    pub nym_mtu: Option<u16>,

    /// The DNS server to use
    pub dns: Option<IpAddr>,

    /// Disable routing all traffic through the VPN TUN device.
    pub disable_routing: bool,

    /// The user agent to use for HTTP requests. This includes client name, version, platform and
    /// git commit hash.
    pub user_agent: Option<UserAgent>,

    tun_provider: Arc<Mutex<TunProvider>>,

    #[cfg(target_os = "ios")]
    ios_tun_provider: Arc<dyn OSTunProvider>,

    // Necessary so that the device doesn't get closed before cleanup has taken place
    shadow_handle: ShadowHandle,
}

#[derive(Debug)]
pub struct MixnetConnectionInfo {
    pub nym_address: Recipient,
    pub entry_gateway: NodeIdentity,
}

#[derive(Debug)]
pub struct MixnetExitConnectionInfo {
    pub exit_gateway: NodeIdentity,
    pub exit_ipr: Recipient,
    pub ips: IpPair,
}

impl NymVpn<WireguardVpn> {
    pub fn new_wireguard_vpn(
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
            mixnet_client_config: MixnetClientConfig {
                enable_poisson_rate: false,
                disable_background_cover_traffic: false,
                enable_credentials_mode: false,
                min_mixnode_performance: None,
                min_gateway_performance: None,
            },
            data_path: None,
            gateway_config: nym_gateway_directory::Config::default(),
            entry_point,
            exit_point,
            nym_ips: None,
            nym_mtu: None,
            dns: None,
            disable_routing: false,
            enable_two_hop: false,
            user_agent: None,
            vpn_config: WireguardVpn {},
            tun_provider,
            #[cfg(target_os = "ios")]
            ios_tun_provider,
            shadow_handle: ShadowHandle { _inner: None },
        }
    }
}

impl NymVpn<MixnetVpn> {
    pub fn new_mixnet_vpn(
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
            mixnet_client_config: MixnetClientConfig {
                enable_poisson_rate: false,
                disable_background_cover_traffic: false,
                enable_credentials_mode: false,
                min_mixnode_performance: None,
                min_gateway_performance: None,
            },
            data_path: None,
            gateway_config: nym_gateway_directory::Config::default(),
            entry_point,
            exit_point,
            nym_ips: None,
            nym_mtu: None,
            dns: None,
            disable_routing: false,
            enable_two_hop: false,
            user_agent: None,
            vpn_config: MixnetVpn {},
            tun_provider,
            #[cfg(target_os = "ios")]
            ios_tun_provider,
            shadow_handle: ShadowHandle { _inner: None },
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn setup_post_mixnet(
        &mut self,
        mixnet_client: SharedMixnetClient,
        route_manager: &mut RouteManager,
        exit_mix_addresses: &IpPacketRouterAddress,
        task_manager: &TaskManager,
        gateway_client: &GatewayClient,
        default_lan_gateway_ip: routing::LanGatewayIp,
        dns_monitor: &mut DnsMonitor,
    ) -> Result<MixnetExitConnectionInfo> {
        let exit_gateway = *exit_mix_addresses.gateway();
        info!("Connecting to exit gateway: {exit_gateway}");
        // Currently the IPR client is only used to connect. The next step would be to use it to
        // spawn a separate task that handles IPR request/responses.
        let mut ipr_client = IprClient::new_from_inner(mixnet_client.inner()).await;
        let our_ips = ipr_client
            .connect(exit_mix_addresses.0, self.nym_ips, self.enable_two_hop)
            .await?;
        info!("Successfully connected to exit gateway");
        info!("Using mixnet VPN IP addresses: {our_ips}");

        // We need the IP of the gateway to correctly configure the routing table
        let mixnet_client_address = mixnet_client.nym_address().await;
        let gateway_used = mixnet_client_address.gateway().to_base58_string();
        debug!("Entry gateway used for setting up routing table: {gateway_used}");
        let entry_mixnet_gateway_ip: IpAddr = gateway_client
            .lookup_gateway_ip(&gateway_used)
            .await
            .map_err(|source| GatewayDirectoryError::FailedToLookupGatewayIp {
                gateway_id: gateway_used,
                source,
            })?;
        debug!("Gateway ip resolves to: {entry_mixnet_gateway_ip}");

        info!("Setting up routing");
        let routing_config = routing::RoutingConfig::new(
            self,
            our_ips,
            entry_mixnet_gateway_ip,
            default_lan_gateway_ip,
            #[cfg(target_os = "android")]
            mixnet_client.gateway_ws_fd().await,
        );
        debug!("Routing config: {}", routing_config);
        let mixnet_tun_dev = routing::setup_mixnet_routing(
            route_manager,
            routing_config,
            #[cfg(target_os = "ios")]
            self.ios_tun_provider.clone(),
            dns_monitor,
            self.dns,
        )
        .await?;

        info!("Setting up mixnet processor");
        let processor_config = mixnet_processor::Config::new(exit_mix_addresses.0);
        debug!("Mixnet processor config: {:#?}", processor_config);

        // For other components that will want to send mixnet packets
        let mixnet_client_sender = mixnet_client.split_sender().await;

        // Setup connection monitor shared tag and channels
        let connection_monitor = ConnectionMonitorTask::setup();

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
            exit_mix_addresses.0,
            task_manager,
        );

        Ok(MixnetExitConnectionInfo {
            exit_gateway,
            exit_ipr: exit_mix_addresses.0,
            ips: our_ips,
        })
    }

    #[allow(clippy::too_many_arguments)]
    async fn setup_tunnel_services(
        &mut self,
        mixnet_client: SharedMixnetClient,
        route_manager: &mut RouteManager,
        exit_mix_addresses: &IpPacketRouterAddress,
        task_manager: &TaskManager,
        gateway_client: &GatewayClient,
        default_lan_gateway_ip: routing::LanGatewayIp,
        dns_monitor: &mut DnsMonitor,
    ) -> Result<(MixnetConnectionInfo, MixnetExitConnectionInfo)> {
        // Now that we have a connection, collection some info about that and return
        let nym_address = mixnet_client.nym_address().await;
        let entry_gateway = *(nym_address.gateway());
        info!("Successfully connected to entry gateway: {entry_gateway}");

        let our_mixnet_connection = MixnetConnectionInfo {
            nym_address,
            entry_gateway,
        };

        // Check that we can ping ourselves before continuing
        info!("Sending mixnet ping to ourselves to verify mixnet connection");
        nym_connection_monitor::self_ping_and_wait(nym_address, mixnet_client.inner()).await?;
        info!("Successfully mixnet pinged ourselves");

        match self
            .setup_post_mixnet(
                mixnet_client.clone(),
                route_manager,
                exit_mix_addresses,
                task_manager,
                gateway_client,
                default_lan_gateway_ip,
                dns_monitor,
            )
            .await
        {
            Err(err) => {
                error!("Failed to setup post mixnet: {err}");
                debug!("{err:?}");
                mixnet_client.disconnect().await;
                Err(err)
            }
            Ok(exit_connection_info) => Ok((our_mixnet_connection, exit_connection_info)),
        }
    }
}

impl<T: Vpn> NymVpn<T> {
    pub(crate) fn set_shadow_handle(&mut self, shadow_handle: JoinHandle<Result<AsyncDevice>>) {
        self.shadow_handle = ShadowHandle {
            _inner: Some(shadow_handle),
        }
    }
}
impl SpecificVpn {
    pub fn mixnet_client_config(&self) -> MixnetClientConfig {
        match self {
            SpecificVpn::Wg(vpn) => vpn.mixnet_client_config.clone(),
            SpecificVpn::Mix(vpn) => vpn.mixnet_client_config.clone(),
        }
    }

    pub fn data_path(&self) -> Option<PathBuf> {
        match self {
            SpecificVpn::Wg(vpn) => vpn.data_path.clone(),
            SpecificVpn::Mix(vpn) => vpn.data_path.clone(),
        }
    }

    pub fn gateway_config(&self) -> GatewayDirectoryConfig {
        match self {
            SpecificVpn::Wg(vpn) => vpn.gateway_config.clone(),
            SpecificVpn::Mix(vpn) => vpn.gateway_config.clone(),
        }
    }

    pub fn entry_point(&self) -> EntryPoint {
        match self {
            SpecificVpn::Wg(vpn) => vpn.entry_point.clone(),
            SpecificVpn::Mix(vpn) => vpn.entry_point.clone(),
        }
    }

    pub fn exit_point(&self) -> ExitPoint {
        match self {
            SpecificVpn::Wg(vpn) => vpn.exit_point.clone(),
            SpecificVpn::Mix(vpn) => vpn.exit_point.clone(),
        }
    }

    pub fn enable_two_hop(&self) -> bool {
        match self {
            SpecificVpn::Wg(vpn) => vpn.enable_two_hop,
            SpecificVpn::Mix(vpn) => vpn.enable_two_hop,
        }
    }

    pub fn user_agent(&self) -> Option<UserAgent> {
        match self {
            SpecificVpn::Wg(vpn) => vpn.user_agent.clone(),
            SpecificVpn::Mix(vpn) => vpn.user_agent.clone(),
        }
    }

    // Start the Nym VPN client, and wait for it to shutdown. The use case is in simple console
    // applications where the main way to interact with the running process is to send SIGINT
    // (ctrl-c)
    pub async fn run(&mut self) -> Result<()> {
        let mut task_manager = TaskManager::new(SHUTDOWN_TIMER_SECS).named("nym_vpn_lib");
        info!("Setting up route manager");
        let mut route_manager = setup_route_manager().await?;
        let (mut firewall, mut dns_monitor) = init_firewall_dns(
            #[cfg(target_os = "linux")]
            route_manager.handle()?,
        )
        .await?;
        let tunnels = match setup_tunnel(
            self,
            &mut task_manager,
            &mut route_manager,
            &mut dns_monitor,
        )
        .await
        {
            Ok(tunnels) => tunnels,
            Err(e) => {
                tokio::task::spawn_blocking(move || {
                    dns_monitor
                        .reset()
                        .inspect_err(|err| {
                            log::error!("Failed to reset dns monitor: {err}");
                        })
                        .ok();
                    firewall
                        .reset_policy()
                        .inspect_err(|err| {
                            error!("Failed to reset firewall policy: {err}");
                        })
                        .ok();
                    drop(route_manager);
                })
                .await?;
                return Err(e);
            }
        };
        info!("Nym VPN is now running");

        // Finished starting everything, now wait for mixnet client shutdown
        match tunnels {
            AllTunnelsSetup::Mix(_) => {
                wait_for_interrupt(task_manager).await;
                handle_interrupt(route_manager, None).await;
                tokio::task::spawn_blocking(move || {
                    dns_monitor.reset().inspect_err(|err| {
                        log::error!("Failed to reset dns monitor: {err}");
                    })
                })
                .await??;
            }
            AllTunnelsSetup::Wg { entry, exit } => {
                wait_for_interrupt(task_manager).await;
                handle_interrupt(
                    route_manager,
                    Some([entry.specific_setup, exit.specific_setup]),
                )
                .await;

                tokio::task::spawn_blocking(move || {
                    dns_monitor.reset().inspect_err(|err| {
                        log::error!("Failed to reset dns monitor: {err}");
                    })
                })
                .await??;
                firewall.reset_policy().map_err(|err| {
                    error!("Failed to reset firewall policy: {err}");
                    Error::FirewallError(err.to_string())
                })?;
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
        mut vpn_status_tx: nym_task::StatusSender,
        vpn_ctrl_rx: mpsc::UnboundedReceiver<NymVpnCtrlMessage>,
    ) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let mut task_manager = TaskManager::new(SHUTDOWN_TIMER_SECS).named("nym_vpn_lib");
        info!("Setting up route manager");
        let mut route_manager = setup_route_manager().await?;
        let (mut firewall, mut dns_monitor) = init_firewall_dns(
            #[cfg(target_os = "linux")]
            route_manager.handle()?,
        )
        .await?;
        let tunnels = match setup_tunnel(
            self,
            &mut task_manager,
            &mut route_manager,
            &mut dns_monitor,
        )
        .await
        {
            Ok(tunnels) => tunnels,
            Err(e) => {
                tokio::task::spawn_blocking(move || {
                    dns_monitor
                        .reset()
                        .inspect_err(|err| {
                            log::error!("Failed to reset dns monitor: {err}");
                        })
                        .ok();
                    firewall
                        .reset_policy()
                        .inspect_err(|err| {
                            error!("Failed to reset firewall policy: {err}");
                        })
                        .ok();
                    drop(route_manager);
                })
                .await?;
                return Err(Box::new(e));
            }
        };

        // Finished starting everything, now wait for mixnet client shutdown
        match tunnels {
            AllTunnelsSetup::Mix(TunnelSetup { specific_setup, .. }) => {
                // Signal back that mixnet is ready and up with all cylinders firing
                // TODO: this should actually be sent much earlier, when the mixnet client is
                // connected. However that would also require starting the status listener earlier.
                // This means that for now, we basically just ignore the status message and use the
                // NymVpnStatusMessage2 sent below instead.
                let start_status = TaskStatus::ReadyWithGateway(
                    specific_setup
                        .mixnet_connection_info
                        .entry_gateway
                        .to_base58_string(),
                );
                task_manager
                    .start_status_listener(vpn_status_tx.clone(), start_status)
                    .await;

                vpn_status_tx
                    .send(Box::new(NymVpnStatusMessage::MixnetConnectionInfo {
                        mixnet_connection_info: specific_setup.mixnet_connection_info,
                        mixnet_exit_connection_info: specific_setup.exit_connection_info,
                    }))
                    .await
                    .unwrap();

                let result = wait_for_interrupt_and_signal(Some(task_manager), vpn_ctrl_rx).await;
                handle_interrupt(route_manager, None).await;
                tokio::task::spawn_blocking(move || {
                    dns_monitor.reset().inspect_err(|err| {
                        log::error!("Failed to reset dns monitor: {err}");
                    })
                })
                .await??;
                result
            }
            AllTunnelsSetup::Wg { entry, exit } => {
                let result = wait_for_interrupt_and_signal(Some(task_manager), vpn_ctrl_rx).await;
                handle_interrupt(
                    route_manager,
                    Some([entry.specific_setup, exit.specific_setup]),
                )
                .await;
                tokio::task::spawn_blocking(move || {
                    dns_monitor.reset().inspect_err(|err| {
                        log::error!("Failed to reset dns monitor: {err}");
                    })
                })
                .await??;
                firewall.reset_policy().map_err(|err| {
                    error!("Failed to reset firewall policy: {err}");
                    NymVpnExitError::FailedToResetFirewallPolicy {
                        reason: err.to_string(),
                    }
                })?;
                result
            }
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum NymVpnStatusMessage {
    #[error("mixnet connection info")]
    MixnetConnectionInfo {
        mixnet_connection_info: MixnetConnectionInfo,
        mixnet_exit_connection_info: MixnetExitConnectionInfo,
    },
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
/// let mut vpn_config = nym_vpn_lib::NymVpn::new_mixnet_vpn(EntryPoint::Gateway { identity: NodeIdentity::from_base58_string("Qwertyuiopasdfghjklzxcvbnm1234567890").unwrap()},
/// ExitPoint::Gateway { identity: NodeIdentity::from_base58_string("Qwertyuiopasdfghjklzxcvbnm1234567890".to_string()).unwrap()});
/// vpn_config.enable_two_hop = true;
/// let vpn_handle = nym_vpn_lib::spawn_nym_vpn(vpn_config.into());
/// ```
pub fn spawn_nym_vpn(nym_vpn: SpecificVpn) -> Result<NymVpnHandle> {
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
/// let mut vpn_config = nym_vpn_lib::NymVpn::new_mixnet_vpn(EntryPoint::Gateway { identity: NodeIdentity::from_base58_string("Qwertyuiopasdfghjklzxcvbnm1234567890").unwrap()},
/// ExitPoint::Gateway { identity: NodeIdentity::from_base58_string("Qwertyuiopasdfghjklzxcvbnm1234567890".to_string()).unwrap()});
/// vpn_config.enable_two_hop = true;
/// let vpn_handle = nym_vpn_lib::spawn_nym_vpn_with_new_runtime(vpn_config.into());
/// ```
pub fn spawn_nym_vpn_with_new_runtime(nym_vpn: SpecificVpn) -> Result<NymVpnHandle> {
    let (vpn_ctrl_tx, vpn_ctrl_rx) = mpsc::unbounded();
    let (vpn_status_tx, vpn_status_rx) = mpsc::channel(128);
    let (vpn_exit_tx, vpn_exit_rx) = oneshot::channel();

    std::thread::spawn(|| {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio run time");
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
    mut nym_vpn: SpecificVpn,
    vpn_status_tx: nym_task::StatusSender,
    vpn_ctrl_rx: mpsc::UnboundedReceiver<NymVpnCtrlMessage>,
    vpn_exit_tx: oneshot::Sender<NymVpnExitStatusMessage>,
) {
    match nym_vpn.run_and_listen(vpn_status_tx, vpn_ctrl_rx).await {
        Ok(()) => {
            log::info!("Nym VPN has shut down");
            vpn_exit_tx
                .send(NymVpnExitStatusMessage::Stopped)
                .expect("Failed to send exit status");
        }
        Err(err) => {
            error!("Nym VPN returned error: {err}");
            debug!("{err:?}");
            vpn_exit_tx
                .send(NymVpnExitStatusMessage::Failed(err))
                .expect("Failed to send exit status");
        }
    }
}

pub struct NymVpnHandle {
    pub vpn_ctrl_tx: mpsc::UnboundedSender<NymVpnCtrlMessage>,
    pub vpn_status_rx: nym_task::StatusReceiver,
    pub vpn_exit_rx: oneshot::Receiver<NymVpnExitStatusMessage>,
}
