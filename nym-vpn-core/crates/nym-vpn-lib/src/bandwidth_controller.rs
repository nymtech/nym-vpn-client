// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_bandwidth_controller::PreparedCredential;
use nym_credential_storage::storage::Storage;
use nym_credentials_interface::TicketType;
use nym_gateway_directory::GatewayClient;
use nym_sdk::TaskClient;
use nym_validator_client::nyxd::contract_traits::DkgQueryClient;
use nym_wg_gateway_client::{ErrorMessage, GatewayData, WgGatewayClient, WgGatewayLightClient};
use nym_wireguard_types::DEFAULT_PEER_TIMEOUT_CHECK;
use std::net::IpAddr;
use std::time::Duration;
use tokio_stream::{wrappers::IntervalStream, StreamExt};
use tracing::{trace, warn};

use crate::mixnet::SharedMixnetClient;
use crate::SetupWgTunnelError;

const DEFAULT_BANDWIDTH_CHECK: Duration = Duration::from_secs(10); // 10 seconds
const ASSUMED_BANDWIDTH_DEPLETION_RATE: u64 = 10 * 1024 * 1024; // 10 MB/s
const TICKETS_TO_SPEND: u32 = 1;

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
    shared_mixnet_client: SharedMixnetClient,
    wg_entry_gateway_client: WgGatewayLightClient,
    wg_exit_gateway_client: WgGatewayLightClient,
    shutdown: TaskClient,
}

impl<C, St: Storage> BandwidthController<C, St> {
    pub(crate) fn new(
        inner: nym_bandwidth_controller::BandwidthController<C, St>,
        shared_mixnet_client: SharedMixnetClient,
        wg_entry_gateway_client: WgGatewayLightClient,
        wg_exit_gateway_client: WgGatewayLightClient,
        shutdown: TaskClient,
    ) -> Self {
        BandwidthController {
            inner,
            shared_mixnet_client,
            wg_entry_gateway_client,
            wg_exit_gateway_client,
            shutdown,
        }
    }

    pub(crate) async fn get_initial_bandwidth(
        &self,
        gateway_client: &GatewayClient,
        wg_gateway_client: &mut WgGatewayClient,
    ) -> std::result::Result<(GatewayData, IpAddr), SetupWgTunnelError>
    where
        C: DkgQueryClient + Sync + Send,
        <St as Storage>::StorageError: Send + Sync + 'static,
    {
        let credential = self.request_bandwidth().await?;

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

        Ok((wg_gateway_data, gateway_host))
    }

    pub(crate) async fn request_bandwidth(
        &self,
    ) -> std::result::Result<PreparedCredential, SetupWgTunnelError>
    where
        C: DkgQueryClient + Sync + Send,
        <St as Storage>::StorageError: Send + Sync + 'static,
    {
        let credential = self
            .inner
            .prepare_ecash_ticket(TicketType::V1WireguardEntry, [0; 32], TICKETS_TO_SPEND)
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
            Err(e) => warn!("Error querying remaining bandwidth {:?}", e),
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
                    trace!("BandwidthController: Received shutdown");
                    self.shared_mixnet_client.clone().disconnect().await;
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
