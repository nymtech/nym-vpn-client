// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use nym_vpn_network_config::Network;
use tokio_stream::{wrappers::IntervalStream, StreamExt};

use nym_credentials_interface::TicketType;
use nym_gateway_directory::GatewayClient;
use nym_sdk::{
    mixnet::{ConnectionStatsEvent, CredentialStorage as Storage},
    TaskClient,
};
use nym_validator_client::{
    nyxd::{Config as NyxdClientConfig, NyxdClient},
    QueryHttpRpcNyxdClient,
};
use nym_wg_gateway_client::{
    ErrorMessage, GatewayData, WgGatewayClient, WgGatewayLightClient, TICKETS_TO_SPEND,
};
use nym_wireguard_types::DEFAULT_PEER_TIMEOUT_CHECK;

const DEFAULT_BANDWIDTH_CHECK: Duration = Duration::from_secs(5); // 5 seconds
const LOWER_BOUND_CHECK_DURATION: Duration = DEFAULT_PEER_TIMEOUT_CHECK;
const UPPER_BOUND_CHECK_DURATION: Duration =
    Duration::from_secs(6 * DEFAULT_PEER_TIMEOUT_CHECK.as_secs());
const DEFAULT_BANDWIDTH_DEPLETION_RATE: u64 = 1024 * 1024; // 1 MB/s
const MINIMUM_RAMAINING_BANDWIDTH: u64 = 500 * 1024 * 1024; // 500 MB, the same as a wireguard ticket size (but it doesn't have to be)

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

    #[error("failed to connect using nyxd client: {0}")]
    FailedToConnectUsingNyxdClient(nym_validator_client::nyxd::error::NyxdError),
}

fn get_nyxd_client(network: &Network) -> Result<QueryHttpRpcNyxdClient> {
    let config = NyxdClientConfig::try_from_nym_network_details(&network.nym_network.network)
        .map_err(CredentialNyxdClientError::FailedToCreateNyxdClientConfig)?;

    Ok(NyxdClient::connect(config, network.nyxd_url().as_str())
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
            available_bandwidth: 0,
        }
    }
}

impl DepletionRate {
    fn update_dynamic_check_interval(
        &mut self,
        current_period: Duration,
        remaining_bandwidth: u64,
    ) -> Result<Option<Duration>> {
        let Some(new_depletion_rate) = self
            .available_bandwidth
            .saturating_sub(remaining_bandwidth)
            .checked_div(current_period.as_secs())
        else {
            return Err(Error::Internal {
                reason: "check interval shouldn't be 0".to_string(),
            });
        };
        tracing::debug!(
            "current depletion rate of {} bytes per current check period of {} seconds",
            new_depletion_rate,
            current_period.as_secs()
        );
        self.available_bandwidth = remaining_bandwidth;
        // if nothing was consumed since last time, it's possible we had a recent topup already,
        // so take the safer approach of waiting minimal interval
        if new_depletion_rate != 0 {
            self.current_depletion_rate = new_depletion_rate;
        } else {
            return Ok(Some(DEFAULT_PEER_TIMEOUT_CHECK));
        }
        let Some(estimated_depletion_secs) =
            remaining_bandwidth.checked_div(self.current_depletion_rate)
        else {
            return Err(Error::Internal {
                reason: "depletion rate shouldn't be 0".to_string(),
            });
        };
        tracing::debug!(
            "estimated to deplete current bandwidth in {} seconds = ",
            estimated_depletion_secs
        );

        let number_of_checks_before_depletion = estimated_depletion_secs
            .checked_div(current_period.as_secs())
            .unwrap_or_default();
        // try and have at least 10 checks before depletion, to be on the safe side...
        if number_of_checks_before_depletion < 10 {
            return Ok(None);
        }
        // have an above the water minimum, just in case
        if self.available_bandwidth < MINIMUM_RAMAINING_BANDWIDTH {
            return Ok(None);
        }
        if estimated_depletion_secs > UPPER_BOUND_CHECK_DURATION.as_secs() {
            // ... but not too slow, in case bursts come in
            Ok(Some(UPPER_BOUND_CHECK_DURATION))
        } else if estimated_depletion_secs < LOWER_BOUND_CHECK_DURATION.as_secs() {
            // ... and not faster then the gateway bandwidth refresh, as that won't produce any change
            Ok(Some(LOWER_BOUND_CHECK_DURATION))
        } else {
            Ok(Some(Duration::from_secs(number_of_checks_before_depletion)))
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
        network: &Network,
        wg_entry_gateway_client: WgGatewayLightClient,
        wg_exit_gateway_client: WgGatewayLightClient,
        shutdown: TaskClient,
    ) -> Result<Self> {
        let client = get_nyxd_client(network)?;
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
        // First we need to regster with the gateway to setup keys and IP assignment
        let wg_version = wg_gateway_client.auth_version();
        let authenticator_address = wg_gateway_client.auth_recipient();
        let gateway_id = wg_gateway_client.auth_recipient().gateway();
        tracing::info!("Registering with wireguard gateway {gateway_id} ({wg_version})");
        let gateway_host = gateway_client
            .lookup_gateway_ip(&gateway_id.to_base58_string())
            .await
            .map_err(|source| Error::LookupGatewayIp {
                gateway_id: gateway_id.to_base58_string(),
                source,
            })?;
        let wg_gateway_data = wg_gateway_client
            .register_wireguard(
                gateway_host,
                &self.inner,
                enable_credentials_mode,
                ticketbook_type,
            )
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
        let authenticator_address = wg_gateway_client.auth_recipient();
        let gateway_id = wg_gateway_client.auth_recipient().gateway();
        let remaining_bandwidth =
            WgGatewayClient::top_up_wireguard(wg_gateway_client, &self.inner, ticketbook_type)
                .await
                .map_err(|source| Error::TopUpWireguard {
                    gateway_id: gateway_id.to_string(),
                    ticketbook_type,
                    authenticator_address: Box::new(authenticator_address),
                    source,
                })?;
        wg_gateway_client.send_stats_event(
            ConnectionStatsEvent::TicketSpent {
                typ: ticketbook_type,
                amount: TICKETS_TO_SPEND,
            }
            .into(),
        );
        Ok(remaining_bandwidth)
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

        tokio::select! {
            _ = self.shutdown.recv() => {
                tracing::trace!("BandwidthController: Received shutdown");
            }
            ret = wg_gateway_client.query_bandwidth() => {
                match ret {
                    Err(e) => tracing::warn!("Error querying remaining bandwidth {:?}", e),
                    Ok(Some(remaining_bandwidth)) => {
                        match current_depletion_rate
                            .update_dynamic_check_interval(current_period, remaining_bandwidth as u64)
                        {
                            Err(e) => tracing::warn!("Error while updating query coefficients: {:?}", e),
                            Ok(Some(new_duration)) => {
                                tracing::debug!("Adjusting check interval to {} seconds", new_duration.as_secs());
                                return Some(new_duration);
                            }
                            Ok(None) => {
                                let ticketbook_type = if entry {
                                    TicketType::V1WireguardEntry
                                } else {
                                    TicketType::V1WireguardExit
                                };
                                tracing::debug!("Topping up our bandwidth allowance for {ticketbook_type}");
                                if let Err(e) = self
                                    .top_up_bandwidth(ticketbook_type, &mut wg_gateway_client)
                                    .await
                                {
                                    tracing::warn!("Error topping up with more bandwidth {:?}", e);
                                    // TODO: try to return this error in the JoinHandle instead
                                    self.shutdown
                                        .send_we_stopped(Box::new(ErrorMessage::OutOfBandwidth {
                                            gateway_id: Box::new(
                                                wg_gateway_client.auth_recipient().gateway(),
                                            ),
                                            authenticator_address: Box::new(
                                                wg_gateway_client.auth_recipient(),
                                            ),
                                        }));
                                }
                            }
                        }
                    }
                    Ok(None) => {
                        tracing::info!("Empty query for {} gadeway bandwidth check. This is normal, as long as it is not repeating for the same gateway", if entry {"entry".to_string()} else {"exit".to_string()});
                    }
                }
            }
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

        tracing::debug!("BandwidthController: Exiting");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BW_1KB: u64 = 1024;
    const BW_1MB: u64 = 1024 * BW_1KB;
    const BW_128MB: u64 = 128 * BW_1MB;
    const BW_512MB: u64 = 512 * BW_1MB;
    const BW_1GB: u64 = 2 * BW_512MB;

    #[test]
    fn depletion_rate_slow() {
        let mut depletion_rate = DepletionRate::default();
        let mut current_period = DEFAULT_BANDWIDTH_CHECK;
        // the first check would force the placeholder values to be replaced by the actual values
        assert_eq!(
            depletion_rate
                .update_dynamic_check_interval(current_period, BW_512MB)
                .unwrap(),
            Some(DEFAULT_BANDWIDTH_CHECK)
        );

        // simulate 1 byte/second depletion rate
        let consumed = current_period.as_secs();
        current_period = depletion_rate
            .update_dynamic_check_interval(current_period, BW_512MB - consumed)
            .unwrap()
            .unwrap();
        assert_eq!(current_period, UPPER_BOUND_CHECK_DURATION);
    }

    #[test]
    fn depletion_rate_fast() {
        let mut depletion_rate = DepletionRate::default();
        let current_period = DEFAULT_BANDWIDTH_CHECK;
        // the first check would force the placeholder values to be replaced by the actual values
        assert_eq!(
            depletion_rate
                .update_dynamic_check_interval(current_period, BW_1GB)
                .unwrap(),
            Some(DEFAULT_BANDWIDTH_CHECK)
        );

        // simulate 128 MB/s depletion rate, so we would be depleted in the next 5 seconds after the function call (too fast)
        let consumed = current_period.as_secs() * BW_128MB;
        assert!(depletion_rate
            .update_dynamic_check_interval(current_period, BW_1GB - consumed)
            .unwrap()
            .is_none());
    }

    #[test]
    fn depletion_rate_spike() {
        let mut depletion_rate = DepletionRate::default();
        let mut current_period = DEFAULT_BANDWIDTH_CHECK;
        let mut current_bandwidth = BW_1GB;
        // the first check would force the placeholder values to be replaced by the actual values
        assert_eq!(
            depletion_rate
                .update_dynamic_check_interval(current_period, BW_1GB)
                .unwrap(),
            Some(DEFAULT_BANDWIDTH_CHECK)
        );

        // simulate 1 KB/s depletion rate, constant
        for _ in 0..5 {
            current_bandwidth -= current_period.as_secs() * BW_1KB;
            current_period = depletion_rate
                .update_dynamic_check_interval(current_period, current_bandwidth)
                .unwrap()
                .unwrap();
            assert_eq!(current_period, UPPER_BOUND_CHECK_DURATION);
        }

        // spike a 1 MB/s depletion rate
        for _ in 0..17 {
            current_bandwidth -= current_period.as_secs() * BW_1MB;
            current_period = depletion_rate
                .update_dynamic_check_interval(current_period, current_bandwidth)
                .unwrap()
                .unwrap();
            assert_eq!(current_period, UPPER_BOUND_CHECK_DURATION);
            assert!(current_bandwidth > 500 * BW_1MB);
        }

        current_bandwidth -= current_period.as_secs() * BW_1MB;
        let ret = depletion_rate
            .update_dynamic_check_interval(current_period, current_bandwidth)
            .unwrap();
        // when we get bellow a convinient dynamic threshold, we start reqwesting more bandwidth (returning None)
        assert!(current_bandwidth < 500 * BW_1MB);
        assert!(ret.is_none());
    }
}
