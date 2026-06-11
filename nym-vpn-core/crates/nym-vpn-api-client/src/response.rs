// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    collections::HashSet,
    fmt,
    net::{IpAddr, SocketAddr},
};

use crate::{error::VpnApiClientError, network_compatibility::NetworkCompatibility};
use itertools::Itertools;
use nym_contracts_common::Percent;
use nym_credential_proxy_requests::api::v1::ticketbook::models::TicketbookWalletSharesResponse;
pub use nym_credential_proxy_requests::api::v1::ticketbook::models::UpgradeModeAttestation;
use nym_validator_client::models::described::type_translation::LewesProtocolDetailsV1;
use nym_network_defaults::network::NetworkingSpecifics;
use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, UtcDateTime, format_description::well_known::Iso8601};

const MAX_PROBE_RESULT_AGE_MINUTES: i64 = 60;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NymVpnRegisterAccountResponse {
    pub created_on_utc: String,
    pub last_updated_utc: String,
    pub account_addr: String,
    pub status: NymVpnRegisterAccountStatusResponse,
    pub account_token: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NymVpnRegisterAccountStatusResponse {
    Active,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NymVpnAccountResponse {
    pub created_on_utc: String,
    pub last_updated_utc: String,
    pub account_addr: String,
    pub status: NymVpnAccountStatusResponse,
    pub canonical_account_addr: Option<String>,
    pub auth_methods: Vec<NymVpnAccountAuthMethodResponse>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, strum_macros::Display)]
#[serde(rename_all = "snake_case")]
pub enum NymVpnAccountStatusResponse {
    Active,
    Inactive,
    DeleteMe,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct NymVpnAccountAuthMethodResponse {
    pub id: String,
    pub pubkey: String,
    pub kind: String,
    pub label: String,
    pub status: NymVpnAccountStatusResponse,
    pub created: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NymVpnAccountSummaryResponse {
    pub account: NymVpnAccountResponse,
    pub subscription: NymVpnAccountSummarySubscription,
    pub devices: NymVpnAccountSummaryDevices,
    pub fair_usage: NymVpnAccountSummaryFairUsage,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NymVpnAccountSummaryWithDeviceResponse {
    #[serde(flatten)]
    pub account_summary: NymVpnAccountSummaryResponse,
    pub active_device: Option<NymVpnDevice>,
}

impl NymVpnAccountSummaryWithDeviceResponse {
    pub fn account_active(&self) -> bool {
        self.account_summary.account.status == NymVpnAccountStatusResponse::Active
    }

    pub fn subscription_active(&self) -> bool {
        self.account_summary.subscription.is_active
    }

    pub fn subscription_pending(&self) -> bool {
        !self.account_summary.subscription.is_active
            && self.account_summary.subscription.pending.is_some()
    }

    pub fn bandwidth_limit(&self) -> u64 {
        self.account_summary.fair_usage.limitGB
    }

    pub fn used_bandwidth(&self) -> u64 {
        self.account_summary.fair_usage.usedGB
    }

    pub fn remaining_devices(&self) -> u64 {
        self.account_summary.devices.remaining
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NymVpnAccountSummarySubscription {
    pub is_active: bool,
    pub active: Option<NymVpnSubscription>,
    pub pending: Option<NymVpnSubscription>,
    #[serde(default)]
    pub is_stacked: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NymVpnAccountSummaryDevices {
    pub active: u64,
    pub max: u64,
    pub remaining: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[allow(non_snake_case)]
// These fields have the substring 'GB' in them, meaning we can't use `rename_all = "camelCase"`
// like for the other structs
pub struct NymVpnAccountSummaryFairUsage {
    pub usedGB: u64,
    pub limitGB: u64,
    pub resetsOnUtc: Option<String>,
    // Absent in older API responses - treat as false (data available) during rollout.
    #[serde(default, rename = "dataUnavailable")]
    pub data_unavailable: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NymVpnCanonicalAccountIdentityResponse {
    pub canonical_account_addr: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NymVpnHealthResponse {
    pub status: String,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp_utc: OffsetDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NymVpnDevice {
    pub created_on_utc: String,
    pub last_updated_utc: String,
    pub device_identity_key: String,
    pub status: NymVpnDeviceStatus,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NymVpnDeviceStatus {
    Active,
    Inactive,
    DeleteMe,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NymVpnDevicesResponse {
    pub total_items: u64,
    pub page: u64,
    pub page_size: u64,
    pub items: Vec<NymVpnDevice>,
}

impl fmt::Display for NymVpnDevicesResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.items
                .iter()
                .format_with(", ", |item, f| f(&format_args!("{item:?}")))
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NymVpnRefundsResponse {
    pub total_items: u64,
    pub page: u64,
    pub page_size: u64,
    pub items: Vec<NymVpnRefund>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NymVpnRefund {
    pub created_on_utc: String,
    pub last_updated_utc: String,
    pub subscription_invoice: String,
    pub status: NymVpnRefundStatus,
    pub user_reason: NymVpnRefundUserReason,
    pub data: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NymVpnRefundStatus {
    Pending,
    Complete,
    Rejected,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NymVpnRefundUserReason {
    SubscriptionInError,
    PoorPerformance,
    Other,
}

// Legacy type, because the blinded_shares response for the POST seems to be different than the GET
// Remove once it's not needed anymore
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NymVpnZkNymPost {
    pub created_on_utc: String,
    pub last_updated_utc: String,
    pub id: String,
    pub ticketbook_type: String,
    pub valid_until_utc: String,
    pub valid_from_utc: String,
    pub issued_bandwidth_in_gb: f64,
    pub blinded_shares: Option<Vec<Option<TicketbookWalletSharesResponse>>>,
    pub status: NymVpnZkNymStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpgradeModeResponseData {
    pub upgrade_mode_attestation: UpgradeModeAttestation,
    pub upgrade_mode_jwt: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NymVpnZkNym {
    pub created_on_utc: String,
    pub last_updated_utc: String,
    pub id: String,
    pub ticketbook_type: String,
    pub valid_until_utc: String,
    pub valid_from_utc: String,
    pub issued_bandwidth_in_gb: f64,
    pub blinded_shares: Option<TicketbookWalletSharesResponse>,
    pub status: NymVpnZkNymStatus,
    pub upgrade_mode: Option<UpgradeModeResponseData>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, strum::Display)]
#[serde(rename_all = "snake_case")]
pub enum NymVpnZkNymStatus {
    Pending,
    Active,
    Revoking,
    Revoked,
    Error,
    UpgradeMode,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NymVpnZkNymResponse {
    pub total_items: u64,
    pub page: u64,
    pub page_size: u64,
    pub items: Vec<NymVpnZkNym>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NymVpnSubscription {
    pub created_on_utc: String,
    pub last_updated_utc: String,
    pub id: String,
    pub valid_until_utc: String,
    pub valid_from_utc: String,
    pub status: NymVpnSubscriptionStatus,
    pub kind: NymVpnSubscriptionKind,
    #[serde(default, rename = "isRecurring")]
    pub is_recurring: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NymVpnSubscriptionStatus {
    Pending,
    Complete,
    Active,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NymVpnSubscriptionKind {
    OneMonth,
    OneYear,
    TwoYears,
    Freepass,
    #[serde(untagged)]
    Other(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NymVpnSubscriptionResponse {
    pub is_subscription_active: bool,
    pub subscription: Option<NymVpnSubscription>,
    pub remaining_allowance_in_gb: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NymVpnSubscriptionsResponse {
    pub total_items: u64,
    pub page: u64,
    pub page_size: u64,
    pub items: Vec<NymVpnSubscription>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NymVpnUsagesResponse {
    pub total_items: u64,
    pub page: u64,
    pub page_size: u64,
    pub items: Vec<NymVpnUsage>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct NymVpnUsage {
    pub created_on_utc: String,
    pub last_updated_utc: String,
    pub id: String,
    pub subscription_id: String,
    pub valid_until_utc: String,
    pub valid_from_utc: String,
    pub bandwidth_allowance_gb: f64,
    pub bandwidth_used_gb: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NymDirectoryGatewaysResponse(Vec<NymDirectoryGateway>);

impl NymDirectoryGatewaysResponse {
    pub fn into_inner(self) -> Vec<NymDirectoryGateway> {
        self.0
    }
}

impl IntoIterator for NymDirectoryGatewaysResponse {
    type Item = NymDirectoryGateway;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScoreValue {
    Offline,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DVpnGatewayPerformance {
    pub last_updated_utc: String,
    pub score: ScoreValue,
    pub mixnet_score: ScoreValue,
    pub load: ScoreValue,
    pub uptime_percentage_last_24_hours: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NodeStaking {
    // delegations + bond
    pub total_stake: u128,
    pub total_delegations: u128,
    pub total_bond: u128,
    // number of delegations
    pub delegations: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NodeFamily {
    pub id: u32,
    pub name: String,
    pub description: String,
    // in unym
    pub family_stake: u128,
    pub members: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NymDirectoryGateway {
    pub identity_key: String,
    pub name: String,
    pub description: Option<String>,
    pub ip_packet_router: Option<IpPacketRouter>,
    pub authenticator: Option<Authenticator>,
    pub location: Location,
    pub last_probe: Option<Probe>,
    pub ip_addresses: Vec<IpAddr>,
    pub mix_port: u16,
    pub role: Role,
    pub entry: EntryInformation,
    pub bridges: Option<BridgeInformation>,
    // The performance data here originates from the nym-api, and is effectively mixnet performance
    // at the time of writing this
    pub performance: Percent,
    // Node performance information needed by the NymVPN UI / Explorer to show more information
    // about the node in a user-friendly way
    pub performance_v2: Option<DVpnGatewayPerformance>,
    pub build_information: Option<BuildInformation>,
    pub lewes_protocol_details: Option<LewesProtocolDetailsV1>,
    pub staking_data: Option<NodeStaking>,
    pub family_data: Option<NodeFamily>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EntryInformation {
    pub hostname: Option<String>,
    pub ws_port: u16,
    pub wss_port: Option<u16>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UxScore {
    pub max_score: u8,
    pub current_score: u8,
    pub color: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IpPacketRouter {
    pub address: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Authenticator {
    pub address: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BridgeInformation {
    pub version: String,
    pub transports: Vec<BridgeParameters>,
}

impl BridgeInformation {
    pub fn get_addrs(&self) -> Vec<SocketAddr> {
        let mut addrs = Vec::new();
        for transport in &self.transports {
            match transport {
                BridgeParameters::QuicPlain(params) => addrs.extend(&params.addresses),
            }
        }
        addrs
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "transport_type", content = "args")]
#[serde(rename_all = "snake_case")]
pub enum BridgeParameters {
    QuicPlain(QuicClientOptions),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QuicClientOptions {
    /// Address describing the remote transport server. This is a vec to support multiple addresses
    /// so as to support both IPv4 and IPv6. These addresses are meant to describe a single bridge
    /// as the key material should not be used across multiple instances.
    pub addresses: Vec<std::net::SocketAddr>,

    /// Override hostname used for certificate verification
    pub host: Option<String>,

    /// Use identity public key to verify server self signed certificate
    pub id_pubkey: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Role {
    // a properly active mixnode
    Mixnode {
        layer: u8,
    },

    #[serde(alias = "entry", alias = "gateway")]
    EntryGateway,

    #[serde(alias = "exit")]
    ExitGateway,

    // equivalent of node that's in rewarded set but not in the inactive set
    Standby,

    Inactive,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BuildInformation {
    pub build_version: String,
    pub commit_branch: String,
    pub commit_sha: String,
}

impl NymDirectoryGateway {
    pub fn is_fully_operational_entry(&self) -> bool {
        self.last_probe
            .as_ref()
            .map(|probe| probe.is_fully_operational_entry())
            .unwrap_or(false)
    }

    pub fn is_fully_operational_exit(&self) -> bool {
        self.last_probe
            .as_ref()
            .map(|probe| probe.is_fully_operational_exit())
            .unwrap_or(false)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AsnKind {
    Residential,
    Other,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Asn {
    pub asn: String,
    pub name: String,
    pub route: String,
    pub kind: AsnKind,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Location {
    pub two_letter_iso_country_code: String,
    pub latitude: f64,
    pub longitude: f64,

    pub city: String,
    pub region: String,

    pub asn: Option<Asn>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Probe {
    pub last_updated_utc: String,
    pub outcome: ProbeOutcome,
}

impl Probe {
    pub fn is_fully_operational_entry(&self) -> bool {
        self.is_recently_updated() && self.outcome.is_fully_operational_entry()
    }

    pub fn is_fully_operational_exit(&self) -> bool {
        self.is_recently_updated() && self.outcome.is_fully_operational_exit()
    }

    fn is_recently_updated(&self) -> bool {
        UtcDateTime::parse(&self.last_updated_utc, &Iso8601::DEFAULT)
            .map(|last_updated| {
                let now = UtcDateTime::now();
                let duration = now - last_updated;

                duration.whole_minutes() < MAX_PROBE_RESULT_AGE_MINUTES
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeOutcome {
    pub as_entry: Entry,
    pub as_exit: Option<Exit>,
    pub wg: Option<WgProbeResults>,
    pub socks5: Option<Socks5>,
    pub lp: Option<Lp>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Socks5 {
    pub can_proxy_https: bool,
    pub score: Option<ScoreValue>,
    pub errors: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lp {
    pub can_connect: bool,
    pub can_handshake: bool,
    pub can_register: bool,
    pub error: Option<String>,
}

impl ProbeOutcome {
    pub fn is_fully_operational_entry(&self) -> bool {
        self.as_entry.can_connect && self.as_entry.can_route
    }

    pub fn is_fully_operational_exit(&self) -> bool {
        self.as_entry.can_connect
            && self.as_entry.can_route
            && self.as_exit.as_ref().is_some_and(|exit| {
                exit.can_connect
                    && exit.can_route_ip_v4
                    && exit.can_route_ip_external_v4
                    && exit.can_route_ip_v6
                    && exit.can_route_ip_external_v6
            })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub can_connect: bool,
    pub can_route: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exit {
    pub can_connect: bool,
    pub can_route_ip_v4: bool,
    pub can_route_ip_external_v4: bool,
    pub can_route_ip_v6: bool,
    pub can_route_ip_external_v6: bool,
    pub socks5: Option<Socks5>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename = "wg")]
pub struct WgProbeResults {
    pub can_register: bool,
    pub can_handshake: bool,
    pub can_resolve_dns: bool,
    #[serde(default)]
    pub can_query_metadata_v4: Option<bool>,
    pub ping_hosts_performance: f32,
    pub ping_ips_performance: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NymDirectoryGatewayCountriesResponse(Vec<NymDirectoryCountry>);

impl NymDirectoryGatewayCountriesResponse {
    pub fn into_inner(self) -> Vec<NymDirectoryCountry> {
        self.0
    }
}

impl IntoIterator for NymDirectoryGatewayCountriesResponse {
    type Item = NymDirectoryCountry;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NymDirectoryCountry(String);

impl NymDirectoryCountry {
    pub fn iso_code(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl From<String> for NymDirectoryCountry {
    fn from(s: String) -> Self {
        Self(s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NymErrorResponse {
    pub message: String,
    pub message_id: Option<String>,
    pub code_reference_id: Option<String>,
    pub status: String,
}

impl fmt::Display for NymErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = [
            Some(format!("message: {}", self.message)),
            self.message_id
                .as_deref()
                .map(|x| format!("message_id: {x}")),
            self.code_reference_id
                .as_deref()
                .map(|x| format!("code_reference_id: {x}")),
            Some(format!("status: {}", self.status)),
        ]
        .iter()
        .filter_map(|x| x.clone())
        .collect::<Vec<_>>();
        write!(f, "{}", fields.join(", "))
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnexpectedError {
    pub message: String,
}

impl fmt::Display for UnexpectedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusOk {
    pub status: String,
}

impl fmt::Display for StatusOk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.status)
    }
}

pub fn extract_error_response(err: &VpnApiClientError) -> Option<NymErrorResponse> {
    // Try to extract the HttpClientError and parse structured error response
    if let Some(nym_http_api_client::HttpClientError::EndpointFailure { error, .. }) =
        err.http_client_error()
    {
        // Try to parse the error string as NymErrorResponse
        if let Ok(parsed) = serde_json::from_str::<NymErrorResponse>(error) {
            return Some(parsed);
        }
    }
    None
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ApiUrl {
    pub url: String,
    pub fronts: Option<Vec<String>>,
}

impl ApiUrl {
    pub fn new<S: AsRef<str>, T: AsRef<str>>(url: S, fronts: Option<Vec<T>>) -> Self {
        Self {
            url: url.as_ref().to_string(),
            fronts: fronts.map(|fronts| {
                fronts
                    .into_iter()
                    .map(|front| front.as_ref().to_string())
                    .collect()
            }),
        }
    }
}

impl From<nym_network_defaults::ApiUrl> for ApiUrl {
    fn from(value: nym_network_defaults::ApiUrl) -> Self {
        ApiUrl {
            url: value.url,
            fronts: value.front_hosts,
        }
    }
}

impl From<ApiUrl> for nym_network_defaults::ApiUrl {
    fn from(value: ApiUrl) -> Self {
        nym_network_defaults::ApiUrl {
            url: value.url,
            front_hosts: value.fronts,
        }
    }
}

impl From<&ApiUrl> for nym_network_defaults::ApiUrl {
    fn from(value: &ApiUrl) -> Self {
        nym_network_defaults::ApiUrl {
            url: value.url.clone(),
            front_hosts: value.fronts.clone(),
        }
    }
}

// The response type we fetch from the discovery endpoint
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct NymWellknownDiscoveryItemResponse {
    pub network_name: String,
    pub networking_specifics: NetworkingSpecifics,
    pub nym_api_urls: Vec<ApiUrl>,
    pub nym_vpn_api_urls: Vec<ApiUrl>,
    pub account_management: Option<AccountManagementResponse>,
    pub feature_flags: Option<serde_json::Value>,
    pub system_messages: Option<Vec<SystemMessageResponse>>,
    pub system_configuration: Option<SystemConfigurationResponse>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct AccountManagementResponse {
    pub url: String,
    pub paths: AccountManagementPathsResponse,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct AccountManagementPathsResponse {
    pub sign_up: String,
    pub sign_in: String,
    pub account: String,
    pub privy: AccountManagementPrivyPathsResponse,
    pub autologin: AccountManagementAutologinPathsResponse,
    pub pricing: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct AccountManagementAutologinPathsResponse {
    pub mobile: String,
    pub desktop: String,
    pub web: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct AccountManagementPrivyPathsResponse {
    pub mobile: String,
    pub desktop: String,
    pub web: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SystemMessageResponse {
    pub name: String,
    pub display_from: String,
    pub display_until: String,
    pub message: String,
    pub properties: serde_json::Value,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct SystemConfigurationResponse {
    pub mix_thresholds: ScoreThresholdsResponse,
    pub wg_thresholds: ScoreThresholdsResponse,
    pub statistics_api: Option<String>,
    pub min_supported_app_versions: Option<NetworkCompatibility>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct NetworkCompatibilityResponse {
    pub core: String,
    pub macos: String,
    pub ios: String,
    pub tauri: String,
    pub android: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ScoreThresholdsResponse {
    pub high: u8,
    pub medium: u8,
    pub low: u8,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct NymWellknownDiscoveryItem {
    pub network_name: String,
    pub nym_api_url: String,
    pub nym_vpn_api_url: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct NymUserGeoIpLocationResponse {
    pub ip: String,
    pub location: Location,
}

pub type RegisteredNetworksResponse = HashSet<String>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nym_vpn_zk_nym_upgrade_mode_response_parsing() {
        let raw_response = r#"{"id":"dj2e1yzc9jj9s8n","status":"upgrade_mode","ticketbook_type":"v1-mixnet-entry","last_updated_utc":"2025-11-13 15:40:31.166Z","created_on_utc":"2025-11-13 15:40:30.921Z","valid_until_utc":"2025-11-13 23:00:00.000Z","valid_from_utc":"2025-11-13 15:40:30.916Z","issued_bandwidth_in_gb":25,"upgrade_mode":{"upgrade_mode_attestation":{"attester_public_key":"6sfL7xcCzmcsxA1uXtnExcpA7KWypCcsUbs7SzUADxng","authorised_jwt_issuers":["Ddfd2WDCWbW28hmrrGV24GxQxBuGmg1Ra41k3TGn3B4Z"],"signature":"3UR5cL9XzDCFVHt9WVmJ4KemcftbfpCa63TBa9wd6w5dW4uq7u4BwS5UsBfwMB49StH5wDCbSpN1borZbfMnitxn","starting_time":"2025-11-10T09:15:18.967542Z","type":"upgrade_mode"},"upgrade_mode_jwt":"eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCIsImp3ayI6IkRkZmQyV0RDV2JXMjhobXJyR1YyNEd4UXhCdUdtZzFSYTQxazNUR24zQjRaIn0.eyJpYXQiOjE3NjMwNDc5NzYsImV4cCI6MTc2MzA1MTU3NiwibmJmIjoxNzYzMDQ3OTc2LCJpc3MiOiJueW0tY3JlZGVudGlhbC1wcm94eSIsIm5vbmNlIjoiR2ZtYzJYY2c2NUdkc0JaOWJ3RFg2OGxONmNfR2V5NUwiLCJ0eXBlIjoidXBncmFkZV9tb2RlIiwic3RhcnRpbmdfdGltZSI6IjIwMjUtMTEtMTBUMDk6MTU6MTguOTY3NTQyWiIsImF0dGVzdGVyX3B1YmxpY19rZXkiOiI2c2ZMN3hjQ3ptY3N4QTF1WHRuRXhjcEE3S1d5cENjc1ViczdTelVBRHhuZyIsImF1dGhvcmlzZWRfand0X2lzc3VlcnMiOlsiRGRmZDJXRENXYlcyOGhtcnJHVjI0R3hReEJ1R21nMVJhNDFrM1RHbjNCNFoiXSwic2lnbmF0dXJlIjoiM1VSNWNMOVh6RENGVkh0OVdWbUo0S2VtY2Z0YmZwQ2E2M1RCYTl3ZDZ3NWRXNHVxN3U0QndTNVVzQmZ3TUI0OVN0SDV3RENiU3BOMWJvclpiZk1uaXR4biJ9.u66O2q-xoEMtyr-6OBEKL8BEdhoo6xLo6dHkVxDmzB3I6CZ1pFIwzgZUKNA5DT_O6bFzcLv8DyLmOuzdAe-lDw"}}"#;
        let res = serde_json::from_str::<NymVpnZkNym>(raw_response);
        assert!(res.is_ok());
    }
}
