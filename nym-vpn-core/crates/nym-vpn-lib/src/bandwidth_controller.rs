// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{net::IpAddr, time::Duration};

use nym_authenticator_client::AuthenticatorClient;
use nym_bandwidth_controller::{BandwidthTicketProvider, DEFAULT_TICKETS_TO_SPEND};
use nym_registration_common::WireguardConfiguration;
use tokio_stream::{StreamExt, wrappers::IntervalStream};
use tokio_util::sync::CancellationToken;

use nym_config::defaults::{WG_METADATA_PORT, WG_TUN_DEVICE_IP_ADDRESS_V4};
use nym_credentials_interface::TicketType;
use nym_gateway_directory::Gateway;

use crate::tunnel_state_machine::tunnel::SelectedGateways;
use nym_vpn_account_controller::AccountCommandSender;
use nym_wg_metadata_client::{MetadataClient, TunUpReceiver, error::MetadataClientError};
use nym_wireguard_types::DEFAULT_PEER_TIMEOUT_CHECK;
use tracing::{debug, error, info, trace, warn};
use url::Url;

const DEFAULT_BANDWIDTH_CHECK: Duration = Duration::from_secs(5); // 5 seconds
const LOWER_BOUND_CHECK_DURATION: Duration = DEFAULT_PEER_TIMEOUT_CHECK;
const UPPER_BOUND_CHECK_DURATION: Duration =
    Duration::from_secs(6 * DEFAULT_PEER_TIMEOUT_CHECK.as_secs());
const DEFAULT_BANDWIDTH_DEPLETION_RATE: u64 = 1024 * 1024; // 1 MB/s
const MINIMUM_RAMAINING_BANDWIDTH: u64 = 500 * 1024 * 1024; // 500 MB, the same as a wireguard ticket size (but it doesn't have to be)

const DEFAULT_CLIENT_RETRIES: usize = 1;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("entry gateway error")]
    EntryGateway(#[source] SpecificGatewayError),

    #[error("exit gateway error")]
    ExitGateway(#[source] SpecificGatewayError),

    #[error("nyxd client error")]
    Nyxd(#[from] CredentialNyxdClientError),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("connection cancelled")]
    Cancelled,
}

impl Error {
    pub fn internal(msg: impl Into<String>) -> Self {
        Error::Internal(msg.into())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SpecificGatewayError {
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
        source: Box<nym_authenticator_client::AuthenticationClientError>,
    },

    #[error("failed to top-up wireguard bandwidth with the gateway: {gateway_id}")]
    TopUpWireguard {
        gateway_id: String,
        ticketbook_type: TicketType,
        #[source]
        source: Box<MetadataClientError>,
    },

    #[error("failed to query bandwidth from gateway: {gateway_id}")]
    DeprecatedQueryBandwidth {
        gateway_id: String,
        #[source]
        source: Box<nym_authenticator_client::AuthenticationClientError>,
    },

    #[error("failed to query bandwidth from gateway: {gateway_id}")]
    QueryBandwidth {
        gateway_id: String,
        #[source]
        source: Box<MetadataClientError>,
    },

    #[error("failed to request upgrade mode state recheck with the gateway: {gateway_id}")]
    DeprecatedUpgradeModeRecheck {
        gateway_id: String,
        source: Box<nym_authenticator_client::AuthenticationClientError>,
    },

    #[error("failed to request upgrade mode state recheck with the gateway: {gateway_id}")]
    UpgradeModeRecheck {
        gateway_id: String,
        #[source]
        source: Box<nym_wg_metadata_client::error::MetadataClientError>,
    },

    #[error("timed-out while communicating with gateway: {gateway_id}")]
    GatewayTimeout { gateway_id: String },

    #[error("internal error: {reason}")]
    Internal { reason: String },
}

impl SpecificGatewayError {
    pub fn is_no_retry(&self) -> bool {
        matches!(
            self,
            SpecificGatewayError::DeprecatedTopUpWireguard { .. }
                | SpecificGatewayError::TopUpWireguard { .. }
        )
    }

    pub fn from_deprecated_topup_wireguard(
        gateway_id: String,
        ticketbook_type: TicketType,
        source: nym_authenticator_client::AuthenticationClientError,
    ) -> Self {
        if matches!(
            source,
            nym_authenticator_client::AuthenticationClientError::TimeoutWaitingForConnectResponse
        ) {
            return SpecificGatewayError::GatewayTimeout { gateway_id };
        }

        SpecificGatewayError::DeprecatedTopUpWireguard {
            gateway_id,
            ticketbook_type,
            source: Box::new(source),
        }
    }

    pub fn from_topup_wireguard(
        gateway_id: String,
        ticketbook_type: TicketType,
        source: MetadataClientError,
    ) -> Self {
        SpecificGatewayError::TopUpWireguard {
            gateway_id,
            ticketbook_type,
            source: Box::new(source),
        }
    }

    pub fn from_deprecated_query_bandwidth(
        gateway_id: String,
        source: nym_authenticator_client::AuthenticationClientError,
    ) -> Self {
        if matches!(
            source,
            nym_authenticator_client::AuthenticationClientError::TimeoutWaitingForConnectResponse
        ) {
            return SpecificGatewayError::GatewayTimeout { gateway_id };
        }

        SpecificGatewayError::DeprecatedQueryBandwidth {
            gateway_id,
            source: Box::new(source),
        }
    }

    pub fn from_query_bandwidth(gateway_id: String, source: MetadataClientError) -> Self {
        SpecificGatewayError::QueryBandwidth {
            gateway_id,
            source: Box::new(source),
        }
    }

    pub fn from_deprecated_upgrade_mode_recheck_bandwidth(
        gateway_id: String,
        source: nym_authenticator_client::AuthenticationClientError,
    ) -> Self {
        if matches!(
            source,
            nym_authenticator_client::AuthenticationClientError::TimeoutWaitingForConnectResponse
        ) {
            return SpecificGatewayError::GatewayTimeout { gateway_id };
        }

        SpecificGatewayError::DeprecatedUpgradeModeRecheck {
            gateway_id,
            source: Box::new(source),
        }
    }

    pub fn from_upgrade_mode_recheck_bandwidth(
        gateway_id: String,
        source: MetadataClientError,
    ) -> Self {
        SpecificGatewayError::UpgradeModeRecheck {
            gateway_id,
            source: Box::new(source),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UpgradeModeRecheckError {
    #[error("failed to request upgrade mode token to use with the gateway: {gateway_id}")]
    UpgradeModeTokenRequest {
        gateway_id: String,
        #[source]
        source: Box<nym_bandwidth_controller::error::BandwidthControllerError>,
    },

    // **theoretically** this should never get thrown
    #[error(
        "failed to retrieve upgrade mode JWT from storage even though Account Controller reports the upgrade mode. gateway: {gateway_id}"
    )]
    UnavailableUpgradeModeToken { gateway_id: String },

    #[error(transparent)]
    GatewayQuery(#[from] SpecificGatewayError),
}

#[derive(Debug, thiserror::Error)]
pub enum CredentialNyxdClientError {
    #[error("Failed to create nyxd client config")]
    FailedToCreateNyxdClientConfig(nym_validator_client::nyxd::error::NyxdError),

    #[error("Failed to connect using nyxd client")]
    FailedToConnectUsingNyxdClient(nym_validator_client::nyxd::error::NyxdError),
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct AvailableBandwidth {
    pub(crate) bandwidth_bytes: i64,
    pub(crate) upgrade_mode: Option<bool>,
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
    Deprecated(Box<AuthenticatorClient>),
    Latest(Box<MetadataClient>),
}

impl TemporaryBandwidthClient {
    pub(crate) fn new(
        gateway: &Gateway,
        authenticator_client: Option<AuthenticatorClient>,
        metadata_client: MetadataClient,
        gateway_metadata_update_version: Option<semver::Version>,
    ) -> Self {
        if let Some(auth_client) = authenticator_client {
            if let Some(gateway_version) = gateway.version.as_ref()
                && let Ok(gateway_version) = semver::Version::parse(gateway_version)
                && let Some(update_version) = gateway_metadata_update_version
                && gateway_version >= update_version
                && gateway
                    .last_probe
                    .as_ref()
                    .and_then(|p| p.outcome.wg.as_ref())
                    .map(|r| r.can_query_metadata_v4)
                    .unwrap_or(false)
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
                TemporaryBandwidthClient::Deprecated(Box::new(auth_client))
            }
        } else {
            tracing::debug!(
                "No authenticator client provided, using latest metadata client for {}'s bandwidth controller",
                gateway.identity()
            );
            TemporaryBandwidthClient::Latest(Box::new(metadata_client))
        }
    }

    pub(crate) async fn query_bandwidth(
        &mut self,
    ) -> Result<AvailableBandwidth, SpecificGatewayError> {
        match self {
            TemporaryBandwidthClient::Deprecated(authenticator_client) => {
                let response = authenticator_client.query_bandwidth().await.map_err(|e| {
                    SpecificGatewayError::from_deprecated_query_bandwidth(
                        self.gateway_id().to_string(),
                        e,
                    )
                })?;
                let Some(bandwidth_bytes) = response.available_bandwidth_bytes else {
                    return Err(SpecificGatewayError::Internal {
                        reason: "No such peer on the gateway".to_string(),
                    });
                };
                Ok(AvailableBandwidth {
                    bandwidth_bytes,
                    upgrade_mode: response.current_upgrade_mode_status.into(),
                })
            }
            TemporaryBandwidthClient::Latest(metadata_client) => {
                let response = metadata_client.query_bandwidth().await.map_err(|e| {
                    SpecificGatewayError::from_query_bandwidth(self.gateway_id().to_string(), e)
                })?;
                Ok(AvailableBandwidth {
                    bandwidth_bytes: response.bandwidth_bytes,
                    upgrade_mode: response.upgrade_mode,
                })
            }
        }
    }

    pub(crate) async fn query_bandwidth_with_retries(
        &mut self,
        retries: usize,
    ) -> Result<AvailableBandwidth, SpecificGatewayError> {
        let mut res = Ok(AvailableBandwidth::default());
        for attempt in 0..retries + 1 {
            tracing::debug!(
                "Attempt #{} to query bandwidth of gateway {}...",
                attempt + 1,
                self.gateway_id().to_string()
            );
            res = self.query_bandwidth().await;
            let Err(err) = &res else {
                // Success
                break;
            };
            let SpecificGatewayError::GatewayTimeout { .. } = &err else {
                // Error wasn't a timeout
                break;
            };
        }
        res
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
    ) -> Result<AvailableBandwidth, SpecificGatewayError> {
        match self {
            TemporaryBandwidthClient::Deprecated(authenticator_client) => {
                let response = authenticator_client.top_up(credential).await.map_err(|e| {
                    SpecificGatewayError::from_deprecated_topup_wireguard(
                        self.gateway_id().to_string(),
                        ticketbook_type,
                        e,
                    )
                })?;
                Ok(AvailableBandwidth {
                    bandwidth_bytes: response.remaining_bandwidth_bytes,
                    upgrade_mode: response.current_upgrade_mode_status.into(),
                })
            }
            TemporaryBandwidthClient::Latest(metadata_client) => {
                let response = metadata_client
                    .topup_bandwidth(credential)
                    .await
                    .map_err(|e| {
                        SpecificGatewayError::from_topup_wireguard(
                            self.gateway_id().to_string(),
                            ticketbook_type,
                            e,
                        )
                    })?;
                Ok(AvailableBandwidth {
                    bandwidth_bytes: response.bandwidth_bytes,
                    upgrade_mode: response.upgrade_mode,
                })
            }
        }
    }

    pub(crate) async fn request_upgrade_mode_recheck(
        &mut self,
        // this argument will change in the future once we have different kinds of emergency credentials
        upgrade_mode_jwt: String,
    ) -> Result<bool, SpecificGatewayError> {
        match self {
            TemporaryBandwidthClient::Deprecated(authenticator_client) => {
                let upgrade_mode_enabled = authenticator_client
                    .check_upgrade_mode(upgrade_mode_jwt)
                    .await
                    .map_err(|err| {
                        SpecificGatewayError::from_deprecated_upgrade_mode_recheck_bandwidth(
                            self.gateway_id().to_string(),
                            err,
                        )
                    })?;
                Ok(upgrade_mode_enabled)
            }
            TemporaryBandwidthClient::Latest(metadata_client) => {
                let upgrade_mode_enabled = metadata_client
                    .check_upgrade_mode(upgrade_mode_jwt)
                    .await
                    .map_err(|err| {
                        SpecificGatewayError::from_upgrade_mode_recheck_bandwidth(
                            self.gateway_id().to_string(),
                            err,
                        )
                    })?;
                Ok(upgrade_mode_enabled)
            }
        }
    }
}

pub(crate) struct BandwidthController {
    ticket_provider: Box<dyn BandwidthTicketProvider>,
    wg_entry_gateway_client: TemporaryBandwidthClient,
    wg_exit_gateway_client: TemporaryBandwidthClient,
    account_command_tx: AccountCommandSender,
    timeout_check_interval: IntervalStream,
    entry_depletion_rate: DepletionRate,
    exit_depletion_rate: DepletionRate,
    entry_previous_error_query: bool,
    exit_previous_error_query: bool,
    shutdown_token: CancellationToken,
    successful_checks: u64,
    upgrade_mode_enabled_on_last_check: bool,
}

impl BandwidthController {
    pub fn new(
        ticket_provider: Box<dyn BandwidthTicketProvider>,
        wg_entry_gateway_client: TemporaryBandwidthClient,
        wg_exit_gateway_client: TemporaryBandwidthClient,
        account_command_tx: AccountCommandSender,
        shutdown_token: CancellationToken,
    ) -> Self {
        let timeout_check_interval =
            IntervalStream::new(tokio::time::interval(DEFAULT_BANDWIDTH_CHECK));

        BandwidthController {
            ticket_provider,
            wg_entry_gateway_client,
            wg_exit_gateway_client,
            account_command_tx,
            timeout_check_interval,
            entry_depletion_rate: Default::default(),
            exit_depletion_rate: Default::default(),
            entry_previous_error_query: false,
            exit_previous_error_query: false,
            shutdown_token,
            successful_checks: 0,
            upgrade_mode_enabled_on_last_check: false,
        }
    }

    fn construct_bandwidth_client(
        bind_ip: IpAddr,
        signal_channel: TunUpReceiver,
        gateway: &Gateway,
        authenticator_client: Option<AuthenticatorClient>,
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
            DEFAULT_CLIENT_RETRIES,
        );
        TemporaryBandwidthClient::new(
            gateway,
            authenticator_client,
            metadata_client,
            gateway_metadata_update_version,
        )
    }

    fn gateway_id(&self, entry: bool) -> nym_gateway_directory::NodeIdentity {
        if entry {
            self.wg_entry_gateway_client.gateway_id()
        } else {
            self.wg_exit_gateway_client.gateway_id()
        }
    }

    fn depletion_rate(&mut self, entry: bool) -> &mut DepletionRate {
        if entry {
            &mut self.entry_depletion_rate
        } else {
            &mut self.exit_depletion_rate
        }
    }

    fn ticket_type(&self, entry: bool) -> TicketType {
        if entry {
            TicketType::V1WireguardEntry
        } else {
            TicketType::V1WireguardExit
        }
    }

    pub(crate) fn is_using_latest_client(&self) -> bool {
        matches!(
            self.wg_entry_gateway_client,
            TemporaryBandwidthClient::Latest(_)
        ) && matches!(
            self.wg_exit_gateway_client,
            TemporaryBandwidthClient::Latest(_)
        )
    }

    async fn got_upgrade_mode_attestation(&self) -> bool {
        // in case of failure we assume conservative case of NOT having the attestation
        self.account_command_tx
            .query_upgrade_mode_enabled()
            .await
            .inspect_err(|err| {
                error!("critical failure: failed to resolve account controller query: {err}")
            })
            .unwrap_or_default()
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn create(
        ticket_provider: Box<dyn BandwidthTicketProvider>,
        account_command_tx: AccountCommandSender,
        selected_gateways: &SelectedGateways,
        entry_auth_client: Option<AuthenticatorClient>,
        exit_auth_client: Option<AuthenticatorClient>,
        entry_wireguard_config: &WireguardConfiguration,
        exit_wireguard_config: &WireguardConfiguration,
        entry_signal_channel: TunUpReceiver,
        exit_signal_channel: TunUpReceiver,
        gateway_metadata_update_version: Option<semver::Version>,
        cancel_token: CancellationToken,
    ) -> BandwidthController {
        let wg_entry_client = Self::construct_bandwidth_client(
            entry_wireguard_config.private_ipv4.into(),
            entry_signal_channel,
            selected_gateways.entry_gateway(),
            entry_auth_client,
            gateway_metadata_update_version.clone(),
        );
        let wg_exit_client = Self::construct_bandwidth_client(
            exit_wireguard_config.private_ipv4.into(),
            exit_signal_channel,
            selected_gateways.exit_gateway(),
            exit_auth_client,
            gateway_metadata_update_version,
        );

        Self::new(
            ticket_provider,
            wg_entry_client,
            wg_exit_client,
            account_command_tx,
            cancel_token.clone(),
        )
    }

    pub(crate) async fn top_up_bandwidth(
        &mut self,
        entry: bool,
    ) -> Result<AvailableBandwidth, SpecificGatewayError> {
        let ticketbook_type = self.ticket_type(entry);
        tracing::debug!("Topping up our bandwidth allowance for {ticketbook_type}");

        let bw_client = if entry {
            &mut self.wg_entry_gateway_client
        } else {
            &mut self.wg_exit_gateway_client
        };

        let credential = self
            .ticket_provider
            .get_ecash_ticket(
                ticketbook_type,
                bw_client.gateway_id(),
                DEFAULT_TICKETS_TO_SPEND,
            )
            .await
            .map_err(|source| SpecificGatewayError::RequestCredential {
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

    pub(crate) async fn request_upgrade_mode_recheck(
        &mut self,
        entry: bool,
    ) -> Result<bool, UpgradeModeRecheckError> {
        let bw_client = if entry {
            &mut self.wg_entry_gateway_client
        } else {
            &mut self.wg_exit_gateway_client
        };

        let Some(upgrade_mode_jwt) = self
            .ticket_provider
            .get_upgrade_mode_token()
            .await
            .map_err(|source| UpgradeModeRecheckError::UpgradeModeTokenRequest {
                gateway_id: bw_client.gateway_id().to_string(),
                source: Box::new(source),
            })?
        else {
            return Err(UpgradeModeRecheckError::UnavailableUpgradeModeToken {
                gateway_id: bw_client.gateway_id().to_string(),
            });
        };

        let upgrade_mode_enabled = bw_client
            .request_upgrade_mode_recheck(upgrade_mode_jwt)
            .await?;
        Ok(upgrade_mode_enabled)
    }

    async fn handle_bandwidth_query_error(&mut self, entry: bool, err: SpecificGatewayError) {
        tracing::warn!("{err}");
        let gateway_id = self.gateway_id(entry);
        if (entry && self.entry_previous_error_query) || (!entry && self.exit_previous_error_query)
        {
            tracing::error!("gateway {gateway_id} is erroring out",);
            // For now let's keep the old behavior of stopping, but only if we've had a successful check before
            if self.successful_checks != 0 {
                self.shutdown_token.cancel();
            }
        } else {
            if entry {
                self.entry_previous_error_query = true;
            } else {
                self.exit_previous_error_query = true;
            }
            tracing::info!(
                "Empty query for {} gateway bandwidth check. This is normal, as long as it is not repeating for the same gateway",
                if entry {
                    "entry".to_string()
                } else {
                    "exit".to_string()
                }
            );
        }
    }

    async fn handle_bandwidth_query(
        &mut self,
        entry: bool,
        current_period: Duration,
        query_result: AvailableBandwidth,
    ) -> Option<Duration> {
        let remaining_bandwidth = query_result.bandwidth_bytes;
        let gw_upgrade_mode = query_result.upgrade_mode;
        let gateway_id = self.gateway_id(entry);

        self.successful_checks += 1;

        if entry {
            self.entry_previous_error_query = false;
        } else {
            self.exit_previous_error_query = false;
        }

        let current_depletion_rate = self.depletion_rate(entry);

        match current_depletion_rate
            .update_dynamic_check_interval(current_period, remaining_bandwidth as u64)
        {
            Err(e) => {
                tracing::warn!("Error while updating query coefficients: {e:?}");
                return None;
            }
            Ok(Some(new_duration)) => {
                let secs = new_duration.as_secs();
                tracing::debug!("Adjusting check interval to {secs} seconds");
                return Some(new_duration);
            }
            Ok(None) => {}
        }

        let got_um_data = self.got_upgrade_mode_attestation().await;

        // attempt to perform bandwidth top-up, if applicable
        match gw_upgrade_mode {
            None => {
                // no UM support - we have to attempt to send zk-nym otherwise we won't be able to communicate with it much longer
                warn!("gateway {gateway_id} is outdated and does not support upgrade mode queries")
            }
            Some(true) => {
                // gateway informed us it is currently in upgrade mode - we don't have to do anything
                debug!(
                    "gateway {gateway_id} is already in upgrade mode - no need to perform bandwidth top up"
                );
                if !got_um_data {
                    // account controller is not aware of the UM - it didn't have to request bandwidth
                    // from VPN API and thus hasn't received UM JWT
                    // this can happen for clients with a lot of stored ticketbooks. there's nothing
                    // inherently wrong with it
                    debug!("however, we do not possess a corresponding upgrade mode attestation");
                }
                self.upgrade_mode_enabled_on_last_check = true;
                return None;
            }
            Some(false) => {
                if got_um_data && !self.upgrade_mode_enabled_on_last_check {
                    // if we have relevant attestation and the gateway is not in upgrade mode,
                    // we need to trigger it to perform internal state refresh
                    info!(
                        "gateway {gateway_id} is not aware of the upgrade mode that has been triggered"
                    );
                    // there are some legit EDGE CASES where this can fail. consider the following scenario:
                    // 1. upgrade mode has JUST been triggered - gateway doesn't know about it yet
                    // 2. we learned about it within split a second - we got lucky because we just queried vpn api
                    // 3. there's some networking issue happening on the grand internet, e.g. some BGP problems or high AWS latency, whatever,
                    // and nym.com is returning old cached attestation.json copy to the gateway
                    // 4. recheck fails and enters into rate limiting mode for another few seconds,
                    // so I guess that's a long way of saying, if this fails, don't shut down,
                    // but instead treat it as a retryable failure
                    if let Err(err) = self.request_upgrade_mode_recheck(entry).await {
                        warn!("Error requesting upgrade mode recheck: {err:?}");
                    }
                    // since we didn't manage to trigger gateway to go into the upgrade mode,
                    // if we want to continue the connection we have to attempt to send a zk-nym instead
                    // (so we exit the match statement)
                } else if got_um_data && self.upgrade_mode_enabled_on_last_check {
                    // UM is over - we inform AC and top up bandwidth as normal
                    info!("gateway {gateway_id} has informed us the upgrade mode has finished");
                    if let Err(err) = self.account_command_tx.send_disable_upgrade_mode().await {
                        error!("error sending message to the account controller: {err}");
                        // we need to trigger a shutdown here because this message must not fail,
                        // if it did, AC won't exit upgrade mode state and won't resume acquiring zk-nyms
                        self.shutdown_token.cancel();
                        return None;
                    }
                    // we continue sending zk-nym
                } else {
                    // if we got here it means we don't have any attestation data and gateway said
                    // it's not in upgrade mode, meaning it's business as usual
                    // so continue and attempt to top-up bandwidth with a zk-nym
                    trace!("upgrade mode is not enabled anywhere in the system");
                }
            }
        }

        if let Err(e) = self.top_up_bandwidth(entry).await {
            tracing::warn!("Error topping up with more bandwidth {e:?}");
            // TODO: try to return this error in the JoinHandle instead
            // For now let's keep the old behavior of stopping
            self.shutdown_token.cancel();
        }

        None
    }

    async fn check_bandwidth(&mut self, entry: bool, current_period: Duration) -> Option<Duration> {
        let bw_client = if entry {
            &mut self.wg_entry_gateway_client
        } else {
            &mut self.wg_exit_gateway_client
        };
        tokio::select! {
            _ = self.shutdown_token.cancelled() => {
                tracing::trace!("BandwidthController: Received shutdown");
            }
            ret = bw_client.query_bandwidth_with_retries(DEFAULT_CLIENT_RETRIES) => {
                match ret {
                    Ok(query_res) => return self.handle_bandwidth_query(entry, current_period, query_res).await,
                    Err(err) => self.handle_bandwidth_query_error(entry, err).await,
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
