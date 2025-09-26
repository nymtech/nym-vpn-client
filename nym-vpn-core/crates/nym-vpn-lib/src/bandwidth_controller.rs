// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{net::IpAddr, time::Duration};

use nym_authenticator_client::LegacyAuthenticatorClient;
use nym_bandwidth_controller::{BandwidthTicketProvider, DEFAULT_TICKETS_TO_SPEND};
use nym_registration_client::GatewayData;
use tokio_stream::{StreamExt, wrappers::IntervalStream};
use tokio_util::sync::CancellationToken;

use nym_common::ErrorExt;
use nym_config::defaults::{WG_METADATA_PORT, WG_TUN_DEVICE_IP_ADDRESS_V4};
use nym_credentials_interface::TicketType;
use nym_gateway_directory::Gateway;

use nym_wg_metadata_client::{MetadataClient, TunUpReceiver};
use nym_wireguard_types::DEFAULT_PEER_TIMEOUT_CHECK;
use url::Url;

use crate::tunnel_state_machine::tunnel::SelectedGateways;

const DEFAULT_BANDWIDTH_CHECK: Duration = Duration::from_secs(5); // 5 seconds
const LOWER_BOUND_CHECK_DURATION: Duration = DEFAULT_PEER_TIMEOUT_CHECK;
const UPPER_BOUND_CHECK_DURATION: Duration =
    Duration::from_secs(6 * DEFAULT_PEER_TIMEOUT_CHECK.as_secs());
const DEFAULT_BANDWIDTH_DEPLETION_RATE: u64 = 1024 * 1024; // 1 MB/s
const MINIMUM_RAMAINING_BANDWIDTH: u64 = 500 * 1024 * 1024; // 500 MB, the same as a wireguard ticket size (but it doesn't have to be)

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("entry gateway error")]
    EntryGateway(SpecificGatewayError),

    #[error("exit gateway error")]
    ExitGateway(SpecificGatewayError),

    #[error("nyxd client error")]
    Nyxd(#[from] CredentialNyxdClientError),

    #[error("connection cancelled")]
    Cancelled,
}

#[derive(Debug, thiserror::Error)]
pub enum SpecificGatewayError {
    #[error("failed to lookup gateway ip for {gateway_id}")]
    LookupGatewayIp {
        gateway_id: String,
        #[source]
        source: Box<nym_gateway_directory::Error>,
    },

    #[error("failed to register wireguard with the gateway for {gateway_id}")]
    RegisterWireguard {
        gateway_id: String,
        authenticator_address: Box<nym_gateway_directory::Recipient>,
        #[source]
        source: Box<nym_registration_client::RegistrationClientError>,
    },

    #[error("failed to request wireguard credential with the gateway: {gateway_id}")]
    RequestCredential {
        gateway_id: String,
        ticketbook_type: TicketType,
        #[source]
        source: Box<nym_bandwidth_controller::error::BandwidthControllerError>,
    },

    #[error("failed to top-up wireguard bandwidth with the gateway: {gateway_id}")]
    DeprecatedTopUpWireguard {
        gateway_id: String,
        ticketbook_type: TicketType,
        #[source]
        source: Box<nym_authenticator_client::Error>,
    },

    #[error("failed to top-up wireguard bandwidth with the gateway: {gateway_id}")]
    TopUpWireguard {
        gateway_id: String,
        ticketbook_type: TicketType,
        #[source]
        source: Box<nym_wg_metadata_client::error::MetadataClientError>,
    },

    #[error("internal error: {reason}")]
    Internal { reason: String },
}

impl SpecificGatewayError {
    pub fn is_no_retry(&self) -> bool {
        let more_specific_inner = match self {
            SpecificGatewayError::RegisterWireguard { source, .. } => source,
            SpecificGatewayError::RequestCredential { source, .. } => source,
            _ => return false,
        };
        matches!(
            **more_specific_inner,
            nym_wg_gateway_client::Error::NoRetry { .. }
        )
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CredentialNyxdClientError {
    #[error("Failed to create nyxd client config")]
    FailedToCreateNyxdClientConfig(nym_validator_client::nyxd::error::NyxdError),

    #[error("Failed to connect using nyxd client")]
    FailedToConnectUsingNyxdClient(nym_validator_client::nyxd::error::NyxdError),
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
    ) -> Result<Option<Duration>, SpecificGatewayError> {
        let Some(new_depletion_rate) = self
            .available_bandwidth
            .saturating_sub(remaining_bandwidth)
            .checked_div(current_period.as_secs())
        else {
            return Err(SpecificGatewayError::Internal {
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
            return Err(SpecificGatewayError::Internal {
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

pub(crate) enum TemporaryBandwidthClient {
    Deprecated(Box<LegacyAuthenticatorClient>),
    Latest(Box<MetadataClient>),
}

impl TemporaryBandwidthClient {
    pub(crate) fn new(
        gateway: &Gateway,
        authenticator_client: LegacyAuthenticatorClient,
        metadata_client: MetadataClient,
        gateway_metadata_update_version: Option<semver::Version>,
    ) -> Self {
        if let Some(gateway_version) = gateway.version.as_ref()
            && let Ok(gateway_version) = semver::Version::parse(gateway_version)
            && let Some(update_version) = gateway_metadata_update_version
            && gateway_version >= update_version
        {
            tracing::debug!(
                "Using latest metadata client for {}'s bandwidth controller",
                gateway.identity()
            );
            TemporaryBandwidthClient::Latest(Box::new(metadata_client))
        } else {
            tracing::debug!(
                "Using deprecated mixnet client for {}'s bandwidth controller",
                gateway.identity()
            );
            TemporaryBandwidthClient::Deprecated(Box::new(authenticator_client))
        }
    }

    pub(crate) async fn query_bandwidth(&mut self) -> Result<i64, String> {
        match self {
            TemporaryBandwidthClient::Deprecated(authenticator_client) => authenticator_client
                .query_bandwidth()
                .await
                .map_err(|e| e.display_chain_with_msg("error querying remaining bandwidth"))?
                .ok_or("No such peer on the gateway".to_string()),
            TemporaryBandwidthClient::Latest(metadata_client) => metadata_client
                .query_bandwidth()
                .await
                .map_err(|e| e.display_chain_with_msg("error querying remaining bandwidth")),
        }
    }

    pub(crate) fn gateway_id(&self) -> nym_gateway_directory::NodeIdentity {
        match self {
            TemporaryBandwidthClient::Deprecated(authenticator_client) => {
                authenticator_client.auth_recipient.gateway()
            }
            TemporaryBandwidthClient::Latest(metadata_client) => metadata_client.gateway_id(),
        }
    }

    pub(crate) async fn topup_bandwidth(
        &mut self,
        credential: nym_credentials_interface::CredentialSpendingData,
        ticketbook_type: TicketType,
    ) -> Result<i64, SpecificGatewayError> {
        match self {
            TemporaryBandwidthClient::Deprecated(authenticator_client) => authenticator_client
                .top_up(credential)
                .await
                .map_err(|source| Error::DeprecatedTopUpWireguard {
                    gateway_id: self.gateway_id().to_string(),
                    ticketbook_type,
                    source: Box::new(source),
                }),
            TemporaryBandwidthClient::Latest(metadata_client) => metadata_client
                .topup_bandwidth(credential)
                .await
                .map_err(|source| SpecificGatewayError::TopUpWireguard {
                    gateway_id: self.gateway_id().to_string(),
                    ticketbook_type,
                    source: Box::new(source),
                }),
        }
    }
}

pub(crate) struct BandwidthController {
    inner: Box<dyn BandwidthTicketProvider>,
    wg_entry_gateway_client: TemporaryBandwidthClient,
    wg_exit_gateway_client: TemporaryBandwidthClient,
    timeout_check_interval: IntervalStream,
    entry_depletion_rate: DepletionRate,
    exit_depletion_rate: DepletionRate,
    entry_previous_error_query: bool,
    exit_previous_error_query: bool,
    shutdown_token: CancellationToken,
}

impl BandwidthController {
    pub fn new(
        inner: Box<dyn BandwidthTicketProvider>,
        wg_entry_gateway_client: TemporaryBandwidthClient,
        wg_exit_gateway_client: TemporaryBandwidthClient,
        shutdown_token: CancellationToken,
    ) -> Self {
        let timeout_check_interval =
            IntervalStream::new(tokio::time::interval(DEFAULT_BANDWIDTH_CHECK));

        BandwidthController {
            inner,
            wg_entry_gateway_client,
            wg_exit_gateway_client,
            timeout_check_interval,
            entry_depletion_rate: Default::default(),
            exit_depletion_rate: Default::default(),
            entry_previous_error_query: false,
            exit_previous_error_query: false,
            shutdown_token,
        }
    }

    fn construct_bandwidth_client(
        bind_ip: IpAddr,
        signal_channel: TunUpReceiver,
        gateway: &Gateway,
        authenticator_client: LegacyAuthenticatorClient,
        gateway_metadata_update_version: Option<semver::Version>,
    ) -> TemporaryBandwidthClient {
        // this shouldn't fail, verified by unit test as well
        let gateway_private_url = Url::parse(&format!(
            "http://{WG_TUN_DEVICE_IP_ADDRESS_V4}:{WG_METADATA_PORT}"
        ))
        .expect("invalid gateway private URL");
        let metadata_client = MetadataClient::new(
            gateway_private_url,
            gateway.identity(),
            bind_ip,
            signal_channel,
        );
        TemporaryBandwidthClient::new(
            gateway,
            authenticator_client,
            metadata_client,
            gateway_metadata_update_version,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn create(
        bw_controller: Box<dyn BandwidthTicketProvider>,
        selected_gateways: SelectedGateways,
        entry_auth_client: LegacyAuthenticatorClient,
        exit_auth_client: LegacyAuthenticatorClient,
        entry_gateway_data: GatewayData,
        exit_gateway_data: GatewayData,
        entry_signal_channel: TunUpReceiver,
        exit_signal_channel: TunUpReceiver,
        gateway_metadata_update_version: Option<semver::Version>,
        cancel_token: CancellationToken,
    ) -> Result<BandwidthController> {
        let wg_entry_client = Self::construct_bandwidth_client(
            entry_gateway_data.private_ipv4.into(),
            entry_signal_channel,
            &selected_gateways.entry,
            entry_auth_client,
            gateway_metadata_update_version.clone(),
        );
        let wg_exit_client = Self::construct_bandwidth_client(
            exit_gateway_data.private_ipv4.into(),
            exit_signal_channel,
            &selected_gateways.exit,
            exit_auth_client,
            gateway_metadata_update_version,
        );

        let bw = Self::new(
            bw_controller,
            wg_entry_client,
            wg_exit_client,
            cancel_token.clone(),
        );

        Ok(bw)
    }

    pub(crate) async fn top_up_bandwidth(
        controller: &dyn BandwidthTicketProvider,
        ticketbook_type: TicketType,
        bw_client: &mut TemporaryBandwidthClient,
    ) -> Result<i64> {
        let credential = controller
            .get_ecash_ticket(
                ticketbook_type,
                bw_client.gateway_id(),
                DEFAULT_TICKETS_TO_SPEND,
            )
            .await
            .map_err(|source| Error::RequestCredential {
                gateway_id: bw_client.gateway_id().to_string(),
                ticketbook_type,
                source: Box::new(source),
            })?
            .data;
        let remaining_bandwidth = bw_client
            .topup_bandwidth(credential, ticketbook_type)
            .await?;
        Ok(remaining_bandwidth)
    }

    async fn check_bandwidth(&mut self, entry: bool, current_period: Duration) -> Option<Duration> {
        let (wg_metadata_client, current_depletion_rate) = if entry {
            (
                &mut self.wg_entry_gateway_client,
                &mut self.entry_depletion_rate,
            )
        } else {
            (
                &mut self.wg_exit_gateway_client,
                &mut self.exit_depletion_rate,
            )
        };

        tokio::select! {
            _ = self.shutdown_token.cancelled() => {
                tracing::trace!("BandwidthController: Received shutdown");
            }
            ret = wg_metadata_client.query_bandwidth() => {
                match ret {
                    Ok(remaining_bandwidth) => {
                        if entry {
                            self.entry_previous_error_query = false;
                        } else {
                            self.exit_previous_error_query = false;
                        }
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
                                if let Err(e) = Self::top_up_bandwidth(&*self.inner, ticketbook_type, wg_metadata_client)
                                    .await
                                {
                                    tracing::warn!("Error topping up with more bandwidth {:?}", e);
                                    // TODO: try to return this error in the JoinHandle instead
                                    // For now let's keep the old behavior of stopping
                                    self.shutdown_token.cancel();
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("{e}");
                        if (entry && self.entry_previous_error_query) || (!entry && self.exit_previous_error_query) {
                            tracing::error!("gateway {} is erroring out", wg_metadata_client.gateway_id());
                            // For now let's keep the old behavior of stopping
                            self.shutdown_token.cancel();
                        } else {
                            if entry {
                                self.entry_previous_error_query = true;
                            } else {
                                self.exit_previous_error_query = true;
                            }
                            tracing::info!("Empty query for {} gateway bandwidth check. This is normal, as long as it is not repeating for the same gateway", if entry {"entry".to_string()} else {"exit".to_string()});
                        }

                    }
                }
            }
        }
        None
    }

    pub(crate) async fn run(mut self) {
        // Skip the first, immediate tick
        self.timeout_check_interval.next().await;
        while !self.shutdown_token.is_cancelled() {
            tokio::select! {
                _ = self.shutdown_token.cancelled() => {
                    tracing::trace!("BandwidthController: Received shutdown");
                    break;
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
        assert!(
            depletion_rate
                .update_dynamic_check_interval(current_period, BW_1GB - consumed)
                .unwrap()
                .is_none()
        );
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

    #[test]
    fn parse_url() {
        assert!(
            Url::parse(&format!(
                "http://{WG_TUN_DEVICE_IP_ADDRESS_V4}:{WG_METADATA_PORT}"
            ))
            .is_ok()
        );
    }
}
