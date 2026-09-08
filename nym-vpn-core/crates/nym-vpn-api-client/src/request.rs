// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountKind {
    PrivySecp256k1,
    UserGeneratedSecp256k1,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAndroidAccountRequestBody {
    pub account_addr: String,
    pub pub_key: String,
    pub signature_base64: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purchase_token: Option<String>,
    pub kind: AccountKind,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAppleAccountRequestBody {
    pub account_addr: String,
    pub pub_key: String,
    pub signature_base64: String,
    pub kind: AccountKind,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterDeviceRequestBody {
    pub device_identity_key: String,
    pub signature: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestZkNymRequestBody {
    pub withdrawal_request: String,
    pub ecash_pubkey: String,
    pub expiration_date: String,
    pub ticketbook_type: String,
    /// Make sure credential proxy knows we can differentiate between epochs
    /// (allows us to request during DKG ceremony)
    pub epoch_aware: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplyFreepassRequestBody {
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSubscriptionInvoicesRequestBody {
    pub subscription: String,
    pub date: String,
    pub status: CreateSubscriptionInvoicesStatus,
    pub invoice_no: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CreateSubscriptionInvoicesStatus {
    Unpaid,
    Paid,
    Cancelled,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSubscriptionRequestBody {
    pub valid_from_utc: String,
    pub subscription_kind: CreateSubscriptionKind,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CreateSubscriptionKind {
    OneMonth,
    OneYear,
    TwoYears,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestRefundRequestBody {
    subscription_invoice: String,
    status: RequestRefundRequestStatus,
    user_reason: RequestRefundRequestUserReason,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestRefundRequestStatus {
    Pending,
    Complete,
    Rejected,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestRefundRequestUserReason {
    SubscriptionInError,
    PoorPerformance,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDeviceRequestBody {
    pub status: UpdateDeviceRequestStatus,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateDeviceRequestStatus {
    Active,
    Inactive,
    DeleteMe,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LinkAccountRequestBody {
    pub pubkey: String,
    pub signature: String,
    pub kind: String,
    pub label: String,
}
