// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::IpAddr;
use std::time::Duration;

use crate::bandwidth_controller::BandwidthController;
use crate::error::{Error, GatewayDirectoryError, Result};
use crate::mixnet_connect::SharedMixnetClient;
use crate::platform;
use crate::routing::{catch_all_ipv4, catch_all_ipv6, replace_default_prefixes};
use crate::uniffi_custom_impls::TunStatus;
use crate::wg_gateway_client::WgGatewayClient;
use crate::wireguard_setup::create_wireguard_tunnel;
use crate::{init_wireguard_config, WireguardVpn, MIXNET_CLIENT_STARTUP_TIMEOUT_SECS};
use crate::{routing, MixnetConnectionInfo, NymVpn};
use crate::{MixnetExitConnectionInfo, MixnetVpn, SpecificVpn};
use futures::channel::{mpsc, oneshot};
use futures::StreamExt;
use ipnetwork::IpNetwork;
use log::*;
use nym_authenticator_client::AuthClient;
use nym_bin_common::bin_info;
use nym_gateway_directory::{AuthAddresses, GatewayClient, IpPacketRouterAddress};
use nym_task::TaskManager;
use talpid_core::dns::DnsMonitor;
use talpid_core::firewall::Firewall;
use talpid_routing::{Node, RequiredRoute, RouteManager};
use talpid_tunnel::{TunnelEvent, TunnelMetadata};
use tokio::time::timeout;

pub struct TunnelSetup<T: TunnelSpecifcSetup> {
    pub specific_setup: T,
}

pub trait TunnelSpecifcSetup {}

pub struct MixTunnelSetup {
    pub mixnet_connection_info: MixnetConnectionInfo,
    pub exit_connection_info: MixnetExitConnectionInfo,
}

impl TunnelSpecifcSetup for MixTunnelSetup {}

pub struct WgTunnelSetup {
    pub receiver: oneshot::Receiver<()>,
    pub handle: tokio::task::JoinHandle<()>,
    pub tunnel_close_tx: oneshot::Sender<()>,
}

impl TunnelSpecifcSetup for WgTunnelSetup {}

#[allow(clippy::large_enum_variant)]
pub enum AllTunnelsSetup {
    Mix(TunnelSetup<MixTunnelSetup>),
    Wg {
        entry: TunnelSetup<WgTunnelSetup>,
        exit: TunnelSetup<WgTunnelSetup>,
    },
}

pub(crate) async fn init_firewall_dns(
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
                debug!("Received interface up event");
                continue;
            }
            Some((TunnelEvent::Up(metadata), _)) => {
                debug!("Received up event");
                break Ok(metadata);
            }
            Some((TunnelEvent::AuthFailed(_), _)) | Some((TunnelEvent::Down, _)) | None => {
                debug!("Received unexpected event");
                return Err(Error::BadWireguardEvent);
            }
        }
    }
}

async fn setup_wg_tunnel(
    nym_vpn: &mut NymVpn<WireguardVpn>,
    mixnet_client: SharedMixnetClient,
    task_manager: &mut TaskManager,
    route_manager: &mut RouteManager,
    gateway_directory_client: GatewayClient,
    auth_addresses: AuthAddresses,
    default_lan_gateway_ip: routing::LanGatewayIp,
) -> Result<AllTunnelsSetup> {
    // MTU is computed as (MTU of wire interface) - ((IP header size) + (UDP header size) + (WireGuard metadata size))
    // The IP header size is 20 for IPv4 and 40 for IPv6
    // The UDP header size is 8
    // The Wireguard metadata size is 32
    // Entry tunnel will only deal with IPv4 => 1500 - (20 + 8 + 32)
    let entry_mtu = 1440;
    // Exit tunnel will deal with both v4 and v6, and it's "wire" interface is entry tunnel's MTU
    // 1440 - (40 + 8 + 32)
    let exit_mtu = 1360;

    let bandwidth_controller =
        BandwidthController::new(mixnet_client.clone(), task_manager.subscribe());
    tokio::spawn(bandwidth_controller.run());

    let (Some(entry_auth_recipient), Some(exit_auth_recipient)) =
        (auth_addresses.entry().0, auth_addresses.exit().0)
    else {
        return Err(Error::AuthenticationNotPossible(auth_addresses.to_string()));
    };
    let auth_client = AuthClient::new_from_inner(mixnet_client.inner()).await;
    log::info!("Created wg gateway clients");
    let mut wg_entry_gateway_client = WgGatewayClient::new_entry(
        &nym_vpn.data_path,
        auth_client.clone(),
        entry_auth_recipient,
    );
    let mut wg_exit_gateway_client =
        WgGatewayClient::new_exit(&nym_vpn.data_path, auth_client.clone(), exit_auth_recipient);

    let (mut exit_wireguard_config, _) = init_wireguard_config(
        &gateway_directory_client,
        &mut wg_exit_gateway_client,
        None,
        exit_mtu,
    )
    .await?;
    let wg_gateway = exit_wireguard_config
        .0
        .peers
        .first()
        .map(|config| config.endpoint.ip());
    let (mut entry_wireguard_config, entry_gateway_ip) = init_wireguard_config(
        &gateway_directory_client,
        &mut wg_entry_gateway_client,
        wg_gateway,
        entry_mtu,
    )
    .await?;

    if wg_entry_gateway_client.suspended().await? || wg_exit_gateway_client.suspended().await? {
        return Err(Error::NotEnoughBandwidth);
    }
    tokio::spawn(
        wg_entry_gateway_client.run(task_manager.subscribe_named("bandwidth_entry_client")),
    );
    tokio::spawn(wg_exit_gateway_client.run(task_manager.subscribe_named("bandwidth_exit_client")));
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

    let default_node = if let Some(addr) = default_lan_gateway_ip.0.gateway.and_then(|g| {
        g.ipv4
            .first()
            .map(|a| IpAddr::from(*a))
            .or(g.ipv6.first().map(|a| IpAddr::from(*a)))
    }) {
        Node::new(addr, default_lan_gateway_ip.0.name)
    } else {
        Node::device(default_lan_gateway_ip.0.name)
    };
    let routes = replace_default_prefixes(entry_gateway_ip.into())
        .into_iter()
        .map(move |ip| RequiredRoute::new(ip, default_node.clone()));
    #[cfg(target_os = "linux")]
    let routes = routes.map(|route| route.use_main_table(false));
    route_manager.add_routes(routes.collect()).await?;

    std::env::set_var("TALPID_FORCE_USERSPACE_WIREGUARD", "1");
    let (wireguard_waiting_entry, event_rx) = create_wireguard_tunnel(
        route_manager,
        task_manager.subscribe_named("entry_wg_tunnel"),
        nym_vpn.tun_provider.clone(),
        entry_wireguard_config,
    )
    .await?;

    // Wait for entry gateway routes to be finished before moving to exit gateway routes, as the two might race if
    // started one after the other
    debug!("Waiting for first interface up");
    let metadata = wait_interface_up(event_rx).await?;
    info!(
        "Created entry tun device {device_name} with ip={device_ip:?}",
        device_name = metadata.interface,
        device_ip = metadata.ips
    );

    let (wireguard_waiting_exit, event_rx) = create_wireguard_tunnel(
        route_manager,
        task_manager.subscribe_named("exit_wg_tunnel"),
        nym_vpn.tun_provider.clone(),
        exit_wireguard_config,
    )
    .await?;
    debug!("Waiting for second interface up");
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

    Ok(AllTunnelsSetup::Wg { entry, exit })
}

#[allow(clippy::too_many_arguments)]
async fn setup_mix_tunnel(
    nym_vpn: &mut NymVpn<MixnetVpn>,
    mixnet_client: SharedMixnetClient,
    task_manager: &mut TaskManager,
    route_manager: &mut RouteManager,
    dns_monitor: &mut DnsMonitor,
    gateway_directory_client: GatewayClient,
    exit_mix_addresses: &IpPacketRouterAddress,
    default_lan_gateway_ip: routing::LanGatewayIp,
) -> Result<AllTunnelsSetup> {
    info!("Wireguard is disabled");

    let connection_info = nym_vpn
        .setup_tunnel_services(
            mixnet_client,
            route_manager,
            exit_mix_addresses,
            task_manager,
            &gateway_directory_client,
            default_lan_gateway_ip,
            dns_monitor,
        )
        .await?;

    Ok(AllTunnelsSetup::Mix(TunnelSetup {
        specific_setup: MixTunnelSetup {
            mixnet_connection_info: connection_info.0,
            exit_connection_info: connection_info.1,
        },
    }))
}

pub async fn setup_tunnel(
    nym_vpn: &mut SpecificVpn,
    task_manager: &mut TaskManager,
    route_manager: &mut RouteManager,
    dns_monitor: &mut DnsMonitor,
) -> Result<AllTunnelsSetup> {
    // The user agent is set on HTTP REST API calls, and ideally should identify the type of
    // client. This means it needs to be set way higher in the call stack, but set a default for
    // what we know here if we don't have anything.
    let user_agent = nym_vpn.user_agent().unwrap_or_else(|| {
        warn!("No user agent provided, using default");
        bin_info!().into()
    });
    info!("User agent: {user_agent}");

    // Create a gateway client that we use to interact with the entry gateway, in particular to
    // handle wireguard registration
    let gateway_directory_client = GatewayClient::new(nym_vpn.gateway_config(), user_agent.clone())
        .map_err(
            |err| GatewayDirectoryError::FailedtoSetupGatewayDirectoryClient {
                config: Box::new(nym_vpn.gateway_config()),
                source: err,
            },
        )?;

    let SelectedGateways { entry, exit } =
        select_gateways(&gateway_directory_client, nym_vpn).await?;

    // Get the IP address of the local LAN gateway
    let default_lan_gateway_ip = routing::LanGatewayIp::get_default_interface()?;
    debug!("default_lan_gateway_ip: {default_lan_gateway_ip}");

    platform::set_listener_status(TunStatus::EstablishingConnection);

    info!("Setting up mixnet client");
    info!("Connecting to mixnet gateway: {}", entry.identity());
    let mixnet_client = timeout(
        Duration::from_secs(MIXNET_CLIENT_STARTUP_TIMEOUT_SECS),
        crate::setup_mixnet_client(
            entry.identity(),
            &nym_vpn.data_path(),
            task_manager.subscribe_named("mixnet_client_main"),
            false,
            nym_vpn.enable_two_hop(),
            nym_vpn.mixnet_client_config(),
        ),
    )
    .await
    .map_err(|_| Error::StartMixnetTimeout(MIXNET_CLIENT_STARTUP_TIMEOUT_SECS))??;

    let tunnels_setup = match nym_vpn {
        SpecificVpn::Wg(vpn) => {
            let entry_authenticator_address = entry
                .authenticator_address
                .ok_or(Error::AuthenticatorAddressNotFound)?;
            let exit_authenticator_address = exit
                .authenticator_address
                .ok_or(Error::AuthenticatorAddressNotFound)?;
            let auth_addresses =
                AuthAddresses::new(entry_authenticator_address, exit_authenticator_address);
            setup_wg_tunnel(
                vpn,
                mixnet_client,
                task_manager,
                route_manager,
                gateway_directory_client,
                auth_addresses,
                default_lan_gateway_ip,
            )
            .await
        }
        SpecificVpn::Mix(vpn) => {
            setup_mix_tunnel(
                vpn,
                mixnet_client,
                task_manager,
                route_manager,
                dns_monitor,
                gateway_directory_client,
                &exit.ipr_address.unwrap(),
                default_lan_gateway_ip,
            )
            .await
        }
    }?;
    Ok(tunnels_setup)
}

struct SelectedGateways {
    entry: nym_gateway_directory::Gateway,
    exit: nym_gateway_directory::Gateway,
}

async fn select_gateways(
    gateway_directory_client: &GatewayClient,
    nym_vpn: &SpecificVpn,
) -> std::result::Result<SelectedGateways, GatewayDirectoryError> {
    // The set of exit gateways is smaller than the set of entry gateways, so we start by selecting
    // the exit gateway and then filter out the exit gateway from the set of entry gateways.

    // Setup the gateway that we will use as the exit point
    let exit_gateways = gateway_directory_client
        .lookup_exit_gateways()
        .await
        .map_err(|source| GatewayDirectoryError::FailedToLookupGateways { source })?;

    let exit_gateway = nym_vpn
        .exit_point()
        .lookup_gateway(&exit_gateways)
        .map_err(|source| GatewayDirectoryError::FailedToSelectExitGateway { source })?;

    // Setup the gateway that we will use as the entry point
    let mut entry_gateways = gateway_directory_client
        .lookup_entry_gateways()
        .await
        .map_err(|source| GatewayDirectoryError::FailedToLookupGateways { source })?;

    // Exclude the exit gateway from the list of entry gateways for privacy reasons
    entry_gateways.remove_gateway(&exit_gateway);

    let entry_gateway = nym_vpn
        .entry_point()
        .lookup_gateway(&entry_gateways)
        .map_err(|source| match source {
            nym_gateway_directory::Error::NoMatchingEntryGatewayForLocation {
                requested_location,
                available_countries: _,
            } if Some(requested_location.as_str())
                == exit_gateway.two_letter_iso_country_code() =>
            {
                GatewayDirectoryError::SameEntryAndExitGatewayFromCountry {
                    requested_location: requested_location.to_string(),
                }
            }
            _ => GatewayDirectoryError::FailedToSelectEntryGateway { source },
        })?;

    info!("Found {} entry gateways", entry_gateways.len());
    info!("Found {} exit gateways", exit_gateways.len());
    info!(
        "Using entry gateway: {}, location: {}",
        *entry_gateway.identity(),
        entry_gateway
            .two_letter_iso_country_code()
            .map_or_else(|| "unknown".to_string(), |code| code.to_string())
    );
    info!(
        "Using exit gateway: {}, location: {}",
        *exit_gateway.identity(),
        exit_gateway
            .two_letter_iso_country_code()
            .map_or_else(|| "unknown".to_string(), |code| code.to_string())
    );
    info!(
        "Using exit router address {}",
        exit_gateway.ipr_address.unwrap()
    );

    Ok(SelectedGateways {
        entry: entry_gateway,
        exit: exit_gateway,
    })
}
