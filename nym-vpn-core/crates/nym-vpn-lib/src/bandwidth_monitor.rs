// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{cmp::max, net::IpAddr, time::Duration};

use nym_authenticator_client::AuthenticatorClient;

use nym_bandwidth_controller::{
    DEFAULT_TICKETS_TO_SPEND, requests::BandwidthControllerRequestSender,
};
use nym_network_defaults::TicketTypeRepr;
use nym_registration_common::WireguardConfiguration;
use nym_vpn_api_client::SkewManager;
use sysinfo::Networks;
use tokio_stream::{StreamExt, wrappers::IntervalStream};
use tokio_util::sync::CancellationToken;

use nym_config::defaults::{WG_METADATA_PORT, WG_TUN_DEVICE_IP_ADDRESS_V4};
use nym_credentials_interface::TicketType;
use nym_gateway_directory::Gateway;

use crate::tunnel_state_machine::tunnel::{
    SelectedGateways,
    wireguard::two_hop_config::{EXIT_MTU, WG_TUNNEL_OVERHEAD},
};
use nym_common::trace_err_chain;
use nym_wg_metadata_client::{MetadataClient, TunUpReceiver, error::MetadataClientError};
use nym_wireguard_types::DEFAULT_PEER_TIMEOUT_CHECK;
use url::Url;

const MINIMUM_REMAINING_BANDWIDTH: u64 = 2500 * 1024 * 1024; // 2500 MB, the same as five wireguard ticket size
const SYSTEM_BANDWIDTH_CHECK_INTERVAL: Duration = Duration::from_secs(15); // 15 seconds
const REMOTE_BANDWIDTH_CHECK_INTERVAL: Duration =
    Duration::from_secs(12 * DEFAULT_PEER_TIMEOUT_CHECK.as_secs()); // 1 minute

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
    entry_interface_name: Option<String>,
    exit_interface_name: Option<String>,
    entry_approx_bandwidth_left: u64,
    exit_approx_bandwidth_left: u64,
    target_bandwidth: u64,
    min_threshold: u64,
}

impl SystemBandwidthMonitor {
    pub fn new(
        entry_interface_name: Option<String>,
        exit_interface_name: Option<String>,
        entry_approx_bandwidth_left: u64,
        exit_approx_bandwidth_left: u64,
        target_bandwidth: u64,
        min_threshold: u64,
    ) -> Self {
        Self {
            networks: Networks::new_with_refreshed_list(),
            entry_interface_name,
            exit_interface_name,
            entry_approx_bandwidth_left,
            exit_approx_bandwidth_left,
            target_bandwidth,
            min_threshold,
        }
    }

    pub fn update_entry_approximation(&mut self, latest: i64) {
        self.entry_approx_bandwidth_left = if latest < 0 {
            0
        } else {
            latest.cast_unsigned()
        };
    }

    pub fn update_exit_approximation(&mut self, latest: i64) {
        self.exit_approx_bandwidth_left = if latest < 0 {
            0
        } else {
            latest.cast_unsigned()
        };
    }
}

impl<N: NetworkInterfaceStats> SystemBandwidthMonitor<N> {
    pub fn entry_meets_target(&self) -> bool {
        self.entry_approx_bandwidth_left <= self.target_bandwidth
    }

    pub fn exit_meets_target(&self) -> bool {
        self.exit_approx_bandwidth_left <= self.target_bandwidth
    }

    pub async fn force_remote_bandwidth_query(&mut self) -> bool {
        let stats = self.networks.get_interface_stats();

        let Some(exit_interface_usage) = consumption_for(&stats, &self.exit_interface_name) else {
            tracing::warn!(
                "No exit interface name or the interface name was not found: {:?}",
                self.exit_interface_name
            );
            return false;
        };
        let packet_count = exit_interface_usage / EXIT_MTU as u64;
        let entry_interface_usage_fallback =
            exit_interface_usage + packet_count * WG_TUNNEL_OVERHEAD as u64;
        let entry_interface_usage = consumption_for(&stats, &self.entry_interface_name)
            .unwrap_or(entry_interface_usage_fallback);

        self.entry_approx_bandwidth_left = self
            .entry_approx_bandwidth_left
            .saturating_sub(entry_interface_usage);
        self.exit_approx_bandwidth_left = self
            .exit_approx_bandwidth_left
            .saturating_sub(exit_interface_usage);

        // force query only if really necessary i.e. bellow ticket size
        self.entry_approx_bandwidth_left < self.min_threshold
            || self.exit_approx_bandwidth_left < self.min_threshold
    }
}

fn consumption_for(
    stats: &[InterfaceStats],
    target_interface_name: &Option<String>,
) -> Option<u64> {
    target_interface_name
        .as_ref()
        .and_then(|target_interface_name| {
            stats.iter().find_map(
                |InterfaceStats {
                     interface_name,
                     received,
                     transmitted,
                 }| {
                    if interface_name == target_interface_name {
                        Some(received + transmitted)
                    } else {
                        None
                    }
                },
            )
        })
}

pub(crate) struct BandwidthMonitor {
    bc_command_tx: BandwidthControllerRequestSender,
    skew_manager: SkewManager,
    wg_entry_gateway_client: TemporaryBandwidthClient,
    wg_exit_gateway_client: TemporaryBandwidthClient,
    timeout_check_interval: IntervalStream,
    entry_previous_error_query: bool,
    exit_previous_error_query: bool,
    shutdown_token: CancellationToken,
    successful_checks: u64,
    upgrade_mode_enabled_on_last_check: bool,
}

impl BandwidthMonitor {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        bc_command_tx: BandwidthControllerRequestSender,
        skew_manager: SkewManager,
        wg_entry_gateway_client: TemporaryBandwidthClient,
        wg_exit_gateway_client: TemporaryBandwidthClient,
        shutdown_token: CancellationToken,
    ) -> Self {
        let timeout_check_interval =
            IntervalStream::new(tokio::time::interval(REMOTE_BANDWIDTH_CHECK_INTERVAL));

        BandwidthMonitor {
            bc_command_tx,
            skew_manager,
            wg_entry_gateway_client,
            wg_exit_gateway_client,
            timeout_check_interval,
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
        system_bandwidth_monitor: &mut SystemBandwidthMonitor,
        query_result: AvailableBandwidth,
    ) {
        let gw_upgrade_mode = query_result.upgrade_mode;
        let gateway_id = self.gateway_id(entry);

        self.successful_checks += 1;

        let needs_top_up = if entry {
            self.entry_previous_error_query = false;
            system_bandwidth_monitor.update_entry_approximation(query_result.bandwidth_bytes);
            system_bandwidth_monitor.entry_meets_target()
        } else {
            self.exit_previous_error_query = false;
            system_bandwidth_monitor.update_exit_approximation(query_result.bandwidth_bytes);
            system_bandwidth_monitor.exit_meets_target()
        };

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
                return;
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

        if needs_top_up {
            match self.top_up_bandwidth(entry).await {
                Ok(available_bandwidth) => {
                    if entry {
                        system_bandwidth_monitor
                            .update_entry_approximation(available_bandwidth.bandwidth_bytes);
                    } else {
                        system_bandwidth_monitor
                            .update_exit_approximation(available_bandwidth.bandwidth_bytes);
                    }
                }
                Err(e) => {
                    trace_err_chain!(e, "error topping up with more bandwidth");
                    // TODO: try to return this error in the JoinHandle instead
                    // For now let's keep the old behavior of stopping
                    self.shutdown_token.cancel();
                }
            }
        }
    }

    async fn check_bandwidth(
        &mut self,
        entry: bool,
        system_bandwidth_monitor: &mut SystemBandwidthMonitor,
    ) {
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
                match ret {
                    Ok(query_res) => {
                        self
                            .handle_bandwidth_query(entry, system_bandwidth_monitor, query_res)
                            .await
                    }
                    Err(err) => self.handle_bandwidth_query_error(entry, err).await,
                }
            }
        }
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

        let entry_interface_name = self.wg_entry_gateway_client.interface_name().await;
        let exit_interface_name = self.wg_exit_gateway_client.interface_name().await;
        let mut system_bandwidth_monitor = SystemBandwidthMonitor::new(
            entry_interface_name,
            exit_interface_name,
            TicketTypeRepr::WIREGUARD_ENTRY_TICKET_SIZE,
            TicketTypeRepr::WIREGUARD_EXIT_TICKET_SIZE,
            MINIMUM_REMAINING_BANDWIDTH,
            max(
                TicketTypeRepr::WIREGUARD_ENTRY_TICKET_SIZE,
                TicketTypeRepr::WIREGUARD_EXIT_TICKET_SIZE,
            ),
        );
        let mut system_bandwidth_check_interval =
            IntervalStream::new(tokio::time::interval(SYSTEM_BANDWIDTH_CHECK_INTERVAL));
        let mut system_triggered_check = false;

        while !self.shutdown_token.is_cancelled() {
            tokio::select! {
                _ = self.shutdown_token.cancelled() => {
                    tracing::trace!("BandwidthMonitor: Received shutdown");
                    break;
                }
                // local, cheap counting of consumed bytes
                _ = system_bandwidth_check_interval.next() => {
                    if system_bandwidth_monitor.force_remote_bandwidth_query().await {
                        // we only force a bandwidth check once. If for some reason, it doesn't work, we rely on the normal timer to retry
                        if !system_triggered_check {
                            tracing::debug!("Remaining bandwidth approximation is bellow the configured threshold, forcing bandwidth checks");
                            system_triggered_check = true;
                            self.check_bandwidth(true, &mut system_bandwidth_monitor).await;
                            self.check_bandwidth(false, &mut system_bandwidth_monitor).await;
                        }
                    } else {
                        system_triggered_check = false;
                    }
                }
                // network refresh, more expensive, especially for mobile battery consumption
                _ = self.timeout_check_interval.next() => {
                    self.check_bandwidth(true, &mut system_bandwidth_monitor).await;
                    self.check_bandwidth(false, &mut system_bandwidth_monitor).await;
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
        fn with_networks(
            entry_interface_name: Option<String>,
            exit_interface_name: Option<String>,
            target_bandwidth: u64,
            networks: N,
        ) -> Self {
            Self {
                networks,
                entry_interface_name,
                exit_interface_name,
                target_bandwidth,
                entry_approx_bandwidth_left: 6 * TicketTypeRepr::WIREGUARD_ENTRY_TICKET_SIZE,
                exit_approx_bandwidth_left: 6 * TicketTypeRepr::WIREGUARD_EXIT_TICKET_SIZE,
                min_threshold: max(
                    TicketTypeRepr::WIREGUARD_ENTRY_TICKET_SIZE,
                    TicketTypeRepr::WIREGUARD_EXIT_TICKET_SIZE,
                ),
            }
        }
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
            SystemBandwidthMonitor::with_networks(None, None, MINIMUM_REMAINING_BANDWIDTH, mock);
        assert!(!monitor.force_remote_bandwidth_query().await);
    }

    #[tokio::test]
    async fn system_bandwidth_monitor_calls_refresh_on_each_check() {
        let mock = MockNetworks::new();
        let mut monitor =
            SystemBandwidthMonitor::with_networks(None, None, MINIMUM_REMAINING_BANDWIDTH, mock);
        monitor.force_remote_bandwidth_query().await;
        assert_eq!(monitor.networks.refresh_count, 1);
        monitor.force_remote_bandwidth_query().await;
        assert_eq!(monitor.networks.refresh_count, 2);
    }

    #[tokio::test]
    async fn system_bandwidth_monitor_below_threshold_returns_false() {
        // 256 MB combined < 2500 MB threshold
        let mock = MockNetworks::new().with_interface("eth0", BW_128MB, BW_128MB);
        let mut monitor =
            SystemBandwidthMonitor::with_networks(None, None, MINIMUM_REMAINING_BANDWIDTH, mock);
        assert!(!monitor.force_remote_bandwidth_query().await);
    }

    #[tokio::test]
    async fn system_bandwidth_monitor_above_threshold_returns_true() {
        // 4 GB combined > 2.5 GB threshold
        let mock = MockNetworks::new().with_interface("wg0", 2 * BW_1GB, 5 * BW_1GB);
        let mut monitor = SystemBandwidthMonitor::with_networks(
            None,
            Some("wg0".to_string()),
            MINIMUM_REMAINING_BANDWIDTH,
            mock,
        );
        assert!(monitor.force_remote_bandwidth_query().await);
    }

    #[tokio::test]
    async fn system_bandwidth_monitor_exactly_at_threshold_returns_false() {
        // usage must be greater or equal to the threshold
        let mock = MockNetworks::new().with_interface(
            "wg0",
            MINIMUM_REMAINING_BANDWIDTH / 2,
            MINIMUM_REMAINING_BANDWIDTH / 2,
        );
        let mut monitor = SystemBandwidthMonitor::with_networks(
            None,
            Some("wg0".to_string()),
            MINIMUM_REMAINING_BANDWIDTH,
            mock,
        );
        assert!(monitor.force_remote_bandwidth_query().await);
    }

    #[tokio::test]
    async fn system_bandwidth_monitor_filters_to_target_interface_below_threshold() {
        // eth0 has 4 GB (above threshold), but we only watch wg0 and wg2 which have 4 MB
        let mock = MockNetworks::new()
            .with_interface("eth0", 2 * BW_1GB, 2 * BW_1GB)
            .with_interface("wg0", BW_1MB, BW_1MB)
            .with_interface("wg1", BW_1MB, BW_1MB);
        let mut monitor = SystemBandwidthMonitor::with_networks(
            Some("wg0".to_string()),
            Some("wg1".to_string()),
            MINIMUM_REMAINING_BANDWIDTH,
            mock,
        );
        assert!(!monitor.force_remote_bandwidth_query().await);
    }

    #[tokio::test]
    async fn system_bandwidth_monitor_filters_to_target_interface_above_threshold() {
        // eth0 has 2 MB (below threshold), but we only watch wg0 which has 4 GB
        let mock = MockNetworks::new()
            .with_interface("eth0", BW_1MB, BW_1MB)
            .with_interface("wg0", 2 * BW_1GB, 2 * BW_1GB);
        let mut monitor = SystemBandwidthMonitor::with_networks(
            None,
            Some("wg0".to_string()),
            MINIMUM_REMAINING_BANDWIDTH,
            mock,
        );
        assert!(monitor.force_remote_bandwidth_query().await);
    }

    #[tokio::test]
    async fn system_bandwidth_monitor_nonexistent_target_interface_returns_fase() {
        // eth0 has 4 GB but we watch a non-existent interface, so value used is maximum
        let mock = MockNetworks::new().with_interface("eth0", 2 * BW_1GB, 2 * BW_1GB);
        let mut monitor = SystemBandwidthMonitor::with_networks(
            Some("nonexistent".to_string()),
            Some("nonexistent2".to_string()),
            MINIMUM_REMAINING_BANDWIDTH,
            mock,
        );
        assert!(!monitor.force_remote_bandwidth_query().await);
    }
}
