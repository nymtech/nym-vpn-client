// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::error::{Error, Result};
use crate::routing::setup_wg_routing;
use crate::util::{handle_interrupt, wait_for_interrupt};
use crate::wg_gateway_client::WgGatewayClient;
use crate::wireguard_setup::{create_wireguard_tunnel, empty_wireguard_setup};
use crate::{routing, MixnetConnectionInfo, NymVpn};
use futures::channel::oneshot;
use log::*;
use log::{debug, error, info};
use nym_gateway_directory::{DescribedGatewayWithLocation, GatewayClient, LookupGateway};
use nym_task::TaskManager;
use talpid_core::dns::DnsMonitor;
use talpid_core::firewall::Firewall;
use talpid_routing::RouteManager;
use tap::TapFallible;

pub struct TunnelSetup<T: TunnelSpecifcSetup> {
    pub route_manager: RouteManager,
    pub tunnel_close_tx: oneshot::Sender<()>,
    pub specific_setup: T,
}

pub trait TunnelSpecifcSetup {}

pub struct MixTunnelSetup {
    pub mixnet_connection_info: MixnetConnectionInfo,
    pub task_manager: TaskManager,
}

impl TunnelSpecifcSetup for MixTunnelSetup {}

pub struct WgTunnelSetup {
    pub receiver: oneshot::Receiver<()>,
    pub handle: tokio::task::JoinHandle<Result<()>>,
}

impl TunnelSpecifcSetup for WgTunnelSetup {}

pub enum AllTunnelsSetup {
    Mix(TunnelSetup<MixTunnelSetup>),
    Wg {
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
    let gateway_client = GatewayClient::new(nym_vpn.gateway_config.clone())?;
    let gateways = gateway_client
        .lookup_described_gateways_with_location()
        .await?;
    // This info would be useful at at least debug level, but it's just so much data that it
    // would be overwhelming
    log::trace!("Got gateways {:?}", gateways);

    let wg_gateway_client = WgGatewayClient::new(nym_vpn.wg_gateway_config.clone())?;
    log::info!("Created wg gateway client");

    // If the entry or exit point relies on location, do a basic defensive consistency check on
    // the fetched location data. If none of the gateways have location data, we can't proceed
    // and it's likely the explorer-api isn't set correctly.
    if (nym_vpn.entry_point.is_location() || nym_vpn.exit_point.is_location())
        && gateways.iter().filter(|g| g.has_location()).count() == 0
    {
        return Err(Error::RequestedGatewayByLocationWithoutLocationDataAvailable);
    }

    //filter so we are only getting exit gateways with current api version
    let working_exit_gateways: Vec<DescribedGatewayWithLocation> = gateways
        .clone()
        .into_iter()
        .filter(|gateway| gateway.is_current_build())
        .collect();

    if working_exit_gateways.is_empty() {
        return Err(Error::CountryExitGatewaysOutdated);
    }

    let (entry_gateway_id, entry_location) = nym_vpn
        .entry_point
        .lookup_gateway_identity(&gateways)
        .await?;
    let entry_location_str = entry_location.as_deref().unwrap_or("unknown");
    let (exit_gateway_id, exit_location) = nym_vpn
        .exit_point
        .lookup_gateway_identity(&working_exit_gateways)
        .await?;
    let exit_location_str = exit_location.as_deref().unwrap_or("unknown");

    info!("Using entry gateway: {entry_gateway_id}, location: {entry_location_str}");
    info!("Using exit gateway: {exit_gateway_id}, location: {exit_location_str}");

    // Get the IP address of the local LAN gateway
    let default_lan_gateway_ip = routing::LanGatewayIp::get_default_interface()?;
    debug!("default_lan_gateway_ip: {default_lan_gateway_ip}");

    let task_manager = TaskManager::new(10);

    if nym_vpn.enable_wireguard {
        let (mut wireguard_setup_exit, wireguard_waiting_exit, tunnel_exit) =
            create_wireguard_tunnel(
                nym_vpn
                    .exit_private_key
                    .as_ref()
                    .expect("clap should enforce value when wireguard enabled"),
                nym_vpn
                    .exit_wg_ip
                    .expect("clap should enforce value when wireguard enabled"),
                nym_vpn.tun_provider.clone(),
                &gateway_client,
                &wg_gateway_client,
                &exit_gateway_id,
            )
            .await?;
        let (firewall, dns_monitor) = init_firewall_dns(
            #[cfg(target_os = "linux")]
            tunnel_exit.route_manager_handle.clone(),
        )?;
        let (wireguard_setup_entry, wireguard_waiting_entry, tunnel_entry) =
            create_wireguard_tunnel(
                nym_vpn
                    .entry_private_key
                    .as_ref()
                    .expect("clap should enforce value when wireguard enabled"),
                nym_vpn
                    .entry_wg_ip
                    .expect("clap should enforce value when wireguard enabled"),
                nym_vpn.tun_provider.clone(),
                &gateway_client,
                &wg_gateway_client,
                &entry_gateway_id,
            )
            .await?;
        wireguard_setup_exit.route_manager.clear_routes()?;
        setup_wg_routing(
            tunnel_entry.config.clone(),
            tunnel_exit.config.clone(),
            tunnel_entry.route_manager_handle.clone(),
            tunnel_exit.route_manager_handle.clone(),
        )
        .await?;
        let entry = TunnelSetup {
            route_manager: wireguard_setup_entry.route_manager,
            tunnel_close_tx: wireguard_setup_entry.tunnel_close_tx,
            specific_setup: WgTunnelSetup {
                receiver: wireguard_waiting_entry.receiver,
                handle: wireguard_waiting_entry.handle,
            },
        };
        let exit = TunnelSetup {
            route_manager: wireguard_setup_exit.route_manager,
            tunnel_close_tx: wireguard_setup_exit.tunnel_close_tx,
            specific_setup: WgTunnelSetup {
                receiver: wireguard_waiting_exit.receiver,
                handle: wireguard_waiting_exit.handle,
            },
        };

        Ok(AllTunnelsSetup::Wg {
            entry,
            exit,
            firewall,
            dns_monitor,
        })
    } else {
        info!("Wireguard is disabled");
        let (mut wireguard_setup, _) = empty_wireguard_setup().await?;
        let (mut firewall, mut dns_monitor) = init_firewall_dns(
            #[cfg(target_os = "linux")]
            wireguard_setup.route_manager.handle()?,
        )?;

        // Now it's time start all the stuff that needs running inside the tunnel, and that we need
        // correctly unwind if it fails
        // - Sets up mixnet client, and connects
        // - Sets up routing
        // - Starts processing packets

        let (exit_router_address, _) = nym_vpn.exit_point.lookup_router_address(&gateways)?;

        let mixnet_connection_info = match nym_vpn
            .setup_tunnel_services(
                &mut wireguard_setup.route_manager,
                &entry_gateway_id,
                &exit_router_address,
                &task_manager,
                &gateway_client,
                default_lan_gateway_ip,
                wireguard_setup.tunnel_gateway_ip,
            )
            .await
        {
            Ok(mixnet_connection_info) => mixnet_connection_info,
            Err(err) => {
                error!("Failed to setup tunnel services: {err}");
                debug!("{err:?}");
                wait_for_interrupt(task_manager).await;
                // Ignore if these fail since we're interesting in the original error anyway
                handle_interrupt(
                    wireguard_setup.route_manager,
                    None,
                    wireguard_setup.tunnel_close_tx,
                )
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
            route_manager: wireguard_setup.route_manager,
            tunnel_close_tx: wireguard_setup.tunnel_close_tx,
            specific_setup: MixTunnelSetup {
                mixnet_connection_info,
                task_manager,
            },
        }))
    }
}
