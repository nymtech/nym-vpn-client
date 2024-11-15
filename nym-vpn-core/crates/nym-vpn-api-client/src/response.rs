// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;

use chrono::{DateTime, Utc};
use itertools::Itertools;
use nym_contracts_common::Percent;
use nym_credential_proxy_requests::api::v1::ticketbook::models::TicketbookWalletSharesResponse;
use serde::{Deserialize, Serialize};

const MAX_PROBE_RESULT_AGE_MINUTES: i64 = 60;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NymVpnAccountResponse {
    pub created_on_utc: String,
    pub last_updated_utc: String,
    pub account_addr: String,
    pub status: NymVpnAccountStatusResponse,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NymVpnAccountStatusResponse {
    Active,
    Inactive,
    DeleteMe,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NymVpnAccountSummaryResponse {
    pub account: NymVpnAccountResponse,
    pub subscription: NymVpnAccountSummarySubscription,
    pub devices: NymVpnAccountSummaryDevices,
    pub fair_usage: NymVpnAccountSummaryFairUsage,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NymVpnAccountSummarySubscription {
    pub is_active: bool,
    pub active: Option<NymVpnSubscription>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NymVpnAccountSummaryDevices {
    pub active: u64,
    pub max: u64,
    pub remaining: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NymVpnAccountSummaryFairUsage {
    pub used_gb: Option<f64>,
    pub limit_gb: Option<f64>,
    pub resets_on_utc: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NymVpnHealthResponse {
    pub status: String,
    pub timestamp_utc: DateTime<Utc>,
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
                .format_with(", ", |item, f| f(&format_args!("{:?}", item)))
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
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NymVpnZkNymStatus {
    Pending,
    Active,
    Revoking,
    Revoked,
    Error,
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NymDirectoryGateway {
    pub identity_key: String,
    pub ip_packet_router: Option<IpPacketRouter>,
    pub authenticator: Option<Authenticator>,
    pub location: Location,
    pub last_probe: Option<Probe>,
    pub ip_addresses: Vec<String>,
    pub mix_port: u16,
    pub role: Role,
    pub entry: EntryInformation,
    // The performance data here originates from the nym-api, and is effectively mixnet performance
    // at the time of writing this
    pub performance: Percent,
    pub build_information: BuildInformation,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EntryInformation {
    pub hostname: Option<String>,
    pub ws_port: u16,
    pub wss_port: Option<u16>,
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Location {
    pub two_letter_iso_country_code: String,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Probe {
    pub last_updated_utc: String,
    pub outcome: ProbeOutcome,
}

impl Probe {
    pub fn is_fully_operational_entry(&self) -> bool {
        if !is_recently_updated(&self.last_updated_utc) {
            return false;
        }
        self.outcome.is_fully_operational_entry()
    }

    pub fn is_fully_operational_exit(&self) -> bool {
        if !is_recently_updated(&self.last_updated_utc) {
            return false;
        }
        self.outcome.is_fully_operational_exit()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeOutcome {
    pub as_entry: Entry,
    pub as_exit: Option<Exit>,
    pub wg: Option<WgProbeResults>,
}

impl ProbeOutcome {
    pub fn is_fully_operational_entry(&self) -> bool {
        self.as_entry.can_connect && self.as_entry.can_route
    }

    pub fn is_fully_operational_exit(&self) -> bool {
        self.as_entry.can_connect
            && self.as_entry.can_route
            && self.as_exit.as_ref().map_or(false, |exit| {
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
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename = "wg")]
pub struct WgProbeResults {
    pub can_register: bool,
    pub can_handshake: bool,
    pub can_resolve_dns: bool,
    pub ping_hosts_performance: f32,
    pub ping_ips_performance: f32,
}

fn is_recently_updated(last_updated_utc: &str) -> bool {
    if let Ok(last_updated) = last_updated_utc.parse::<chrono::DateTime<chrono::Utc>>() {
        let now = chrono::Utc::now();
        let duration = now - last_updated;
        duration.num_minutes() < MAX_PROBE_RESULT_AGE_MINUTES
    } else {
        false
    }
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

#[derive(Debug, Serialize, Deserialize, Default)]
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
                .map(|x| format!("message_id: {}", x)),
            self.code_reference_id
                .as_deref()
                .map(|x| format!("code_reference_id: {}", x)),
            Some(format!("status: {}", self.status)),
        ]
        .iter()
        .filter_map(|x| x.clone())
        .collect::<Vec<_>>();
        write!(f, "{}", fields.join(", "))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorMessage {
    message: String,
}

impl fmt::Display for ErrorMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
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
    status: String,
}
