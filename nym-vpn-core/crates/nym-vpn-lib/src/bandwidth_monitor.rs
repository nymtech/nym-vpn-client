// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{net::IpAddr, time::Duration};

use nym_authenticator_client::AuthenticatorClient;

use crate::tunnel_health::{MetadataPathHealth, update_metadata_path_health};
use nym_bandwidth_controller::{
    DEFAULT_TICKETS_TO_SPEND, requests::BandwidthControllerRequestSender,
};
use nym_registration_common::WireguardConfiguration;
use nym_vpn_api_client::SkewManager;
use sysinfo::Networks;
use tokio_stream::{StreamExt, wrappers::IntervalStream};
use tokio_util::sync::CancellationToken;

use nym_config::defaults::{WG_METADATA_PORT, WG_TUN_DEVICE_IP_ADDRESS_V4};
use nym_credentials_interface::TicketType;
use nym_gateway_directory::Gateway;

use crate::tunnel_state_machine::tunnel::SelectedGateways;
use nym_common::trace_err_chain;
use nym_wg_metadata_client::{MetadataClient, TunUpReceiver, error::MetadataClientError};
use nym_wireguard_types::DEFAULT_PEER_TIMEOUT_CHECK;
use url::Url;

const DEFAULT_BANDWIDTH_CHECK: Duration = Duration::from_secs(5); // 5 seconds
const LOWER_BOUND_CHECK_DURATION: Duration = DEFAULT_PEER_TIMEOUT_CHECK;
const UPPER_BOUND_CHECK_DURATION: Duration =
    Duration::from_secs(6 * DEFAULT_PEER_TIMEOUT_CHECK.as_secs());
const DEFAULT_BANDWIDTH_DEPLETION_RATE: u64 = 1024 * 1024; // 1 MB/s
// If we consider a maximum supported connection speed of 1 Gbps, then we can consume 125 MB/s, which is 125 * 5 = 625 MB in 5 seconds, which is the minimum time between checks.
// So we set the minimum remaining bandwidth to 1000 MB > 625 MB, roughly the size of two tickets
const MINIMUM_RAMAINING_BANDWIDTH: u64 = 1000 * 1024 * 1024; // 1000 MB, the same as two wireguard ticket size

// 8 MB/s, in the worse case scenario with 30 seconds until the next check => 240 MB consumed, under the 1000 MB bandwidth minimum we have assured with the gateway
// We consider anything above this threshold to potentially consume bandwidth too fast, so we do a check immediately
const SYSTEM_BANDWIDTH_THRESHOLD: u64 = 8 * 1024 * 1024; // 8 MB
const SYSTEM_BANDWIDTH_CHECK_INTERVAL: Duration = Duration::from_secs(1); // 1 second

const DEFAULT_CLIENT_RETRIES: usize = 1;
const DEFAULT_CLIENT_TIMEOUT: Duration = Duration::from_secs(5); // 5 seconds

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

    #[error("No credential of type {ticketbook_type} available to send to gateway: {gateway_id}")]
    NoCredentialAvailable {
        ticketbook_type: TicketType,
        gateway_id: String,
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
        source: Box<MetadataClientError>,
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
                    "Using latest metadata client for {}'s bandwidth monitor",
                    gateway.identity()
                );
                TemporaryBandwidthClient::Latest(Box::new(metadata_client))
            } else {
                tracing::debug!(
                    "Using deprecated mixnet client for {}'s bandwidth monitor",
                    gateway.identity()
                );
                TemporaryBandwidthClient::Deprecated(Box::new(auth_client))
            }
        } else {
            tracing::debug!(
                "No authenticator client provided, using latest metadata client for {}'s bandwidth monitor",
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

    pub(crate) async fn lazy_init(&mut self) {
        match self {
            TemporaryBandwidthClient::Deprecated(_) => {}
            TemporaryBandwidthClient::Latest(metadata_client) => {
                metadata_client.lazy_init().await;
            }
        }
    }

    pub(crate) async fn interface_name(&mut self) -> Option<String> {
        match self {
            TemporaryBandwidthClient::Deprecated(_) => None,
            TemporaryBandwidthClient::Latest(metadata_client) => {
                metadata_client.interface_name().await
            }
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

struct InterfaceStats {
    interface_name: String,
    received: u64,
    transmitted: u64,
}

trait NetworkInterfaceStats {
    fn get_interface_stats(&mut self) -> Vec<InterfaceStats>;
}

impl NetworkInterfaceStats for Networks {
    fn get_interface_stats(&mut self) -> Vec<InterfaceStats> {
        Networks::refresh(self, true);
        Networks::list(self)
            .iter()
            .map(|(name, data)| InterfaceStats {
                interface_name: name.clone(),
                received: data.received(),
                transmitted: data.transmitted(),
            })
            .collect()
    }
}

// Structure used to track network usage by looking at the system stats
// It is not used as a source of truth for topping up bandwidth, but rather as a simpler
// way to detect potential bandwidth spikes and react to them faster then waiting for the
// next bandwidth check tick
struct SystemBandwidthMonitor<N = Networks> {
    networks: N,
    interface_name: Option<String>,
    threshold: u64,
}

impl SystemBandwidthMonitor {
    pub fn new(interface_name: Option<String>, threshold: u64) -> Self {
        Self {
            networks: Networks::new_with_refreshed_list(),
            interface_name,
            threshold,
        }
    }
}

impl<N: NetworkInterfaceStats> SystemBandwidthMonitor<N> {
    pub async fn force_bandwidth_checks(&mut self) -> bool {
        let total_network_usage: u64 = self
            .networks
            .get_interface_stats()
            .into_iter()
            .filter_map(
                // if interface name is specified, only consider that interface, otherwise consider all interfaces
                |InterfaceStats {
                     interface_name,
                     received,
                     transmitted,
                 }| match self.interface_name.as_ref() {
                    Some(target) => {
                        if &interface_name == target {
                            Some(received + transmitted)
                        } else {
                            None
                        }
                    }
                    None => Some(received + transmitted),
                },
            )
            .sum();
        // above the configured threshold, we consider the system to be under heavy load and thus we should attempt to top up bandwidth asap
        total_network_usage > self.threshold
    }
}

pub(crate) struct BandwidthMonitor {
    bc_command_tx: BandwidthControllerRequestSender,
    skew_manager: SkewManager,
    wg_entry_gateway_client: TemporaryBandwidthClient,
    wg_exit_gateway_client: TemporaryBandwidthClient,
    timeout_check_interval: IntervalStream,
    entry_depletion_rate: DepletionRate,
    exit_depletion_rate: DepletionRate,
    entry_previous_error_query: bool,
    exit_previous_error_query: bool,
    shutdown_token: CancellationToken,
    successful_checks: u64,
    upgrade_mode_enabled_on_last_check: bool,
    metadata_path_health: Option<MetadataPathHealth>,
}

impl BandwidthMonitor {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        bc_command_tx: BandwidthControllerRequestSender,
        skew_manager: SkewManager,
        wg_entry_gateway_client: TemporaryBandwidthClient,
        wg_exit_gateway_client: TemporaryBandwidthClient,
        shutdown_token: CancellationToken,
        metadata_path_health: Option<MetadataPathHealth>,
    ) -> Self {
        let timeout_check_interval =
            IntervalStream::new(tokio::time::interval(DEFAULT_BANDWIDTH_CHECK));

        BandwidthMonitor {
            bc_command_tx,
            skew_manager,
            wg_entry_gateway_client,
            wg_exit_gateway_client,
            timeout_check_interval,
            entry_depletion_rate: Default::default(),
            exit_depletion_rate: Default::default(),
            entry_previous_error_query: false,
            exit_previous_error_query: false,
            shutdown_token,
            successful_checks: 0,
            upgrade_mode_enabled_on_last_check: false,
            metadata_path_health,
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
            DEFAULT_CLIENT_TIMEOUT,
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
        self.bc_command_tx
            .get_upgrade_mode_token()
            .await
            .inspect_err(|err| {
                trace_err_chain!(
                    err,
                    "critical failure: failed to resolve account controller query"
                )
            })
            .is_ok_and(|maybe_token| maybe_token.is_some())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn create(
        bc_command_tx: BandwidthControllerRequestSender,
        skew_manager: SkewManager,
        selected_gateways: &SelectedGateways,
        entry_auth_client: Option<AuthenticatorClient>,
        exit_auth_client: Option<AuthenticatorClient>,
        entry_wireguard_config: &WireguardConfiguration,
        exit_wireguard_config: &WireguardConfiguration,
        entry_signal_channel: TunUpReceiver,
        exit_signal_channel: TunUpReceiver,
        gateway_metadata_update_version: Option<semver::Version>,
        cancel_token: CancellationToken,
        metadata_path_health: MetadataPathHealth,
    ) -> BandwidthMonitor {
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
            bc_command_tx,
            skew_manager,
            wg_entry_client,
            wg_exit_client,
            cancel_token.clone(),
            Some(metadata_path_health),
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

        let spend_time = self.skew_manager.cached_skew_corrected_time();

        let credential = self
            .bc_command_tx
            .get_ecash_ticket(
                ticketbook_type,
                bw_client.gateway_id(),
                DEFAULT_TICKETS_TO_SPEND,
                spend_time,
            )
            .await
            .map_err(|source| SpecificGatewayError::RequestCredential {
                gateway_id: bw_client.gateway_id().to_string(),
                ticketbook_type,
                source: Box::new(source),
            })?
            .ok_or(SpecificGatewayError::NoCredentialAvailable {
                ticketbook_type,
                gateway_id: bw_client.gateway_id().to_string(),
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

        let Some(upgrade_mode_jwt) =
            self.bc_command_tx
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
        trace_err_chain!(err, "query error");
        let gateway_id = self.gateway_id(entry);
        if (entry && self.entry_previous_error_query) || (!entry && self.exit_previous_error_query)
        {
            // For now let's keep the old behavior of stopping, but only if we've had a successful check before
            if self.successful_checks != 0 {
                tracing::error!("Gateway {gateway_id} is erroring out; shutting-down the tunnel");
                self.shutdown_token.cancel();
            } else {
                tracing::warn!("Gateway {gateway_id} is erroring out");
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
                trace_err_chain!(e, "error while updating query coefficients");
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
                tracing::warn!(
                    "gateway {gateway_id} is outdated and does not support upgrade mode queries"
                )
            }
            Some(true) => {
                // gateway informed us it is currently in upgrade mode - we don't have to do anything
                tracing::debug!(
                    "gateway {gateway_id} is already in upgrade mode - no need to perform bandwidth top up"
                );
                if !got_um_data {
                    // account controller is not aware of the UM - it didn't have to request bandwidth
                    // from VPN API and thus hasn't received UM JWT
                    // this can happen for clients with a lot of stored ticketbooks. there's nothing
                    // inherently wrong with it
                    tracing::debug!(
                        "however, we do not possess a corresponding upgrade mode attestation"
                    );
                }
                self.upgrade_mode_enabled_on_last_check = true;
                return None;
            }
            Some(false) => {
                if got_um_data && !self.upgrade_mode_enabled_on_last_check {
                    // if we have relevant attestation and the gateway is not in upgrade mode,
                    // we need to trigger it to perform internal state refresh
                    tracing::info!(
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
                        trace_err_chain!(err, "error requesting upgrade mode recheck");
                    }
                    // since we didn't manage to trigger gateway to go into the upgrade mode,
                    // if we want to continue the connection we have to attempt to send a zk-nym instead
                    // (so we exit the match statement)
                } else if got_um_data && self.upgrade_mode_enabled_on_last_check {
                    // UM is over - we inform BC and top up bandwidth as normal
                    tracing::info!(
                        "gateway {gateway_id} has informed us the upgrade mode has finished"
                    );
                    if let Err(err) = self.bc_command_tx.clear_emergency_credentials().await {
                        trace_err_chain!(err, "error sending message to the bandwidth controller");
                    }
                    // we continue sending zk-nym
                } else {
                    // if we got here it means we don't have any attestation data and gateway said
                    // it's not in upgrade mode, meaning it's business as usual
                    // so continue and attempt to top-up bandwidth with a zk-nym
                    tracing::trace!("upgrade mode is not enabled anywhere in the system");
                }
            }
        }

        if let Err(e) = self.top_up_bandwidth(entry).await {
            trace_err_chain!(e, "error topping up with more bandwidth");
            // TODO: try to return this error in the JoinHandle instead
            // For now let's keep the old behavior of stopping
            self.shutdown_token.cancel();
        }

        None
    }

    async fn check_bandwidth(
        &mut self,
        entry: bool,
        current_period: Duration,
    ) -> (Option<Duration>, bool) {
        let bw_client = if entry {
            &mut self.wg_entry_gateway_client
        } else {
            &mut self.wg_exit_gateway_client
        };
        tokio::select! {
            _ = self.shutdown_token.cancelled() => {
                tracing::trace!("BandwidthMonitor: Received shutdown");
            }
            ret = bw_client.query_bandwidth_with_retries(DEFAULT_CLIENT_RETRIES) => {
                return match ret {
                    Ok(query_res) => {
                        let next_interval = self
                            .handle_bandwidth_query(entry, current_period, query_res)
                            .await;
                        (next_interval, true)
                    }
                    Err(err) => {
                        self.handle_bandwidth_query_error(entry, err).await;
                        (None, false)
                    }
                };
            }
        }
        (None, false)
    }

    async fn init_clients(&mut self) {
        tokio::join!(
            self.wg_entry_gateway_client.lazy_init(),
            self.wg_exit_gateway_client.lazy_init()
        );
    }

    pub(crate) async fn run(mut self) {
        // Make sure the clients are up and ready, or notifying early of any problems
        if self
            .shutdown_token
            .clone()
            .run_until_cancelled(self.init_clients())
            .await
            .is_none()
        {
            tracing::debug!("BandwidthMonitor: Exiting");
            return;
        }

        let exit_interface_name = self.wg_exit_gateway_client.interface_name().await;
        let mut system_bandwidth_monitor =
            SystemBandwidthMonitor::new(exit_interface_name, SYSTEM_BANDWIDTH_THRESHOLD);
        let mut system_bandwidth_check_interval =
            IntervalStream::new(tokio::time::interval(SYSTEM_BANDWIDTH_CHECK_INTERVAL));

        // Skip the first, immediate tick
        self.timeout_check_interval.next().await;
        while !self.shutdown_token.is_cancelled() {
            tokio::select! {
                _ = self.shutdown_token.cancelled() => {
                    tracing::trace!("BandwidthMonitor: Received shutdown");
                    break;
                }
                _ = system_bandwidth_check_interval.next() => {
                    if system_bandwidth_monitor.force_bandwidth_checks().await && self.timeout_check_interval.as_ref().period() > LOWER_BOUND_CHECK_DURATION {
                        tracing::debug!("System bandwidth usage is above the configured threshold, forcing bandwidth checks");
                        // we set the check interval to the lower bound, but we don't do the first tick, which is immediate,
                        // so we implicitely trigger the check branch bellow
                        self.timeout_check_interval = IntervalStream::new(tokio::time::interval(LOWER_BOUND_CHECK_DURATION));
                    }
                }
                _ = self.timeout_check_interval.next() => {
                    let current_period = self.timeout_check_interval.as_ref().period();
                    let (entry_duration, entry_query_ok) =
                        self.check_bandwidth(true, current_period).await;
                    let (exit_duration, exit_query_ok) =
                        self.check_bandwidth(false, current_period).await;
                    update_metadata_path_health(
                        &self.metadata_path_health,
                        entry_query_ok,
                        exit_query_ok,
                    );
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

        tracing::debug!("BandwidthMonitor: Exiting");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    const BW_1KB: u64 = 1024;
    const BW_1MB: u64 = 1024 * BW_1KB;
    const BW_128MB: u64 = 128 * BW_1MB;
    const BW_512MB: u64 = 512 * BW_1MB;
    const BW_1GB: u64 = 2 * BW_512MB;

    struct MockNetworks {
        data: HashMap<String, (u64, u64)>,
        refresh_count: u32,
    }

    impl MockNetworks {
        fn new() -> Self {
            Self {
                data: HashMap::new(),
                refresh_count: 0,
            }
        }

        fn with_interface(mut self, name: &str, received: u64, transmitted: u64) -> Self {
            self.data.insert(name.to_string(), (received, transmitted));
            self
        }
    }

    impl NetworkInterfaceStats for MockNetworks {
        fn get_interface_stats(&mut self) -> Vec<InterfaceStats> {
            self.refresh_count += 1;
            self.data
                .iter()
                .map(|(name, &(received, transmitted))| InterfaceStats {
                    interface_name: name.clone(),
                    received,
                    transmitted,
                })
                .collect()
        }
    }

    impl<N: NetworkInterfaceStats> SystemBandwidthMonitor<N> {
        fn with_networks(interface_name: Option<String>, threshold: u64, networks: N) -> Self {
            Self {
                networks,
                interface_name,
                threshold,
            }
        }
    }

    #[test]
    fn depletion_rate_slow() {
        let mut depletion_rate = DepletionRate::default();
        let mut current_period = DEFAULT_BANDWIDTH_CHECK;
        // the first check would force the placeholder values to be replaced by the actual values
        assert_eq!(
            depletion_rate
                .update_dynamic_check_interval(current_period, BW_1GB)
                .unwrap(),
            Some(DEFAULT_BANDWIDTH_CHECK)
        );

        // simulate 1 byte/second depletion rate
        let consumed = current_period.as_secs();
        current_period = depletion_rate
            .update_dynamic_check_interval(current_period, BW_1GB - consumed)
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
        let mut current_bandwidth = 2 * BW_1GB;
        // the first check would force the placeholder values to be replaced by the actual values
        assert_eq!(
            depletion_rate
                .update_dynamic_check_interval(current_period, current_bandwidth)
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
        for _ in 0..34 {
            current_bandwidth -= current_period.as_secs() * BW_1MB;
            current_period = depletion_rate
                .update_dynamic_check_interval(current_period, current_bandwidth)
                .unwrap()
                .unwrap();
            assert_eq!(current_period, UPPER_BOUND_CHECK_DURATION);
            assert!(current_bandwidth > 1000 * BW_1MB);
        }

        current_bandwidth -= current_period.as_secs() * BW_1MB;
        let ret = depletion_rate
            .update_dynamic_check_interval(current_period, current_bandwidth)
            .unwrap();
        // when we get bellow a convinient dynamic threshold, we start reqwesting more bandwidth (returning None)
        assert!(current_bandwidth < 1000 * BW_1MB);
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

    #[tokio::test]
    async fn system_bandwidth_monitor_no_interfaces_is_below_threshold() {
        let mock = MockNetworks::new();
        let mut monitor =
            SystemBandwidthMonitor::with_networks(None, SYSTEM_BANDWIDTH_THRESHOLD, mock);
        assert!(!monitor.force_bandwidth_checks().await);
    }

    #[tokio::test]
    async fn system_bandwidth_monitor_calls_refresh_on_each_check() {
        let mock = MockNetworks::new();
        let mut monitor =
            SystemBandwidthMonitor::with_networks(None, SYSTEM_BANDWIDTH_THRESHOLD, mock);
        monitor.force_bandwidth_checks().await;
        assert_eq!(monitor.networks.refresh_count, 1);
        monitor.force_bandwidth_checks().await;
        assert_eq!(monitor.networks.refresh_count, 2);
    }

    #[tokio::test]
    async fn system_bandwidth_monitor_below_threshold_returns_false() {
        // 2 MB combined < 8 MB threshold
        let mock = MockNetworks::new().with_interface("eth0", BW_1MB, BW_1MB);
        let mut monitor =
            SystemBandwidthMonitor::with_networks(None, SYSTEM_BANDWIDTH_THRESHOLD, mock);
        assert!(!monitor.force_bandwidth_checks().await);
    }

    #[tokio::test]
    async fn system_bandwidth_monitor_above_threshold_returns_true() {
        // 10 MB combined > 8 MB threshold
        let mock = MockNetworks::new().with_interface("eth0", 5 * BW_1MB, 5 * BW_1MB);
        let mut monitor =
            SystemBandwidthMonitor::with_networks(None, SYSTEM_BANDWIDTH_THRESHOLD, mock);
        assert!(monitor.force_bandwidth_checks().await);
    }

    #[tokio::test]
    async fn system_bandwidth_monitor_exactly_at_threshold_returns_false() {
        // usage must be strictly greater than threshold, not equal
        let mock = MockNetworks::new().with_interface(
            "eth0",
            SYSTEM_BANDWIDTH_THRESHOLD / 2,
            SYSTEM_BANDWIDTH_THRESHOLD / 2,
        );
        let mut monitor =
            SystemBandwidthMonitor::with_networks(None, SYSTEM_BANDWIDTH_THRESHOLD, mock);
        assert!(!monitor.force_bandwidth_checks().await);
    }

    #[tokio::test]
    async fn system_bandwidth_monitor_sums_all_interfaces_without_filter() {
        // 3 MB per interface × 3 interfaces = 9 MB total > 8 MB threshold
        let mock = MockNetworks::new()
            .with_interface("eth0", 3 * BW_1MB, 0)
            .with_interface("eth1", 3 * BW_1MB, 0)
            .with_interface("wlan0", 3 * BW_1MB, 0);
        let mut monitor =
            SystemBandwidthMonitor::with_networks(None, SYSTEM_BANDWIDTH_THRESHOLD, mock);
        assert!(monitor.force_bandwidth_checks().await);
    }

    #[tokio::test]
    async fn system_bandwidth_monitor_filters_to_target_interface_below_threshold() {
        // eth0 has 10 MB (above threshold), but we only watch wg0 which has 2 MB
        let mock = MockNetworks::new()
            .with_interface("eth0", 5 * BW_1MB, 5 * BW_1MB)
            .with_interface("wg0", BW_1MB, BW_1MB);
        let mut monitor = SystemBandwidthMonitor::with_networks(
            Some("wg0".to_string()),
            SYSTEM_BANDWIDTH_THRESHOLD,
            mock,
        );
        assert!(!monitor.force_bandwidth_checks().await);
    }

    #[tokio::test]
    async fn system_bandwidth_monitor_filters_to_target_interface_above_threshold() {
        // eth0 has 2 MB (below threshold), but we only watch wg0 which has 10 MB
        let mock = MockNetworks::new()
            .with_interface("eth0", BW_1MB, BW_1MB)
            .with_interface("wg0", 5 * BW_1MB, 5 * BW_1MB);
        let mut monitor = SystemBandwidthMonitor::with_networks(
            Some("wg0".to_string()),
            SYSTEM_BANDWIDTH_THRESHOLD,
            mock,
        );
        assert!(monitor.force_bandwidth_checks().await);
    }

    #[tokio::test]
    async fn system_bandwidth_monitor_nonexistent_target_interface_returns_false() {
        // eth0 has 10 MB but we watch a non-existent interface, so total is 0
        let mock = MockNetworks::new().with_interface("eth0", 5 * BW_1MB, 5 * BW_1MB);
        let mut monitor = SystemBandwidthMonitor::with_networks(
            Some("nonexistent".to_string()),
            SYSTEM_BANDWIDTH_THRESHOLD,
            mock,
        );
        assert!(!monitor.force_bandwidth_checks().await);
    }
}
