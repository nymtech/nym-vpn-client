// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{Country, Gateway};

#[derive(Debug, Serialize, Deserialize)]
pub struct NymVpnAccountResponse {
    created_on_utc: String,
    last_updated_utc: String,
    account_addr: String,
    status: NymVpnAccountStatusResponse,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NymVpnAccountStatusResponse {
    Active,
    Inactive,
    DeleteMe,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NymVpnAccountSummaryResponse {
    account: NymVpnAccountResponse,
    subscription: NymVpnAccountSummarySubscription,
    devices: NymVpnAccountSummaryDevices,
    fair_usage: NymVpnAccountSummaryFairUsage,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NymVpnAccountSummarySubscription {
    is_active: bool,
    active: Option<NymVpnSubscription>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NymVpnAccountSummaryDevices {
    active: u64,
    max: u64,
    remaining: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NymVpnAccountSummaryFairUsage {
    used_gb: f64,
    limit_gb: f64,
    resets_on_utc: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NymVpnDevice {
    created_on_utc: String,
    last_updated_utc: String,
    device_identity_key: String,
    status: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NymVpnDeviceStatus {
    Active,
    Inactive,
    DeleteMe,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NymVpnDevicesResponse {
    total_items: u64,
    page: u64,
    page_size: u64,
    devices: Vec<NymVpnDevice>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NymVpnRefundsResponse {
    total_items: u64,
    page: u64,
    page_size: u64,
    items: Vec<NymVpnRefund>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NymVpnRefund {
    created_on_utc: String,
    last_updated_utc: String,
    subscription_invoice: String,
    status: NymVpnRefundStatus,
    user_reason: NymVpnRefundUserReason,
    data: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NymVpnRefundStatus {
    Pending,
    Complete,
    Rejected,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NymVpnRefundUserReason {
    SubscriptionInError,
    PoorPerformance,
    Other,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NymVpnZkNym {
    created_on_utc: String,
    last_updated_utc: String,
    id: String,
    valid_until_utc: String,
    valid_from_utc: String,
    issued_bandwidth_in_gb: f64,
    blinded_shares: Vec<String>,
    status: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NymVpnZkNymStatus {
    Pending,
    Active,
    Revoking,
    Revoked,
    Error,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NymVpnZkNymResponse {
    total_items: u64,
    page: u64,
    page_size: u64,
    zk_nyms: Vec<NymVpnZkNym>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NymVpnSubscription {
    created_on_utc: String,
    last_updated_utc: String,
    id: String,
    valid_until_utc: String,
    valid_from_utc: String,
    status: NymVpnSubscriptionStatus,
    kind: NymVpnSubscriptionKind,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NymVpnSubscriptionStatus {
    Pending,
    Complete,
    Active,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NymVpnSubscriptionKind {
    OneMonth,
    OneYear,
    TwoYears,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NymVpnSubscriptionResponse {
    is_subscription_active: bool,
    subscription: Option<NymVpnSubscription>,
    remaining_allowance_in_gb: f64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NymVpnSubscriptionsResponse {
    total_items: u64,
    page: u64,
    page_size: u64,
    subscriptions: Vec<NymVpnSubscription>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NymDirectoryGatewaysResponse(Vec<Gateway>);

impl NymDirectoryGatewaysResponse {
    pub fn into_inner(self) -> Vec<Gateway> {
        self.0
    }
}

impl IntoIterator for NymDirectoryGatewaysResponse {
    type Item = Gateway;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NymDirectoryGatewayCountriesResponse(Vec<Country>);

impl NymDirectoryGatewayCountriesResponse {
    pub fn into_inner(self) -> Vec<Country> {
        self.0
    }
}

impl IntoIterator for NymDirectoryGatewayCountriesResponse {
    type Item = Country;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
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
#[serde(rename_all = "camelCase")]
pub struct UnexpectedError {
    pub message: String,
}

impl fmt::Display for UnexpectedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
