// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use nym_wireguard_types::DEFAULT_PEER_TIMEOUT_CHECK;
use tokio_stream::{wrappers::IntervalStream, StreamExt};

use nym_bandwidth_controller::PreparedCredential;
use nym_credentials_interface::TicketType;
use nym_gateway_directory::GatewayClient;
use nym_sdk::{mixnet::CredentialStorage as Storage, NymNetworkDetails, TaskClient};
use nym_validator_client::{
    nyxd::{Config as NyxdClientConfig, NyxdClient},
    QueryHttpRpcNyxdClient,
};
use nym_wg_gateway_client::{ErrorMessage, GatewayData, WgGatewayClient, WgGatewayLightClient};

const DEFAULT_BANDWIDTH_CHECK: Duration = Duration::from_secs(5); // 5 seconds
const DEFAULT_BANDWIDTH_DEPLETION_RATE: u64 = 1024 * 1024; // 1 MB/s
const TICKETS_TO_SPEND: u32 = 1;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to lookup gateway ip: {source}")]
    LookupGatewayIp {
        gateway_id: String,
        #[source]
        source: nym_gateway_directory::Error,
    },

    #[error("failed to register wireguard with the gateway: {source}")]
    RegisterWireguard {
        gateway_id: String,
        authenticator_address: Box<nym_gateway_directory::Recipient>,
        #[source]
        source: nym_wg_gateway_client::Error,
    },

    #[error("failed to top-up wireguard bandwidth with the gateway: {source}")]
    TopUpWireguard {
        gateway_id: String,
        ticketbook_type: TicketType,
        authenticator_address: Box<nym_gateway_directory::Recipient>,
        #[source]
        source: nym_wg_gateway_client::Error,
    },

    #[error("failed to get {ticketbook_type} ticket: {source}")]
    GetTicket {
        ticketbook_type: TicketType,
        #[source]
        source: nym_bandwidth_controller::error::BandwidthControllerError,
    },

    #[error("nyxd client error: {0}")]
    Nyxd(#[from] CredentialNyxdClientError),

    #[error("internal error: {reason}")]
    Internal { reason: String },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum CredentialNyxdClientError {
    #[error("failed to create nyxd client config: {0}")]
    FailedToCreateNyxdClientConfig(nym_validator_client::nyxd::error::NyxdError),

    #[error("no nyxd endpoints found")]
    NoNyxdEndpointsFound,

    #[error("failed to connect using nyxd client: {0}")]
    FailedToConnectUsingNyxdClient(nym_validator_client::nyxd::error::NyxdError),
}

fn get_nyxd_client() -> Result<QueryHttpRpcNyxdClient> {
    let network = NymNetworkDetails::new_from_env();
    let config = NyxdClientConfig::try_from_nym_network_details(&network)
        .map_err(CredentialNyxdClientError::FailedToCreateNyxdClientConfig)?;

    // Safe to use pick the first one?
    let nyxd_url = network
        .endpoints
        .first()
        .ok_or(CredentialNyxdClientError::NoNyxdEndpointsFound)?
        .nyxd_url();

    Ok(NyxdClient::connect(config, nyxd_url.as_str())
        .map_err(CredentialNyxdClientError::FailedToConnectUsingNyxdClient)?)
}

pub(crate) struct DepletionRate {
    current_depletion_rate: u64,
    available_bandwidth: u64,
}

impl Default for DepletionRate {
    fn default() -> Self {
        Self {
            current_depletion_rate: DEFAULT_BANDWIDTH_DEPLETION_RATE,
            available_bandwidth: u64::MAX,
        }
    }
}

impl DepletionRate {
    fn update_dynamic_check_interval(
        &mut self,
        current_period: Duration,
        remaining_bandwidth: u64,
    ) -> Result<Option<Duration>> {
        let Some(new_depletion_rate) = remaining_bandwidth
            .saturating_sub(self.available_bandwidth)
            .checked_div(current_period.as_secs())
        else {
            return Err(Error::Internal {
                reason: "check interval shouldn't be 0".to_string(),
            });
        };
        self.available_bandwidth = remaining_bandwidth;
        // if nothing was consumed since last time, we prefer to stick to the old deplation rate
        if new_depletion_rate != 0 {
            self.current_depletion_rate = new_depletion_rate;
        }
        log::info!("Current deplateion rate {}", self.current_depletion_rate);
        log::info!(
            "Estimated depletion in {}/{}",
            remaining_bandwidth,
            self.current_depletion_rate
        );
        let Some(estimated_depletion_secs) =
            remaining_bandwidth.checked_div(self.current_depletion_rate)
        else {
            return Err(Error::Internal {
                reason: "depletion rate shouldn't be 0".to_string(),
            });
        };
        // try and have at least 10 logs before depletion..
        let next_timeout_secs = estimated_depletion_secs / 10;
        if next_timeout_secs == 0 {
            return Ok(None);
        }
        if next_timeout_secs > 6 * DEFAULT_PEER_TIMEOUT_CHECK.as_secs() {
            // ... but not too slow, in case bursts come in
            Ok(Some(Duration::from_secs(
                6 * DEFAULT_PEER_TIMEOUT_CHECK.as_secs(),
            )))
        } else if next_timeout_secs < DEFAULT_PEER_TIMEOUT_CHECK.as_secs() {
            // ... and not faster then the gateway bandwidth refresh, as that won't produce any change
            Ok(Some(DEFAULT_PEER_TIMEOUT_CHECK))
        } else {
            Ok(Some(Duration::from_secs(next_timeout_secs)))
        }
    }
}

pub(crate) struct BandwidthController<St> {
    inner: nym_bandwidth_controller::BandwidthController<QueryHttpRpcNyxdClient, St>,
    wg_entry_gateway_client: WgGatewayLightClient,
    wg_exit_gateway_client: WgGatewayLightClient,
    timeout_check_interval: IntervalStream,
    entry_depletion_rate: DepletionRate,
    exit_depletion_rate: DepletionRate,
    shutdown: TaskClient,
}

impl<St: Storage> BandwidthController<St> {
    pub(crate) fn new(
        storage: St,
        wg_entry_gateway_client: WgGatewayLightClient,
        wg_exit_gateway_client: WgGatewayLightClient,
        shutdown: TaskClient,
    ) -> Result<Self> {
        let client = get_nyxd_client()?;
        let inner = nym_bandwidth_controller::BandwidthController::new(storage, client);
        let timeout_check_interval =
            IntervalStream::new(tokio::time::interval(DEFAULT_BANDWIDTH_CHECK));

        Ok(BandwidthController {
            inner,
            wg_entry_gateway_client,
            wg_exit_gateway_client,
            timeout_check_interval,
            entry_depletion_rate: Default::default(),
            exit_depletion_rate: Default::default(),
            shutdown,
        })
    }

    pub(crate) async fn get_initial_bandwidth(
        &self,
        enable_credentials_mode: bool,
        ticketbook_type: TicketType,
        gateway_client: &GatewayClient,
        wg_gateway_client: &mut WgGatewayClient,
    ) -> Result<GatewayData>
    where
        <St as Storage>::StorageError: Send + Sync + 'static,
    {
        let credential = if enable_credentials_mode {
            let cred = self
                .request_bandwidth(
                    ticketbook_type,
                    wg_gateway_client.auth_recipient().gateway().to_bytes(),
                )
                .await?;
            Some(cred.data)
        } else {
            None
        };

        // First we need to register with the gateway to setup keys and IP assignment
        tracing::info!("Registering with wireguard gateway");
        let authenticator_address = wg_gateway_client.auth_recipient();
        let gateway_id = *wg_gateway_client.auth_recipient().gateway();
        let gateway_host = gateway_client
            // .lookup_gateway_ip(&gateway_id.to_base58_string())
            .lookup_gateway_ip_legacy(&gateway_id.to_base58_string())
            .await
            .map_err(|source| Error::LookupGatewayIp {
                gateway_id: gateway_id.to_base58_string(),
                source,
            })?;
        let wg_gateway_data = wg_gateway_client
            .register_wireguard(gateway_host, credential)
            .await
            .map_err(|source| Error::RegisterWireguard {
                gateway_id: gateway_id.to_base58_string(),
                authenticator_address: Box::new(authenticator_address),
                source,
            })?;
        tracing::debug!("Received wireguard gateway data: {wg_gateway_data:?}");

        Ok(wg_gateway_data)
    }

    pub(crate) async fn top_up_bandwidth(
        &self,
        ticketbook_type: TicketType,
        wg_gateway_client: &mut WgGatewayLightClient,
    ) -> Result<i64>
    where
        <St as Storage>::StorageError: Send + Sync + 'static,
    {
        let credential = self
            .request_bandwidth(
                ticketbook_type,
                wg_gateway_client.auth_recipient().gateway().to_bytes(),
            )
            .await?;
        let authenticator_address = wg_gateway_client.auth_recipient();
        let gateway_id = *wg_gateway_client.auth_recipient().gateway();
        let remaining_bandwidth =
            wg_gateway_client
                .top_up(credential.data)
                .await
                .map_err(|source| Error::TopUpWireguard {
                    gateway_id: gateway_id.to_string(),
                    ticketbook_type,
                    authenticator_address: Box::new(authenticator_address),
                    source,
                })?;

        Ok(remaining_bandwidth)
    }

    pub(crate) async fn request_bandwidth(
        &self,
        ticketbook_type: TicketType,
        provider_pk: [u8; 32],
    ) -> Result<PreparedCredential>
    where
        <St as Storage>::StorageError: Send + Sync + 'static,
    {
        let credential = self
            .inner
            .prepare_ecash_ticket(ticketbook_type, provider_pk, TICKETS_TO_SPEND)
            .await
            .map_err(|source| Error::GetTicket {
                ticketbook_type,
                source,
            })?;
        Ok(credential)
    }

    async fn check_bandwidth(&mut self, entry: bool, current_period: Duration) -> Option<Duration>
    where
        <St as Storage>::StorageError: Send + Sync + 'static,
    {
        let (mut wg_gateway_client, current_depletion_rate) = if entry {
            (
                self.wg_entry_gateway_client.clone(),
                &mut self.entry_depletion_rate,
            )
        } else {
            (
                self.wg_exit_gateway_client.clone(),
                &mut self.exit_depletion_rate,
            )
        };
        match wg_gateway_client.query_bandwidth().await {
            Err(e) => tracing::warn!("Error querying remaining bandwidth {:?}", e),
            Ok(Some(remaining_bandwidth)) => {
                match current_depletion_rate
                    .update_dynamic_check_interval(current_period, remaining_bandwidth as u64)
                {
                    Err(e) => tracing::warn!("Error while updating query coefficients: {:?}", e),
                    Ok(Some(new_duration)) => {
                        return Some(new_duration);
                    }
                    Ok(None) => {
                        let ticketbook_type = if entry {
                            TicketType::V1WireguardEntry
                        } else {
                            TicketType::V1WireguardExit
                        };
                        if let Err(e) = self
                            .top_up_bandwidth(ticketbook_type, &mut wg_gateway_client)
                            .await
                        {
                            tracing::warn!("Error topping up with more bandwidth {:?}", e);
                            // TODO: try to return this error in the JoinHandle instead
                            self.shutdown
                                .send_we_stopped(Box::new(ErrorMessage::OutOfBandwidth {
                                    gateway_id: Box::new(
                                        *wg_gateway_client.auth_recipient().gateway(),
                                    ),
                                    authenticator_address: Box::new(
                                        wg_gateway_client.auth_recipient(),
                                    ),
                                }));
                        }
                    }
                }
            }
            Ok(None) => {}
        }
        None
    }

    pub(crate) async fn run(mut self)
    where
        <St as Storage>::StorageError: Send + Sync + 'static,
    {
        // Skip the first, immediate tick
        self.timeout_check_interval.next().await;
        while !self.shutdown.is_shutdown() {
            tokio::select! {
                _ = self.shutdown.recv() => {
                    tracing::trace!("BandwidthController: Received shutdown");
                }
                _ = self.timeout_check_interval.next() => {
                    let current_period = self.timeout_check_interval.as_ref().period();
                    let entry_duration = self.check_bandwidth(true, current_period).await;
                    let exit_duration = self.check_bandwidth(false, current_period).await;
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
                        self.timeout_check_interval = IntervalStream::new(tokio::time::interval(minimal_duration));
                        // Skip the first, immediate tick
                        self.timeout_check_interval.next().await;
                    }
                }
            }
        }
    }
}
