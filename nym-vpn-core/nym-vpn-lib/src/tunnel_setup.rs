// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use crate::error::{Error, Result};
use crate::mixnet_connect::SharedMixnetClient;
use crate::routing::{catch_all_ipv4, catch_all_ipv6, replace_default_prefixes};
use crate::tunnel::setup_route_manager;
use crate::util::handle_interrupt;
use crate::wg_gateway_client::WgGatewayClient;
use crate::wireguard_setup::create_wireguard_tunnel;
use crate::{init_wireguard_config, WireguardVpn, MIXNET_CLIENT_STARTUP_TIMEOUT_SECS};
use crate::{routing, MixnetConnectionInfo, NymVpn};
use crate::{MixnetExitConnectionInfo, MixnetVpn, SpecificVpn};
use futures::channel::{mpsc, oneshot};
use futures::StreamExt;
use ipnetwork::IpNetwork;
use log::*;
use nym_bin_common::bin_info;
use nym_gateway_directory::{
    extract_authenticator, extract_router_address, AuthAddresses, GatewayClient,
    GatewayQueryResult, IpPacketRouterAddress, LookupGateway,
};
use nym_task::TaskManager;
use rand::rngs::OsRng;
use talpid_core::dns::DnsMonitor;
use talpid_core::firewall::Firewall;
use talpid_routing::RouteManager;
use talpid_tunnel::{TunnelEvent, TunnelMetadata};
use tap::TapFallible;
use tokio::time::timeout;

pub struct TunnelSetup<T: TunnelSpecifcSetup> {
    pub specific_setup: T,
}

pub trait TunnelSpecifcSetup {}

pub struct MixTunnelSetup {
    pub route_manager: RouteManager,
    pub mixnet_connection_info: MixnetConnectionInfo,
    pub exit_connection_info: MixnetExitConnectionInfo,
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
        _mixnet_client: SharedMixnetClient,
        entry: TunnelSetup<WgTunnelSetup>,
        exit: TunnelSetup<WgTunnelSetup>,
        task_manager: TaskManager,
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

async fn wait_interface_up(
    mut event_rx: mpsc::UnboundedReceiver<(TunnelEvent, oneshot::Sender<()>)>,
) -> Result<TunnelMetadata> {
    loop {
        match event_rx.next().await {
            Some((TunnelEvent::InterfaceUp(_, _), _)) => {
                continue;
            }
            Some((TunnelEvent::Up(metadata), _)) => {
                break Ok(metadata);
            }
            Some((TunnelEvent::AuthFailed(_), _)) | Some((TunnelEvent::Down, _)) | None => {
                return Err(Error::BadWireguardEvent);
            }
        }
    }
}

async fn setup_wg_tunnel(
    nym_vpn: &mut NymVpn<WireguardVpn>,
    mixnet_client: SharedMixnetClient,
    task_manager: TaskManager,
    route_manager: RouteManager,
    gateway_directory_client: GatewayClient,
    auth_addresses: AuthAddresses,
) -> Result<AllTunnelsSetup> {
    let mut rng = OsRng;
    let wg_entry_gateway_client = WgGatewayClient::new(&mut rng, mixnet_client.clone());
    let wg_exit_gateway_client = WgGatewayClient::new(&mut rng, mixnet_client.clone());
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

    let (Some(entry_auth_recipient), Some(exit_auth_recipient)) =
        (auth_addresses.entry().0, auth_addresses.exit().0)
    else {
        return Err(Error::AuthenticationNotPossible(auth_addresses.to_string()));
    };

    let mut entry_wireguard_config = init_wireguard_config(
        &gateway_directory_client,
        &wg_entry_gateway_client,
        entry_auth_recipient,
        entry_mtu,
    )
    .await?;
    let mut exit_wireguard_config = init_wireguard_config(
        &gateway_directory_client,
        &wg_exit_gateway_client,
        exit_auth_recipient,
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
    // If routing is disabled, we don't append the catch all routing rules
    if !nym_vpn.disable_routing {
        exit_wireguard_config.0.peers.iter_mut().for_each(|peer| {
            peer.allowed_ips
                .append(&mut replace_default_prefixes(catch_all_ipv4()));
            peer.allowed_ips
                .append(&mut replace_default_prefixes(catch_all_ipv6()));
        });
    } else {
        info!("Routing is disabled, skipping adding routes");
    }
    info!("Entry wireguard config: \n{entry_wireguard_config}");
    info!("Exit wireguard config: \n{exit_wireguard_config}");
    let (firewall, dns_monitor) = init_firewall_dns(
        #[cfg(target_os = "linux")]
        route_manager.handle()?,
    )
    .await?;
    std::env::set_var("TALPID_FORCE_USERSPACE_WIREGUARD", "1");
    let (wireguard_waiting_entry, event_rx) = create_wireguard_tunnel(
        &route_manager,
        nym_vpn.tun_provider.clone(),
        entry_wireguard_config,
    )
    .await?;
    // Wait for entry gateway routes to be finished before moving to exit gateway routes, as the two might race if
    // started one after the other
    let metadata = wait_interface_up(event_rx).await?;
    info!(
        "Created entry tun device {device_name} with ip={device_ip:?}",
        device_name = metadata.interface,
        device_ip = metadata.ips
    );
    let (wireguard_waiting_exit, event_rx) = create_wireguard_tunnel(
        &route_manager,
        nym_vpn.tun_provider.clone(),
        exit_wireguard_config,
    )
    .await?;
    let metadata = wait_interface_up(event_rx).await?;
    info!(
        "Created exit tun device {device_name} with ip={device_ip:?}",
        device_name = metadata.interface,
        device_ip = metadata.ips
    );
    let entry = TunnelSetup {
        specific_setup: wireguard_waiting_entry,
    };
    let exit = TunnelSetup {
        specific_setup: wireguard_waiting_exit,
    };

    Ok(AllTunnelsSetup::Wg {
        route_manager,
        _mixnet_client: mixnet_client,
        entry,
        exit,
        task_manager,
        firewall,
        dns_monitor,
    })
}

async fn setup_mix_tunnel(
    nym_vpn: &mut NymVpn<MixnetVpn>,
    mixnet_client: SharedMixnetClient,
    mut task_manager: TaskManager,
    mut route_manager: RouteManager,
    gateway_directory_client: GatewayClient,
    exit_mix_addresses: &IpPacketRouterAddress,
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
            mixnet_client,
            &mut route_manager,
            &exit_mix_addresses,
            &task_manager,
            &gateway_directory_client,
            default_lan_gateway_ip,
            &mut dns_monitor,
        )
        .await;
    let connection_info = match ret {
        Ok(connection_info) => connection_info,
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
            mixnet_connection_info: connection_info.0,
            exit_connection_info: connection_info.1,
            task_manager,
            dns_monitor,
        },
    }))
}

pub async fn setup_tunnel(nym_vpn: &mut SpecificVpn) -> Result<AllTunnelsSetup> {
    // The user agent is set on HTTP REST API calls, and ideally should idenfy the type of client.
    // This means it needs to be set way higher in the call stack, but set a default for what we
    // know here if we don't have anything.
    let user_agent = nym_vpn.user_agent().unwrap_or_else(|| {
        warn!("No user agent provided, using default");
        bin_info!().into()
    });
    info!("User agent: {user_agent}");

    // Create a gateway client that we use to interact with the entry gateway, in particular to
    // handle wireguard registration
    let gateway_directory_client = GatewayClient::new(nym_vpn.gateway_config(), user_agent)
        .map_err(|err| Error::FailedtoSetupGatewayDirectoryClient {
            config: Box::new(nym_vpn.gateway_config()),
            source: err,
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
    let entry_authenticator_address =
        extract_authenticator(&entry_gateways, entry_gateway_id.to_string())?;

    let (exit_gateway_id, exit_location) = nym_vpn
        .exit_point()
        .lookup_gateway_identity(&exit_gateways)
        .await
        .map_err(|err| Error::FailedToLookupGatewayIdentity { source: err })?;
    let exit_authenticator_address =
        extract_authenticator(&exit_gateways, exit_gateway_id.to_string())?;

    let exit_router_address = extract_router_address(&exit_gateways, exit_gateway_id.to_string())?;
    let exit_location_str = exit_location.as_deref().unwrap_or("unknown");
    let exit_gateway_id = exit_router_address.gateway();
    let auth_addresses =
        AuthAddresses::new(entry_authenticator_address, exit_authenticator_address);

    info!("Using entry gateway: {entry_gateway_id}, location: {entry_location_str}");
    info!("Using exit gateway: {exit_gateway_id}, location: {exit_location_str}");
    info!("Using exit router address {exit_router_address}");

    // Get the IP address of the local LAN gateway
    let default_lan_gateway_ip = routing::LanGatewayIp::get_default_interface()?;
    debug!("default_lan_gateway_ip: {default_lan_gateway_ip}");

    let task_manager = TaskManager::new(10).named("nym_vpn_lib");
    info!("Setting up route manager");
    let route_manager = setup_route_manager().await?;

    info!("Setting up mixnet client");
    info!("Connecting to entry gateway: {entry_gateway_id}");
    let mixnet_client = timeout(
        Duration::from_secs(MIXNET_CLIENT_STARTUP_TIMEOUT_SECS),
        crate::setup_mixnet_client(
            &entry_gateway_id,
            &nym_vpn.mixnet_client_config().mixnet_data_path,
            task_manager.subscribe_named("mixnet_client_main"),
            false,
            nym_vpn.enable_two_hop(),
            nym_vpn.mixnet_client_config().enable_poisson_rate,
            nym_vpn
                .mixnet_client_config()
                .disable_background_cover_traffic,
            nym_vpn.mixnet_client_config().enable_credentials_mode,
        ),
    )
    .await
    .map_err(|_| Error::StartMixnetTimeout(MIXNET_CLIENT_STARTUP_TIMEOUT_SECS))??;

    match nym_vpn {
        SpecificVpn::Wg(vpn) => {
            setup_wg_tunnel(
                vpn,
                mixnet_client,
                task_manager,
                route_manager,
                gateway_directory_client,
                auth_addresses,
            )
            .await
        }
        SpecificVpn::Mix(vpn) => {
            setup_mix_tunnel(
                vpn,
                mixnet_client,
                task_manager,
                route_manager,
                gateway_directory_client,
                &exit_router_address,
                default_lan_gateway_ip,
            )
            .await
        }
    }
}
