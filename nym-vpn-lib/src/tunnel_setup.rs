// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::error::{Error, Result};
use crate::tunnel::Tunnel;
use crate::util::{handle_interrupt, wait_for_interrupt};
use crate::wg_gateway_client::WgGatewayClient;
use crate::wireguard_setup::{create_wireguard_tunnel, empty_wireguard_setup};
use crate::{routing, MixnetConnectionInfo, NymVpn};
use futures::channel::oneshot;
use log::*;
use log::{debug, error, info};
use nym_gateway_directory::{GatewayClient, LookupGateway};
use nym_task::TaskManager;
use talpid_routing::RouteManager;
use tap::TapFallible;

pub struct TunnelSetup {
    pub tunnel: Tunnel,
    pub task_manager: TaskManager,
    pub route_manager: RouteManager,
    pub wireguard_waiting: Option<(oneshot::Receiver<()>, tokio::task::JoinHandle<Result<()>>)>,
    pub tunnel_close_tx: oneshot::Sender<()>,
    pub mixnet_connection_info: Option<MixnetConnectionInfo>,
}

pub async fn setup_tunnel(nym_vpn: &mut NymVpn) -> Result<Vec<TunnelSetup>> {
    // Create a gateway client that we use to interact with the entry gateway, in particular to
    // handle wireguard registration
    let gateway_client = GatewayClient::new(nym_vpn.gateway_config.clone())?;
    let gateways = gateway_client
        .lookup_described_gateways_with_location()
        .await?;
    log::debug!("Got gateways {:?}", gateways);

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

    let entry_gateway_id = nym_vpn
        .entry_point
        .lookup_gateway_identity(&gateways)
        .await?;
    info!("Using entry gateway: {entry_gateway_id}");
    let exit_gateway_id = nym_vpn
        .exit_point
        .lookup_gateway_identity(&gateways)
        .await?;
    info!("Using exit gateway: {exit_gateway_id}");

    // Get the IP address of the local LAN gateway
    let default_lan_gateway_ip = routing::LanGatewayIp::get_default_interface()?;
    debug!("default_lan_gateway_ip: {default_lan_gateway_ip}");

    let task_manager = TaskManager::new(10);

    if nym_vpn.tunnel_in_tunnel {
        let (wireguard_setup, tunnel) = create_wireguard_tunnel(
            nym_vpn
                .private_key
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
        let entry_tunnel_setup = TunnelSetup {
            tunnel,
            task_manager,
            route_manager: wireguard_setup.route_manager,
            wireguard_waiting: wireguard_setup.wireguard_waiting,
            tunnel_close_tx: wireguard_setup.tunnel_close_tx,
            mixnet_connection_info: None,
        };
        let (wireguard_setup, tunnel) = create_wireguard_tunnel(
            nym_vpn
                .private_key
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
        let exit_tunnel_setup = TunnelSetup {
            tunnel,
            task_manager: TaskManager::new(10),
            route_manager: wireguard_setup.route_manager,
            wireguard_waiting: wireguard_setup.wireguard_waiting,
            tunnel_close_tx: wireguard_setup.tunnel_close_tx,
            mixnet_connection_info: None,
        };

        Ok(vec![entry_tunnel_setup, exit_tunnel_setup])
    } else {
        let (mut wireguard_setup, mut tunnel) = if nym_vpn.enable_wireguard {
            create_wireguard_tunnel(
                nym_vpn
                    .private_key
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
            .await?
        } else {
            info!("Wireguard is disabled");
            let (wireguard_setup, _) = empty_wireguard_setup().await?;
            let tunnel = Tunnel::new(
                None,
                wireguard_setup.route_manager.handle()?,
                nym_vpn.tun_provider.clone(),
            )?;
            (wireguard_setup, tunnel)
        };

        // Now it's time start all the stuff that needs running inside the tunnel, and that we need
        // correctly unwind if it fails
        // - Sets up mixnet client, and connects
        // - Sets up routing
        // - Starts processing packets
        let exit_router_address = nym_vpn.exit_point.lookup_router_address(&gateways)?;
        info!("Using exit router address {exit_router_address}");

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
            Ok(mixnet_connection_info) => Some(mixnet_connection_info),
            Err(err) => {
                error!("Failed to setup tunnel services: {err}");
                debug!("{err:?}");
                wait_for_interrupt(task_manager).await;
                // Ignore if these fail since we're interesting in the original error anyway
                handle_interrupt(
                    wireguard_setup.route_manager,
                    wireguard_setup.wireguard_waiting,
                    wireguard_setup.tunnel_close_tx,
                )
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

        Ok(vec![TunnelSetup {
            tunnel,
            task_manager,
            route_manager: wireguard_setup.route_manager,
            wireguard_waiting: wireguard_setup.wireguard_waiting,
            tunnel_close_tx: wireguard_setup.tunnel_close_tx,
            mixnet_connection_info,
        }])
    }
}
