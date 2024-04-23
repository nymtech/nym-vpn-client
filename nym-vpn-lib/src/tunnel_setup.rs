// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::{Arc, RwLock};

use crate::error::{Error, Result};
use crate::routing::setup_wg_routing;
use crate::tunnel::setup_route_manager;
use crate::util::handle_interrupt;
use crate::wg_gateway_client::WgGatewayClient;
use crate::wireguard_setup::create_wireguard_tunnel;
use crate::{routing, MixnetConnectionInfo, NymVpn};
use futures::channel::oneshot;
use log::*;
use nym_gateway_directory::{GatewayClient, GatewayQueryResult, LookupGateway};
use nym_task::TaskManager;
use talpid_core::dns::DnsMonitor;
use talpid_core::firewall::Firewall;
use talpid_routing::RouteManager;
use tap::TapFallible;

pub struct TunnelSetup<T: TunnelSpecifcSetup> {
    pub specific_setup: T,
}

pub trait TunnelSpecifcSetup {}

pub struct MixTunnelSetup {
    pub route_manager: RouteManager,
    pub mixnet_connection_info: MixnetConnectionInfo,
    pub task_manager: TaskManager,
}

impl TunnelSpecifcSetup for MixTunnelSetup {}

pub struct WgTunnelSetup {
    pub receiver: oneshot::Receiver<()>,
    pub tunnel_close_tx: oneshot::Sender<()>,
    pub handle: tokio::task::JoinHandle<Result<()>>,
}

impl TunnelSpecifcSetup for WgTunnelSetup {}

pub enum AllTunnelsSetup {
    Mix(TunnelSetup<MixTunnelSetup>),
    Wg {
        route_manager: Arc<RwLock<RouteManager>>,
        entry: TunnelSetup<WgTunnelSetup>,
        exit: TunnelSetup<WgTunnelSetup>,
        firewall: Firewall,
        dns_monitor: DnsMonitor,
    },
}

fn init_firewall_dns(
    #[cfg(target_os = "linux")] route_manager_handle: talpid_routing::RouteManagerHandle,
) -> Result<(Firewall, DnsMonitor)> {
    #[cfg(target_os = "macos")]
    {
        let (command_tx, _) = futures::channel::mpsc::unbounded();
        let command_tx = std::sync::Arc::new(command_tx);
        let weak_command_tx = std::sync::Arc::downgrade(&command_tx);
        debug!("Starting firewall");
        let firewall =
            Firewall::new().map_err(|err| crate::error::Error::FirewallError(err.to_string()))?;
        debug!("Starting dns monitor");
        let dns_monitor = DnsMonitor::new(weak_command_tx)?;
        Ok((firewall, dns_monitor))
    }

    #[cfg(target_os = "linux")]
    {
        let fwmark = 0; // ?
        debug!("Starting firewall");
        let firewall = Firewall::new(fwmark)
            .map_err(|err| crate::error::Error::FirewallError(err.to_string()))?;
        debug!("Starting dns monitor");
        let dns_monitor = DnsMonitor::new(
            tokio::runtime::Handle::current(),
            route_manager_handle.clone(),
        )?;
        Ok((firewall, dns_monitor))
    }

    #[cfg(all(not(target_os = "macos"), not(target_os = "linux")))]
    {
        debug!("Starting firewall");
        let firewall =
            Firewall::new().map_err(|err| crate::error::Error::FirewallError(err.to_string()))?;
        debug!("Starting dns monitor");
        let dns_monitor = DnsMonitor::new()?;
        Ok((firewall, dns_monitor))
    }
}

pub async fn setup_tunnel(nym_vpn: &mut NymVpn) -> Result<AllTunnelsSetup> {
    // Create a gateway client that we use to interact with the entry gateway, in particular to
    // handle wireguard registration
    let gateway_directory_client = GatewayClient::new(nym_vpn.gateway_config.clone())?;
    let GatewayQueryResult {
        entry_gateways,
        exit_gateways,
    } = gateway_directory_client
        .lookup_described_entry_and_exit_gateways_with_location()
        .await?;

    // This info would be useful at at least debug level, but it's just so much data that it
    // would be overwhelming
    log::trace!("Got entry gateways {:?}", entry_gateways);
    log::trace!("Got exit gateways {:?}", exit_gateways);

    let wg_gateway_client = WgGatewayClient::new(nym_vpn.wg_gateway_config.clone())?;
    log::info!("Created wg gateway client");

    let (entry_gateway_id, entry_location) = nym_vpn
        .entry_point
        .lookup_gateway_identity(&entry_gateways)
        .await?;
    let entry_location_str = entry_location.as_deref().unwrap_or("unknown");

    let (exit_router_address, exit_location) =
        nym_vpn.exit_point.lookup_router_address(&exit_gateways)?;
    let exit_location_str = exit_location.as_deref().unwrap_or("unknown");
    let exit_gateway_id = exit_router_address.gateway();

    info!("Using entry gateway: {entry_gateway_id}, location: {entry_location_str}");
    info!("Using exit gateway: {exit_gateway_id}, location: {exit_location_str}");
    info!("Using exit router address {exit_router_address}");

    // Get the IP address of the local LAN gateway
    let default_lan_gateway_ip = routing::LanGatewayIp::get_default_interface()?;
    debug!("default_lan_gateway_ip: {default_lan_gateway_ip}");

    let mut task_manager = TaskManager::new(10).named("nym_vpn_lib");
    info!("Setting up route manager");
    let mut route_manager = setup_route_manager().await?;

    if nym_vpn.enable_wireguard {
        let route_manager = Arc::new(RwLock::new(route_manager));
        let (wireguard_waiting_exit, tunnel_exit) = create_wireguard_tunnel(
            nym_vpn
                .exit_private_key
                .as_ref()
                .expect("clap should enforce value when wireguard enabled"),
            nym_vpn
                .exit_wg_ip
                .expect("clap should enforce value when wireguard enabled"),
            route_manager.clone(),
            nym_vpn.tun_provider.clone(),
            &gateway_directory_client,
            &wg_gateway_client,
            exit_gateway_id,
        )
        .await?;
        let (firewall, dns_monitor) = init_firewall_dns(
            #[cfg(target_os = "linux")]
            tunnel_exit.route_manager_handle.clone(),
        )?;
        let (wireguard_waiting_entry, tunnel_entry) = create_wireguard_tunnel(
            nym_vpn
                .entry_private_key
                .as_ref()
                .expect("clap should enforce value when wireguard enabled"),
            nym_vpn
                .entry_wg_ip
                .expect("clap should enforce value when wireguard enabled"),
            route_manager.clone(),
            nym_vpn.tun_provider.clone(),
            &gateway_directory_client,
            &wg_gateway_client,
            &entry_gateway_id,
        )
        .await?;
        route_manager
            .write()
            .map_err(|_| Error::RouteManagerPoisonedLock)?
            .clear_routes()?;
        setup_wg_routing(
            tunnel_entry.config.clone(),
            tunnel_exit.config.clone(),
            tunnel_entry.route_manager_handle.clone(),
            tunnel_exit.route_manager_handle.clone(),
        )
        .await?;
        let entry = TunnelSetup {
            specific_setup: WgTunnelSetup {
                tunnel_close_tx: wireguard_waiting_entry.tunnel_close_tx,
                receiver: wireguard_waiting_entry.receiver,
                handle: wireguard_waiting_entry.handle,
            },
        };
        let exit = TunnelSetup {
            specific_setup: WgTunnelSetup {
                tunnel_close_tx: wireguard_waiting_exit.tunnel_close_tx,
                receiver: wireguard_waiting_exit.receiver,
                handle: wireguard_waiting_exit.handle,
            },
        };

        Ok(AllTunnelsSetup::Wg {
            route_manager,
            entry,
            exit,
            firewall,
            dns_monitor,
        })
    } else {
        info!("Wireguard is disabled");
        let (mut firewall, mut dns_monitor) = init_firewall_dns(
            #[cfg(target_os = "linux")]
            route_manager.handle()?,
        )?;

        // Now it's time start all the stuff that needs running inside the tunnel, and that we need
        // correctly unwind if it fails
        // - Sets up mixnet client, and connects
        // - Sets up routing
        // - Starts processing packets
        let ret = nym_vpn
            .setup_tunnel_services(
                &mut route_manager,
                &entry_gateway_id,
                &exit_router_address,
                &task_manager,
                &gateway_directory_client,
                default_lan_gateway_ip,
            )
            .await;
        let mixnet_connection_info = match ret {
            Ok(mixnet_connection_info) => mixnet_connection_info,
            Err(err) => {
                error!("Failed to setup tunnel services: {err}");
                debug!("{err:?}");
                task_manager.signal_shutdown().ok();
                task_manager.wait_for_shutdown().await;
                info!("Interrupt handled");
                // Ignore if these fail since we're interesting in the original error anyway
                handle_interrupt(Arc::new(RwLock::new(route_manager)), None)
                    .await
                    .tap_err(|err| {
                        warn!("Failed to handle interrupt: {err}");
                    })
                    .ok();
                dns_monitor
                    .reset()
                    .tap_err(|err| {
                        warn!("Failed to reset dns monitor: {err}");
                    })
                    .ok();
                firewall
                    .reset_policy()
                    .tap_err(|err| {
                        warn!("Failed to reset firewall policy: {err}");
                    })
                    .ok();
                return Err(err);
            }
        };

        Ok(AllTunnelsSetup::Mix(TunnelSetup {
            specific_setup: MixTunnelSetup {
                route_manager,
                mixnet_connection_info,
                task_manager,
            },
        }))
    }
}
