// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use nym_bandwidth_controller::PreparedCredential;
use nym_credentials_interface::TicketType;
use nym_gateway_directory::GatewayClient;
use nym_sdk::{mixnet::CredentialStorage as Storage, NymNetworkDetails, TaskClient};
use nym_validator_client::{
    nyxd::{contract_traits::DkgQueryClient, Config as NyxdClientConfig, NyxdClient},
    QueryHttpRpcNyxdClient,
};
use nym_wg_gateway_client::{ErrorMessage, GatewayData, WgGatewayClient, WgGatewayLightClient};
use nym_wireguard_types::DEFAULT_PEER_TIMEOUT_CHECK;
use tokio_stream::{wrappers::IntervalStream, StreamExt};

use crate::SetupWgTunnelError;

const DEFAULT_BANDWIDTH_CHECK: Duration = Duration::from_secs(10); // 10 seconds
const ASSUMED_BANDWIDTH_DEPLETION_RATE: u64 = 10 * 1024 * 1024; // 10 MB/s
const TICKETS_TO_SPEND: u32 = 1;

#[derive(Debug, thiserror::Error)]
pub enum CredentialNyxdClientError {
    #[error("failed to create nyxd client config: {0}")]
    FailedToCreateNyxdClientConfig(nym_validator_client::nyxd::error::NyxdError),

    #[error("no nyxd endpoints found")]
    NoNyxdEndpointsFound,

    #[error("failed to connect using nyxd client: {0}")]
    FailedToConnectUsingNyxdClient(nym_validator_client::nyxd::error::NyxdError),
}

pub(crate) fn get_nyxd_client(
) -> std::result::Result<QueryHttpRpcNyxdClient, CredentialNyxdClientError> {
    let network = NymNetworkDetails::new_from_env();
    let config = NyxdClientConfig::try_from_nym_network_details(&network)
        .map_err(CredentialNyxdClientError::FailedToCreateNyxdClientConfig)?;

    // Safe to use pick the first one?
    let nyxd_url = network
        .endpoints
        .first()
        .ok_or(CredentialNyxdClientError::NoNyxdEndpointsFound)?
        .nyxd_url();

    NyxdClient::connect(config, nyxd_url.as_str())
        .map_err(CredentialNyxdClientError::FailedToConnectUsingNyxdClient)
}

fn update_dynamic_check_interval(remaining_bandwidth: u64) -> Option<Duration> {
    let estimated_depletion_secs = remaining_bandwidth / ASSUMED_BANDWIDTH_DEPLETION_RATE;
    // try and have 10 logs before depletion...
    let next_timeout_secs = estimated_depletion_secs / 10;
    if next_timeout_secs == 0 {
        return None;
    }
    // ... but not faster then the gateway bandwidth refresh
    if next_timeout_secs > DEFAULT_PEER_TIMEOUT_CHECK.as_secs() {
        Some(Duration::from_secs(next_timeout_secs))
    } else {
        Some(DEFAULT_PEER_TIMEOUT_CHECK)
    }
}

pub(crate) struct BandwidthController<C, St> {
    inner: nym_bandwidth_controller::BandwidthController<C, St>,
    wg_entry_gateway_client: WgGatewayLightClient,
    wg_exit_gateway_client: WgGatewayLightClient,
    shutdown: TaskClient,
}

impl<C, St: Storage> BandwidthController<C, St> {
    pub(crate) fn new(
        inner: nym_bandwidth_controller::BandwidthController<C, St>,
        wg_entry_gateway_client: WgGatewayLightClient,
        wg_exit_gateway_client: WgGatewayLightClient,
        shutdown: TaskClient,
    ) -> Self {
        BandwidthController {
            inner,
            wg_entry_gateway_client,
            wg_exit_gateway_client,
            shutdown,
        }
    }

    pub(crate) async fn get_initial_bandwidth(
        &self,
        gateway_client: &GatewayClient,
        wg_gateway_client: &mut WgGatewayClient,
    ) -> std::result::Result<GatewayData, SetupWgTunnelError>
    where
        C: DkgQueryClient + Sync + Send,
        <St as Storage>::StorageError: Send + Sync + 'static,
    {
        let credential = self
            .request_bandwidth(wg_gateway_client.auth_recipient().gateway().to_bytes())
            .await?;

        // First we need to register with the gateway to setup keys and IP assignment
        tracing::info!("Registering with wireguard gateway");
        let authenticator_address = wg_gateway_client.auth_recipient();
        let gateway_id = *wg_gateway_client.auth_recipient().gateway();
        let gateway_host = gateway_client
            .lookup_gateway_ip(&gateway_id.to_base58_string())
            .await
            .map_err(|source| SetupWgTunnelError::FailedToLookupGatewayIp {
                gateway_id: Box::new(gateway_id),
                source,
            })?;
        let wg_gateway_data = wg_gateway_client
            .register_wireguard(gateway_host, Some(credential.data))
            .await
            .map_err(|source| SetupWgTunnelError::WgGatewayClientError {
                gateway_id: Box::new(gateway_id),
                authenticator_address: Box::new(authenticator_address),
                source,
            })?;
        tracing::debug!("Received wireguard gateway data: {wg_gateway_data:?}");

        Ok(wg_gateway_data)
    }

    pub(crate) async fn request_bandwidth(
        &self,
        provider_pk: [u8; 32],
    ) -> std::result::Result<PreparedCredential, SetupWgTunnelError>
    where
        C: DkgQueryClient + Sync + Send,
        <St as Storage>::StorageError: Send + Sync + 'static,
    {
        let credential = self
            .inner
            .prepare_ecash_ticket(TicketType::V1WireguardEntry, provider_pk, TICKETS_TO_SPEND)
            .await?;
        Ok(credential)
    }

    async fn check_bandwidth(&mut self, entry: bool) -> Option<Duration> {
        let wg_gateway_client = if entry {
            &mut self.wg_entry_gateway_client
        } else {
            &mut self.wg_exit_gateway_client
        };
        match wg_gateway_client.query_bandwidth().await {
            Err(e) => tracing::warn!("Error querying remaining bandwidth {:?}", e),
            Ok(Some(remaining_bandwidth)) => {
                match update_dynamic_check_interval(remaining_bandwidth) {
                    Some(new_duration) => {
                        return Some(new_duration);
                    }
                    None => {
                        // TODO: try to return this error in the JoinHandle instead
                        self.shutdown
                            .send_we_stopped(Box::new(ErrorMessage::OutOfBandwidth {
                                gateway_id: Box::new(*wg_gateway_client.auth_recipient().gateway()),
                                authenticator_address: Box::new(wg_gateway_client.auth_recipient()),
                            }));
                    }
                }
            }
            Ok(None) => {}
        }
        None
    }

    pub(crate) async fn run(mut self) {
        let mut timeout_check_interval =
            IntervalStream::new(tokio::time::interval(DEFAULT_BANDWIDTH_CHECK));
        // Skip the first, immediate tick
        timeout_check_interval.next().await;
        while !self.shutdown.is_shutdown() {
            tokio::select! {
                _ = self.shutdown.recv() => {
                    tracing::trace!("BandwidthController: Received shutdown");
                }
                _ = timeout_check_interval.next() => {
                    let entry_duration = self.check_bandwidth(true).await;
                    let exit_duration = self.check_bandwidth(false).await;
                    if let Some(minimal_duration) = match (entry_duration, exit_duration) {
                        (Some(d1), Some(d2)) => {
                            if d1 < d2 {
                                Some(d1)
                            } else {
                                Some(d2)
                            }
                        },
                        (Some(d), None) => Some(d),
                        (None, Some(d)) => Some(d),
                        _ => None,
                    } {
                        timeout_check_interval = IntervalStream::new(tokio::time::interval(minimal_duration));
                        // Skip the first, immediate tick
                        timeout_check_interval.next().await;
                    }
                }
            }
        }
    }
}
