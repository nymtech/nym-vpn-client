// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod messages;
mod mixnet;
mod start;
mod wireguard;

use std::{
    net::IpAddr,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use futures::{channel::mpsc, SinkExt};
use log::{error, info};
use nym_gateway_directory::{Config as GatewayDirectoryConfig, EntryPoint, ExitPoint};
use nym_ip_packet_requests::IpPair;
use nym_sdk::UserAgent;
use nym_task::{manager::TaskStatus, TaskManager};
use talpid_tunnel::tun_provider::TunProvider;
use tokio::task::JoinHandle;
use tun2::AsyncDevice;

#[cfg(target_os = "ios")]
use crate::platform::swift::OSTunProvider;
use crate::{
    error::Result,
    tunnel,
    tunnel_setup::{init_firewall_dns, setup_tunnel, AllTunnelsSetup, TunnelSetup},
    util::{wait_and_handle_interrupt, wait_for_interrupt_and_signal},
    Error,
};

pub use messages::{NymVpnCtrlMessage, NymVpnExitStatusMessage, NymVpnStatusMessage};
pub use mixnet::{MixnetClientConfig, MixnetConnectionInfo, MixnetExitConnectionInfo, MixnetVpn};
pub use start::{spawn_nym_vpn, spawn_nym_vpn_with_new_runtime, NymVpnHandle};
pub use wireguard::{WireguardConnectionInfo, WireguardVpn};

pub(crate) const MIXNET_CLIENT_STARTUP_TIMEOUT_SECS: u64 = 30;
pub const SHUTDOWN_TIMER_SECS: u64 = 10;

pub struct GenericNymVpnConfig {
    pub mixnet_client_config: mixnet::MixnetClientConfig,

    /// Path to the data directory, where keys reside.
    pub data_path: Option<PathBuf>,

    /// Gateway configuration
    pub gateway_config: GatewayDirectoryConfig,

    /// Mixnet public ID of the entry gateway.
    pub entry_point: EntryPoint,

    /// Mixnet recipient address.
    pub exit_point: ExitPoint,

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
}

pub trait Vpn {}

pub struct NymVpn<T: Vpn> {
    /// VPN configuration, independent of the type used
    pub generic_config: GenericNymVpnConfig,

    /// VPN configuration, depending on the type used
    pub vpn_config: T,

    pub tun_provider: Arc<Mutex<TunProvider>>,

    #[cfg(target_os = "ios")]
    ios_tun_provider: Arc<dyn OSTunProvider>,

    // Necessary so that the device doesn't get closed before cleanup has taken place
    shadow_handle: ShadowHandle,
}

struct ShadowHandle {
    _inner: Option<JoinHandle<Result<AsyncDevice>>>,
}

impl<T: Vpn> NymVpn<T> {
    pub(crate) fn set_shadow_handle(&mut self, shadow_handle: JoinHandle<Result<AsyncDevice>>) {
        self.shadow_handle = ShadowHandle {
            _inner: Some(shadow_handle),
        }
    }
}

pub enum SpecificVpn {
    Wg(NymVpn<wireguard::WireguardVpn>),
    Mix(NymVpn<mixnet::MixnetVpn>),
}

impl From<NymVpn<wireguard::WireguardVpn>> for SpecificVpn {
    fn from(value: NymVpn<wireguard::WireguardVpn>) -> Self {
        Self::Wg(value)
    }
}

impl From<NymVpn<mixnet::MixnetVpn>> for SpecificVpn {
    fn from(value: NymVpn<mixnet::MixnetVpn>) -> Self {
        Self::Mix(value)
    }
}

impl SpecificVpn {
    pub fn mixnet_client_config(&self) -> mixnet::MixnetClientConfig {
        match self {
            SpecificVpn::Wg(vpn) => vpn.generic_config.mixnet_client_config.clone(),
            SpecificVpn::Mix(vpn) => vpn.generic_config.mixnet_client_config.clone(),
        }
    }

    pub fn data_path(&self) -> Option<PathBuf> {
        match self {
            SpecificVpn::Wg(vpn) => vpn.generic_config.data_path.clone(),
            SpecificVpn::Mix(vpn) => vpn.generic_config.data_path.clone(),
        }
    }

    pub fn gateway_config(&self) -> GatewayDirectoryConfig {
        match self {
            SpecificVpn::Wg(vpn) => vpn.generic_config.gateway_config.clone(),
            SpecificVpn::Mix(vpn) => vpn.generic_config.gateway_config.clone(),
        }
    }

    pub fn entry_point(&self) -> EntryPoint {
        match self {
            SpecificVpn::Wg(vpn) => vpn.generic_config.entry_point.clone(),
            SpecificVpn::Mix(vpn) => vpn.generic_config.entry_point.clone(),
        }
    }

    pub fn exit_point(&self) -> ExitPoint {
        match self {
            SpecificVpn::Wg(vpn) => vpn.generic_config.exit_point.clone(),
            SpecificVpn::Mix(vpn) => vpn.generic_config.exit_point.clone(),
        }
    }

    pub fn user_agent(&self) -> Option<UserAgent> {
        match self {
            SpecificVpn::Wg(vpn) => vpn.generic_config.user_agent.clone(),
            SpecificVpn::Mix(vpn) => vpn.generic_config.user_agent.clone(),
        }
    }

    // Start the Nym VPN client, and wait for it to shutdown. The use case is in simple console
    // applications where the main way to interact with the running process is to send SIGINT
    // (ctrl-c)
    pub async fn run(&mut self) -> Result<()> {
        let mut task_manager = TaskManager::new(SHUTDOWN_TIMER_SECS).named("nym_vpn_lib");
        info!("Setting up route manager");
        let mut route_manager = tunnel::setup_route_manager().await?;
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
                wait_and_handle_interrupt(&mut task_manager, route_manager, None).await;
                tokio::task::spawn_blocking(move || {
                    dns_monitor.reset().inspect_err(|err| {
                        log::error!("Failed to reset dns monitor: {err}");
                    })
                })
                .await??;
            }
            AllTunnelsSetup::Wg { entry, exit } => {
                wait_and_handle_interrupt(
                    &mut task_manager,
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
        let mut route_manager = tunnel::setup_route_manager().await?;
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

        info!("Nym VPN is now running");

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
                    .send(Box::new(NymVpnStatusMessage::MixConnectionInfo {
                        mixnet_connection_info: specific_setup.mixnet_connection_info,
                        mixnet_exit_connection_info: Box::new(specific_setup.exit_connection_info),
                    }))
                    .await
                    .unwrap();

                let result = wait_for_interrupt_and_signal(
                    Some(task_manager),
                    vpn_ctrl_rx,
                    route_manager,
                    None,
                )
                .await;
                tokio::task::spawn_blocking(move || {
                    dns_monitor.reset().inspect_err(|err| {
                        log::error!("Failed to reset dns monitor: {err}");
                    })
                })
                .await??;
                result
            }
            AllTunnelsSetup::Wg { entry, exit } => {
                let start_status = TaskStatus::ReadyWithGateway(
                    entry
                        .specific_setup
                        .connection_info
                        .gateway_id
                        .to_base58_string(),
                );
                task_manager
                    .start_status_listener(vpn_status_tx.clone(), start_status)
                    .await;

                vpn_status_tx
                    .send(Box::new(NymVpnStatusMessage::WgConnectionInfo {
                        entry_connection_info: entry.specific_setup.connection_info.clone(),
                        exit_connection_info: exit.specific_setup.connection_info.clone(),
                    }))
                    .await
                    .unwrap();
                let result = wait_for_interrupt_and_signal(
                    Some(task_manager),
                    vpn_ctrl_rx,
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
