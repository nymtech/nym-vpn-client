// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::IpAddr;

use nym_gateway_directory::{GatewayCacheHandle, IpPacketRouterAddress, Recipient};
use nym_ip_packet_client::IprClientConnect;
use nym_ip_packet_requests::IpPair;
use nym_sdk::mixnet::ConnectionStatsEvent;
use tokio_util::sync::CancellationToken;

use crate::{
    mixnet::SharedMixnetClient,
    tunnel_state_machine::tunnel::{
        self, Error, Result, gateway_selector::SelectedGateways,
        mixnet::connected_tunnel::ConnectedTunnel,
    },
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
    mixnet_client: SharedMixnetClient,
    gateway_cache_handle: GatewayCacheHandle,
}

impl Connector {
    pub fn new(
        mixnet_client: SharedMixnetClient,
        gateway_cache_handle: GatewayCacheHandle,
    ) -> Self {
        Self {
            mixnet_client,
            gateway_cache_handle,
        }
    }

    pub async fn connect(
        self,
        selected_gateways: SelectedGateways,
        cancel_token: CancellationToken,
    ) -> Result<ConnectedTunnel> {
        let assigned_addresses = Self::connect_inner(
            selected_gateways,
            self.mixnet_client.clone(),
            self.gateway_cache_handle.clone(),
            cancel_token.clone(),
        )
        .await?;

        Ok(ConnectedTunnel::new(
            self.mixnet_client,
            assigned_addresses,
            cancel_token,
        ))
    }

    async fn connect_inner(
        selected_gateways: SelectedGateways,
        mixnet_client: SharedMixnetClient,
        gateway_cache_handle: GatewayCacheHandle,
        cancel_token: CancellationToken,
    ) -> Result<AssignedAddresses> {
        let mixnet_client_address = *mixnet_client
            .lock()
            .await
            .as_ref()
            .ok_or(Error::MixnetClientDisposed)?
            .nym_address();
        let gateway_used = mixnet_client_address.gateway().to_base58_string();
        let entry_mixnet_gateway_ip: IpAddr = cancel_token
            .run_until_cancelled(gateway_cache_handle.lookup_gateway_ip(gateway_used.clone()))
            .await
            .ok_or(tunnel::Error::Cancelled)?
            .map_err(|source| Error::LookupGatewayIp {
                gateway_id: gateway_used.clone(),
                source: Box::new(source),
            })?;

        let exit_mix_addresses = selected_gateways
            .exit
            .ipr_address
            .expect("failed to unwrap ipr_address");
        let gateway_used = exit_mix_addresses.gateway().to_base58_string();
        let exit_mixnet_gateway_ip = cancel_token
            .run_until_cancelled(gateway_cache_handle.lookup_gateway_ip(gateway_used.clone()))
            .await
            .ok_or(tunnel::Error::Cancelled)?
            .map_err(|source| Error::LookupGatewayIp {
                gateway_id: gateway_used,
                source: Box::new(source),
            })?;

        let mut ipr_client = IprClientConnect::new(mixnet_client.clone(), cancel_token).await;
        let interface_addresses = ipr_client
            .connect(exit_mix_addresses)
            .await
            .map_err(Error::ConnectToIpPacketRouter)?;

        if let Some(exit_country_code) = selected_gateways.exit.two_letter_iso_country_code() {
            mixnet_client
                .lock()
                .await
                .as_ref()
                .ok_or(Error::MixnetClientDisposed)?
                .send_stats_event(
                    ConnectionStatsEvent::MixCountry(exit_country_code.to_string()).into(),
                );
        }

        Ok(AssignedAddresses {
            entry_mixnet_gateway_ip,
            exit_mixnet_gateway_ip,
            mixnet_client_address,
            exit_mix_addresses,
            interface_addresses,
        })
    }
}
