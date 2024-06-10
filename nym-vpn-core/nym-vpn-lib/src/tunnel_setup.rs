// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::error::{Error, Result};
use crate::routing::{catch_all_ipv4, catch_all_ipv6, replace_default_prefixes};
use crate::tunnel::setup_route_manager;
use crate::util::handle_interrupt;
use crate::wg_gateway_client::WgGatewayClient;
use crate::wireguard_setup::create_wireguard_tunnel;
use crate::{init_wireguard_config, MixnetVpn, SpecificVpn, WireguardVpn};
use crate::{routing, MixnetConnectionInfo, NymVpn};
use futures::channel::oneshot;
use futures::StreamExt;
use ipnetwork::IpNetwork;
use log::*;
use nym_gateway_directory::{
    GatewayClient, GatewayQueryResult, IpPacketRouterAddress, LookupGateway, NodeIdentity,
};
use nym_task::TaskManager;
use rand::rngs::OsRng;
use talpid_core::dns::DnsMonitor;
use talpid_core::firewall::Firewall;
use talpid_routing::RouteManager;
use talpid_tunnel::TunnelEvent;
use tap::TapFallible;

pub struct TunnelSetup<T: TunnelSpecifcSetup> {
    pub specific_setup: T,
}

pub trait TunnelSpecifcSetup {}

pub struct MixTunnelSetup {
    pub route_manager: RouteManager,
    pub mixnet_connection_info: MixnetConnectionInfo,
    pub task_manager: TaskManager,
    pub dns_monitor: DnsMonitor,
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
        route_manager: RouteManager,
        entry: TunnelSetup<WgTunnelSetup>,
        exit: TunnelSetup<WgTunnelSetup>,
        firewall: Firewall,
        dns_monitor: DnsMonitor,
    },
}

async fn init_firewall_dns(
    #[cfg(target_os = "linux")] route_manager_handle: talpid_routing::RouteManagerHandle,
) -> Result<(Firewall, DnsMonitor)> {
    #[cfg(target_os = "macos")]
    {
        let (command_tx, _) = futures::channel::mpsc::unbounded();
        let command_tx = std::sync::Arc::new(command_tx);
        let weak_command_tx = std::sync::Arc::downgrade(&command_tx);
        debug!("Starting firewall");
        let firewall = tokio::task::spawn_blocking(move || {
            Firewall::new().map_err(|err| crate::error::Error::FirewallError(err.to_string()))
        })
        .await??;
        debug!("Starting dns monitor");
        let dns_monitor = DnsMonitor::new(weak_command_tx)?;
        Ok((firewall, dns_monitor))
    }

    #[cfg(target_os = "linux")]
    {
        let fwmark = 0; // ?
        debug!("Starting firewall");
        let firewall = tokio::task::spawn_blocking(move || {
            Firewall::new(fwmark).map_err(|err| crate::error::Error::FirewallError(err.to_string()))
        })
        .await??;
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
        let firewall = tokio::task::spawn_blocking(move || {
            Firewall::new().map_err(|err| crate::error::Error::FirewallError(err.to_string()))
        })
        .await??;
        debug!("Starting dns monitor");
        let dns_monitor = DnsMonitor::new()?;
        Ok((firewall, dns_monitor))
    }
}

async fn setup_wg_tunnel(
    nym_vpn: &mut NymVpn<WireguardVpn>,
    route_manager: RouteManager,
    gateway_directory_client: GatewayClient,
    entry_gateway_id: NodeIdentity,
    exit_gateway_id: NodeIdentity,
) -> Result<AllTunnelsSetup> {
    let mut rng = OsRng;
    let wg_entry_gateway_client = WgGatewayClient::new(&mut rng);
    let wg_exit_gateway_client = WgGatewayClient::new(&mut rng);
    log::info!("Created wg gateway client");
    // MTU is computed as (MTU of wire interface) - ((IP header size) + (UDP header size) + (WireGuard metadata size))
    // The IP header size is 20 for IPv4 and 40 for IPv6
    // The UDP header size is 8
    // The Wireguard metadata size is 32
    // Entry tunnel will only deal with IPv4 => 1500 - (20 + 8 + 32)
    let entry_mtu = 1440;
    // Exit tunnel will deal with both v4 and v6, and it's "wire" interface is entry tunnel's MTU
    // 1440 - (40 + 8 + 32)
    let exit_mtu = 1360;

    let mut entry_wireguard_config = init_wireguard_config(
        &gateway_directory_client,
        &wg_entry_gateway_client,
        &entry_gateway_id.to_base58_string(),
        entry_mtu,
    )
    .await?;
    let mut exit_wireguard_config = init_wireguard_config(
        &gateway_directory_client,
        &wg_exit_gateway_client,
        &exit_gateway_id.to_base58_string(),
        exit_mtu,
    )
    .await?;
    entry_wireguard_config.0.peers.iter_mut().for_each(|peer| {
        peer.allowed_ips.append(
            &mut exit_wireguard_config
                .0
                .peers
                .iter()
                .map(|peer| IpNetwork::from(peer.endpoint.ip()))
                .collect::<Vec<_>>(),
        );
    });
    exit_wireguard_config.0.peers.iter_mut().for_each(|peer| {
        peer.allowed_ips
            .append(&mut replace_default_prefixes(catch_all_ipv4()));
        peer.allowed_ips
            .append(&mut replace_default_prefixes(catch_all_ipv6()));
    });
    info!("Entry wireguard config: \n{entry_wireguard_config}");
    info!("Exit wireguard config: \n{exit_wireguard_config}");
    let (firewall, dns_monitor) = init_firewall_dns(
        #[cfg(target_os = "linux")]
        route_manager.handle()?,
    )
    .await?;
    std::env::set_var("TALPID_FORCE_USERSPACE_WIREGUARD", "1");
    let (wireguard_waiting_entry, mut event_rx) = create_wireguard_tunnel(
        &route_manager,
        nym_vpn.tun_provider.clone(),
        entry_wireguard_config,
    )
    .await?;
    // Wait for entry gateway routes to be finished before moving to exit gateway routes, as the two might race if
    // started one after the other
    loop {
        match event_rx.next().await {
            Some((TunnelEvent::InterfaceUp(_, _), _)) => {
                continue;
            }
            Some((TunnelEvent::Up(_), _)) => {
                break;
            }
            Some((TunnelEvent::AuthFailed(_), _)) | Some((TunnelEvent::Down, _)) | None => {
                return Err(Error::BadWireguardEvent);
            }
        }
    }
    let (wireguard_waiting_exit, _) = create_wireguard_tunnel(
        &route_manager,
        nym_vpn.tun_provider.clone(),
        exit_wireguard_config,
    )
    .await?;
    let entry = TunnelSetup {
        specific_setup: wireguard_waiting_entry,
    };
    let exit = TunnelSetup {
        specific_setup: wireguard_waiting_exit,
    };

    Ok(AllTunnelsSetup::Wg {
        route_manager,
        entry,
        exit,
        firewall,
        dns_monitor,
    })
}

async fn setup_mix_tunnel(
    nym_vpn: &mut NymVpn<MixnetVpn>,
    mut task_manager: TaskManager,
    mut route_manager: RouteManager,
    gateway_directory_client: GatewayClient,
    entry_gateway_id: NodeIdentity,
    exit_router_address: IpPacketRouterAddress,
    default_lan_gateway_ip: routing::LanGatewayIp,
) -> Result<AllTunnelsSetup> {
    info!("Wireguard is disabled");
    let (mut firewall, mut dns_monitor) = init_firewall_dns(
        #[cfg(target_os = "linux")]
        route_manager.handle()?,
    )
    .await?;

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
            &mut dns_monitor,
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
            handle_interrupt(route_manager, None)
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
            dns_monitor,
        },
    }))
}

pub async fn setup_tunnel(nym_vpn: &mut SpecificVpn) -> Result<AllTunnelsSetup> {
    // Create a gateway client that we use to interact with the entry gateway, in particular to
    // handle wireguard registration
    let gateway_directory_client = GatewayClient::new(nym_vpn.gateway_config()).map_err(|err| {
        Error::FailedtoSetupGatewayDirectoryClient {
            config: Box::new(nym_vpn.gateway_config()),
            source: err,
        }
    })?;
    let GatewayQueryResult {
        entry_gateways,
        exit_gateways,
    } = gateway_directory_client
        .lookup_described_entry_and_exit_gateways_with_location()
        .await
        .map_err(|err| Error::FailedToLookupGateways { source: err })?;

    // This info would be useful at at least debug level, but it's just so much data that it
    // would be overwhelming
    log::trace!("Got entry gateways {:?}", entry_gateways);
    log::trace!("Got exit gateways {:?}", exit_gateways);

    let (entry_gateway_id, entry_location) = nym_vpn
        .entry_point()
        .lookup_gateway_identity(&entry_gateways)
        .await
        .map_err(|err| Error::FailedToLookupGatewayIdentity { source: err })?;
    let entry_location_str = entry_location.as_deref().unwrap_or("unknown");

    let (exit_router_address, exit_location) = nym_vpn
        .exit_point()
        .lookup_router_address(&exit_gateways)
        .map_err(|err| Error::FailedToLookupRouterAddress { source: err })?;
    let exit_location_str = exit_location.as_deref().unwrap_or("unknown");
    let exit_gateway_id = exit_router_address.gateway();

    info!("Using entry gateway: {entry_gateway_id}, location: {entry_location_str}");
    info!("Using exit gateway: {exit_gateway_id}, location: {exit_location_str}");
    info!("Using exit router address {exit_router_address}");

    // Get the IP address of the local LAN gateway
    let default_lan_gateway_ip = routing::LanGatewayIp::get_default_interface()?;
    debug!("default_lan_gateway_ip: {default_lan_gateway_ip}");

    let task_manager = TaskManager::new(10).named("nym_vpn_lib");
    info!("Setting up route manager");
    let route_manager = setup_route_manager().await?;

    match nym_vpn {
        SpecificVpn::Wg(vpn) => {
            setup_wg_tunnel(
                vpn,
                route_manager,
                gateway_directory_client,
                entry_gateway_id,
                *exit_gateway_id,
            )
            .await
        }
        SpecificVpn::Mix(vpn) => {
            setup_mix_tunnel(
                vpn,
                task_manager,
                route_manager,
                gateway_directory_client,
                entry_gateway_id,
                exit_router_address,
                default_lan_gateway_ip,
            )
            .await
        }
    }
}
