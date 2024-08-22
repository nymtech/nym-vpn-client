// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    net::IpAddr,
    sync::{Arc, Mutex},
};

use log::{debug, error, info};
use nym_connection_monitor::ConnectionMonitorTask;
use nym_gateway_directory::{
    EntryPoint, ExitPoint, GatewayClient, IpPacketRouterAddress, NodeIdentity, Recipient,
};
use nym_ip_packet_client::IprClientConnect;
use nym_ip_packet_requests::IpPair;
use nym_task::TaskManager;
use talpid_core::dns::DnsMonitor;
use talpid_routing::RouteManager;
use talpid_tunnel::tun_provider::TunProvider;

#[cfg(target_os = "ios")]
use crate::platform::swift::OSTunProvider;
use crate::{error::Result, mixnet::SharedMixnetClient, routing, Error, GatewayDirectoryError};

use super::base::{GenericNymVpnConfig, NymVpn, ShadowHandle, Vpn};

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

#[derive(Debug, Clone, Copy)]
pub struct MixnetConnectionInfo {
    pub nym_address: Recipient,
    pub entry_gateway: NodeIdentity,
}

#[derive(Debug, Clone, Copy)]
pub struct MixnetExitConnectionInfo {
    pub exit_gateway: NodeIdentity,
    pub exit_ipr: Recipient,
    pub ips: IpPair,
}

pub struct MixnetVpn {}

impl Vpn for MixnetVpn {}

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
            generic_config: GenericNymVpnConfig {
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
                user_agent: None,
            },
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
        let mut ipr_client = IprClientConnect::new_from_inner(mixnet_client.inner()).await;
        let our_ips = ipr_client
            .connect(exit_mix_addresses.0, self.generic_config.nym_ips)
            .await
            .map_err(Error::FailedToConnectToIpPacketRouter)?;
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
            self.generic_config.dns,
        )
        .await?;

        info!("Setting up mixnet processor");
        let processor_config = crate::mixnet::Config::new(exit_mix_addresses.0);
        debug!("Mixnet processor config: {:#?}", processor_config);

        // For other components that will want to send mixnet packets
        let mixnet_client_sender = mixnet_client.split_sender().await;

        // Setup connection monitor shared tag and channels
        let connection_monitor = ConnectionMonitorTask::setup();

        let shadow_handle = crate::mixnet::start_processor(
            processor_config,
            mixnet_tun_dev,
            mixnet_client,
            task_manager,
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
    pub(crate) async fn setup_tunnel_services(
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
