// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::IpAddr;

use nym_gateway_directory::{GatewayClient, IpPacketRouterAddress, Recipient};
use nym_ip_packet_client::IprClientConnect;
use nym_ip_packet_requests::IpPair;
use nym_mixnet_client::SharedMixnetClient;
use nym_sdk::mixnet::ConnectionStatsEvent;
use nym_task::TaskManager;
use tokio_util::sync::CancellationToken;

use super::connected_tunnel::ConnectedTunnel;
use crate::tunnel_state_machine::tunnel::{
    self, gateway_selector::SelectedGateways, AnyConnector, ConnectorError, Error, Result,
};

/// Struct holding addresses assigned by mixnet upon connect.
pub struct AssignedAddresses {
    pub entry_mixnet_gateway_ip: IpAddr,
    pub exit_mixnet_gateway_ip: IpAddr,
    pub mixnet_client_address: Recipient,
    pub exit_mix_addresses: IpPacketRouterAddress,
    pub interface_addresses: IpPair,
}

/// Type responsible for connecting the mixnet tunnel.
pub struct Connector {
    task_manager: TaskManager,
    mixnet_client: SharedMixnetClient,
    gateway_directory_client: GatewayClient,
}

impl Connector {
    pub fn new(
        task_manager: TaskManager,
        mixnet_client: SharedMixnetClient,
        gateway_directory_client: GatewayClient,
    ) -> Self {
        Self {
            task_manager,
            mixnet_client,
            gateway_directory_client,
        }
    }

    pub async fn connect(
        self,
        selected_gateways: SelectedGateways,
        nym_ips: Option<IpPair>,
        cancel_token: CancellationToken,
    ) -> Result<ConnectedTunnel, ConnectorError> {
        let result = Self::connect_inner(
            selected_gateways,
            nym_ips,
            self.mixnet_client.clone(),
            &self.gateway_directory_client,
            cancel_token,
        )
        .await;

        match result {
            Ok(assigned_addresses) => Ok(ConnectedTunnel::new(
                self.task_manager,
                self.mixnet_client,
                assigned_addresses,
            )),
            Err(e) => Err(ConnectorError::new(
                e,
                AnyConnector::Mixnet(Self::new(
                    self.task_manager,
                    self.mixnet_client,
                    self.gateway_directory_client,
                )),
            )),
        }
    }

    async fn connect_inner(
        selected_gateways: SelectedGateways,
        nym_ips: Option<IpPair>,
        mixnet_client: SharedMixnetClient,
        gateway_directory_client: &GatewayClient,
        cancel_token: CancellationToken,
    ) -> Result<AssignedAddresses> {
        let mixnet_client_address = mixnet_client.nym_address().await;
        let gateway_used = mixnet_client_address.gateway().to_base58_string();
        let entry_mixnet_gateway_ip: IpAddr = cancel_token
            .run_until_cancelled(gateway_directory_client.lookup_gateway_ip(&gateway_used))
            .await
            .ok_or(tunnel::Error::Cancelled)?
            .map_err(|source| Error::LookupGatewayIp {
                gateway_id: gateway_used,
                source,
            })?;

        let exit_mix_addresses = selected_gateways
            .exit
            .ipr_address
            .expect("failed to unwrap ipr_address");
        let gateway_used = exit_mix_addresses.gateway().to_base58_string();
        let exit_mixnet_gateway_ip = cancel_token
            .run_until_cancelled(gateway_directory_client.lookup_gateway_ip(&gateway_used))
            .await
            .ok_or(tunnel::Error::Cancelled)?
            .map_err(|source| Error::LookupGatewayIp {
                gateway_id: gateway_used,
                source,
            })?;

        let mut ipr_client = IprClientConnect::new(mixnet_client.clone(), cancel_token).await;
        let interface_addresses = ipr_client
            .connect(exit_mix_addresses.0, nym_ips)
            .await
            .map_err(Error::ConnectToIpPacketRouter)?;

        if let Some(exit_country_code) = selected_gateways.exit.two_letter_iso_country_code() {
            mixnet_client
                .send_stats_event(
                    ConnectionStatsEvent::MixCountry(exit_country_code.to_string()).into(),
                )
                .await;
        }

        Ok(AssignedAddresses {
            entry_mixnet_gateway_ip,
            exit_mixnet_gateway_ip,
            mixnet_client_address,
            exit_mix_addresses,
            interface_addresses,
        })
    }

    /// Gracefully shutdown task manager and mixnet client, and consume the struct.
    pub async fn dispose(self) {
        tracing::debug!("Shutting down mixnet client");
        tunnel::shutdown_mixnet_client(self.task_manager, self.mixnet_client).await;
    }
}
