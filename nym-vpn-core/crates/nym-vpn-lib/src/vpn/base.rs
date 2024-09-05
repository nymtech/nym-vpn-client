// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    net::IpAddr,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use futures::{channel::mpsc, SinkExt};
use nym_gateway_directory::{Config as GatewayDirectoryConfig, EntryPoint, ExitPoint};
use nym_ip_packet_requests::IpPair;
use nym_sdk::UserAgent;
use nym_task::{manager::TaskStatus, TaskManager};
use talpid_core::{dns::DnsMonitor, firewall::Firewall};
use talpid_tunnel::tun_provider::TunProvider;
use tokio::task::JoinHandle;
use tracing::{error, info};
use tun2::AsyncDevice;

use super::{
    messages::NymVpnStatusMessage,
    mixnet::{MixnetClientConfig, MixnetVpn},
    wireguard::WireguardVpn,
};
#[cfg(target_os = "ios")]
use crate::mobile::ios::tun_provider::OSTunProvider;
#[cfg(target_os = "android")]
use crate::platform::android::AndroidTunProvider;
use crate::{
    error::Result,
    tunnel_setup::{AllTunnelsSetup, TunnelSetup},
    MixnetError,
};

pub(crate) const MIXNET_CLIENT_STARTUP_TIMEOUT_SECS: u64 = 30;
const SHUTDOWN_TIMER_SECS: u64 = 10;

pub struct GenericNymVpnConfig {
    pub mixnet_client_config: MixnetClientConfig,

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

    #[cfg(target_os = "android")]
    pub android_tun_provider: Arc<dyn AndroidTunProvider>,

    #[cfg(target_os = "ios")]
    pub(super) ios_tun_provider: Arc<dyn OSTunProvider>,

    // Necessary so that the device doesn't get closed before cleanup has taken place
    // Observation: this seems only used for mixnet mode? If so, can we move it to MixnetVpn?
    pub(super) shadow_handle: ShadowHandle,
}

pub(super) struct ShadowHandle {
    pub(super) _inner: Option<JoinHandle<std::result::Result<AsyncDevice, MixnetError>>>,
}

impl<T: Vpn> NymVpn<T> {
    pub(crate) fn set_shadow_handle(
        &mut self,
        shadow_handle: JoinHandle<std::result::Result<AsyncDevice, MixnetError>>,
    ) {
        self.shadow_handle = ShadowHandle {
            _inner: Some(shadow_handle),
        }
    }
}

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

impl SpecificVpn {
    pub fn mixnet_client_config(&self) -> MixnetClientConfig {
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

    // Start the Nym VPN client, but also listen for external messages to e.g. disconnect as well
    // as reporting it's status on the provided channel.
    pub async fn run(
        &mut self,
        mut vpn_status_tx: nym_task::StatusSender,
        vpn_ctrl_rx: mpsc::UnboundedReceiver<super::NymVpnCtrlMessage>,
    ) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let mut task_manager = TaskManager::new(SHUTDOWN_TIMER_SECS).named("nym_vpn_lib");
        info!("Setting up route manager");
        let mut route_manager = crate::tunnel::setup_route_manager().await?;
        let (mut firewall, mut dns_monitor) = init_firewall_dns(
            #[cfg(target_os = "linux")]
            route_manager.handle()?,
        )
        .await?;

        let tunnels = match crate::tunnel_setup::setup_tunnel(
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
                            error!("Failed to reset dns monitor: {err}");
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
                // NymVpnStatusMessage sent below instead.
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

                // We are operational, wait for exit
                let result = crate::util::wait_for_interrupt(
                    Some(task_manager),
                    Some(vpn_ctrl_rx),
                    route_manager,
                    None,
                )
                .await;

                tokio::task::spawn_blocking(move || {
                    dns_monitor.reset().inspect_err(|err| {
                        error!("Failed to reset dns monitor: {err}");
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

                // We are operational, wait for exit
                let result = crate::util::wait_for_interrupt(
                    Some(task_manager),
                    Some(vpn_ctrl_rx),
                    route_manager,
                    Some([entry.specific_setup, exit.specific_setup]),
                )
                .await;

                tokio::task::spawn_blocking(move || {
                    dns_monitor.reset().inspect_err(|err| {
                        error!("Failed to reset dns monitor: {err}");
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
    // TODO: capture the concrete error type once we have time to investigate on Mac
    #[error("failed to reset firewall policy: {reason}")]
    FailedToResetFirewallPolicy { reason: String },
}

async fn init_firewall_dns(
    #[cfg(target_os = "linux")] route_manager_handle: talpid_routing::RouteManagerHandle,
) -> Result<(Firewall, DnsMonitor)> {
    #[cfg(target_os = "macos")]
    {
        let (command_tx, _) = futures::channel::mpsc::unbounded();
        let command_tx = std::sync::Arc::new(command_tx);
        let weak_command_tx = std::sync::Arc::downgrade(&command_tx);
        tracing::debug!("Starting firewall");
        let firewall = tokio::task::spawn_blocking(move || {
            Firewall::new().map_err(|err| crate::error::Error::FirewallError(err.to_string()))
        })
        .await??;
        tracing::debug!("Starting dns monitor");
        let dns_monitor = DnsMonitor::new(weak_command_tx)?;
        Ok((firewall, dns_monitor))
    }

    #[cfg(target_os = "linux")]
    {
        let fwmark = 0; // ?
        tracing::debug!("Starting firewall");
        let firewall = tokio::task::spawn_blocking(move || {
            Firewall::new(fwmark).map_err(|err| crate::error::Error::FirewallError(err.to_string()))
        })
        .await??;
        tracing::debug!("Starting dns monitor");
        let dns_monitor = DnsMonitor::new(
            tokio::runtime::Handle::current(),
            route_manager_handle.clone(),
        )?;
        Ok((firewall, dns_monitor))
    }

    #[cfg(all(not(target_os = "macos"), not(target_os = "linux")))]
    {
        tracing::debug!("Starting firewall");
        let firewall = tokio::task::spawn_blocking(move || {
            Firewall::new().map_err(|err| crate::error::Error::FirewallError(err.to_string()))
        })
        .await??;
        tracing::debug!("Starting dns monitor");
        let dns_monitor = DnsMonitor::new()?;
        Ok((firewall, dns_monitor))
    }
}
