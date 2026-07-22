// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod controller_error;
pub mod controller_event;
pub mod controller_state;
pub mod deeplink;
pub mod storage;
pub mod ticketbooks;

use std::time::Duration;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
#[cfg(feature = "typescript-bindings")]
use ts_rs::TS;

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg(any(target_os = "ios", target_os = "android"))]
pub struct RegisterAccountRequest {
    /// Present for Google Play in-app purchases
    /// Omitted for anonymous registration
    #[cfg(target_os = "android")]
    pub purchase_token: Option<String>,
}

#[cfg(feature = "nym-type-conversions")]
#[cfg(any(target_os = "ios", target_os = "android"))]
impl From<RegisterAccountRequest> for nym_vpn_api_client::types::Platform {
    fn from(_value: RegisterAccountRequest) -> Self {
        #[cfg(target_os = "ios")]
        {
            Self::Apple
        }

        #[cfg(target_os = "android")]
        {
            Self::Android {
                purchase_token: _value.purchase_token,
            }
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
pub struct RegisterAccountResponse {
    pub account_token: String,
}

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Error))]
pub enum AccountCommandError {
    // Internal error that should not happen
    #[error("internal error: {0}")]
    Internal(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("vpn-api error")]
    VpnApi(#[from] VpnApiError),

    #[error("unexpected vpn-api response: {0}")]
    UnexpectedVpnApiResponse(String),

    #[error("failed to connect to nyxd instance: {0}")]
    NyxdConnectionFailure(String),

    #[error("failed to resolve query to a nyxd instance: {0}")]
    NyxdQueryFailure(String),

    #[error("account doesn't exist on chain")]
    AccountDoesntExistOnChain,

    #[error("no account stored")]
    NoAccountStored,

    #[error("no device stored")]
    NoDeviceStored,

    #[error("an account is already stored")]
    ExistingAccount,

    #[error("no connectivity")]
    Offline,

    #[error("account is not set in decentralised mode")]
    AccountNotDecentralised,

    #[error("account is set in decentralised mode")]
    AccountDecentralised,

    #[error("account does not have sufficient funds")]
    InsufficientFunds,

    #[error("failed to obtain zk-nym: {0}")]
    ZkNymAcquisitionFailure(String),

    #[error("invalid passphrase: {0}")]
    InvalidMnemonic(String),

    #[error("invalid secret: {0}")]
    InvalidSecret(String),

    #[error("deeplink error: {0}")]
    DeeplinkError(String),
}

impl AccountCommandError {
    pub fn internal(message: impl ToString) -> Self {
        AccountCommandError::Internal(message.to_string())
    }

    pub fn storage(message: impl ToString) -> Self {
        AccountCommandError::Storage(message.to_string())
    }

    pub fn unexpected_response(message: impl std::fmt::Debug) -> Self {
        AccountCommandError::UnexpectedVpnApiResponse(format!("{message:?}"))
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_validator_client::nyxd::error::NyxdError> for AccountCommandError {
    fn from(e: nym_validator_client::nyxd::error::NyxdError) -> Self {
        AccountCommandError::NyxdQueryFailure(e.to_string())
    }
}

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Error))]
pub enum VpnApiError {
    #[error("timeout: {0}")]
    Timeout(String),

    #[error("status code: {code}, error: {msg}")]
    StatusCode { code: u16, msg: String },

    #[error(transparent)]
    Response(#[from] VpnApiErrorResponse),
}

impl VpnApiError {
    pub fn message(&self) -> String {
        match self {
            VpnApiError::Response(err) => err.message.clone(),
            VpnApiError::StatusCode { .. } => self.to_string(),
            VpnApiError::Timeout(_) => self.to_string(),
        }
    }

    pub fn message_id(&self) -> Option<String> {
        if let VpnApiError::Response(err) = self {
            err.message_id.clone()
        } else {
            None
        }
    }

    pub fn code_reference_id(&self) -> Option<String> {
        if let VpnApiError::Response(err) = self {
            err.code_reference_id.clone()
        } else {
            None
        }
    }
}

// That should disappear when reworking those errors
#[cfg(feature = "nym-type-conversions")]
impl TryFrom<nym_vpn_api_client::error::VpnApiClientError> for VpnApiError {
    type Error = nym_vpn_api_client::error::VpnApiClientError;

    fn try_from(err: nym_vpn_api_client::error::VpnApiClientError) -> Result<Self, Self::Error> {
        let err = match VpnApiErrorResponse::try_from(err) {
            Ok(err) => return Ok(Self::Response(err)),
            Err(err) => err,
        };

        if err
            .http_client_error()
            .is_some_and(nym_vpn_api_client::error::HttpClientError::is_timeout)
        {
            return Ok(Self::Timeout(err.to_string()));
        }

        match err
            .http_client_error()
            .and_then(nym_vpn_api_client::error::HttpClientError::status_code)
        {
            Some(code) => Ok(Self::StatusCode {
                code: code.into(),
                msg: err.to_string(),
            }),
            None => Err(err),
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_vpn_api_client::error::VpnApiClientError> for AccountCommandError {
    fn from(err: nym_vpn_api_client::error::VpnApiClientError) -> Self {
        use nym_vpn_api_client::response::NymErrorResponse;
        // TODO: Another example of losing information about the original error cause
        match NymErrorResponse::try_from(err) {
            Ok(err) => AccountCommandError::VpnApi(VpnApiError::Response(err.into())),
            Err(e) => AccountCommandError::Internal(e.to_string()),
        }
    }
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
#[error("{message}, message_id: {message_id:?}, code_reference_id: {code_reference_id:?}")]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
pub struct VpnApiErrorResponse {
    pub message: String,
    pub message_id: Option<String>,
    pub code_reference_id: Option<String>,
}

#[cfg(feature = "nym-type-conversions")]
impl TryFrom<nym_vpn_api_client::error::VpnApiClientError> for VpnApiErrorResponse {
    type Error = nym_vpn_api_client::error::VpnApiClientError;

    fn try_from(err: nym_vpn_api_client::error::VpnApiClientError) -> Result<Self, Self::Error> {
        Ok(VpnApiErrorResponse::from(
            nym_vpn_api_client::response::NymErrorResponse::try_from(err)?,
        ))
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_vpn_api_client::response::NymErrorResponse> for VpnApiErrorResponse {
    fn from(err: nym_vpn_api_client::response::NymErrorResponse) -> Self {
        Self {
            message: err.message,
            message_id: err.message_id,
            code_reference_id: err.code_reference_id,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct VpnAccountSummary {
    pub traffic_used_gb: u64,

    pub traffic_limit_gb: u64,

    #[cfg_attr(feature = "typescript-bindings", ts(as = "String"))]
    #[cfg_attr(feature = "serde", serde(with = "time::serde::iso8601::option"))]
    pub traffic_reset_time: Option<OffsetDateTime>,

    /// True when the API could not retrieve fair-usage data from the database.
    /// Clients should treat this as fail-open rather than surfacing a quota-exceeded error.
    pub fair_usage_data_unavailable: bool,

    pub account_addr: String,
    pub canonical_account_addr: Option<String>,
    pub auth_methods: Vec<VpnAccountAuthMethod>,
    pub account_mode: Option<StoredAccountMode>,
    pub subscription: Option<Subscription>,
    pub is_subscription_stacked: bool,

    /// Status of the account itself (active/inactive/delete-me).
    pub account_status: VpnAccountStatus,

    /// Number of additional devices that can still be registered to this account.
    pub remaining_devices: u64,

    /// Whether the current device is registered and active on this account.
    pub is_device_active: bool,

    /// Whether the time was acceptably synced when the summary was built
    pub time_synced: bool,

    /// Additional staleness flag
    pub stale: bool,

    /// When this summary was last synced from the VPN API. Used to decide when a
    /// background refresh is due.
    #[cfg_attr(feature = "typescript-bindings", ts(as = "String"))]
    #[cfg_attr(feature = "serde", serde(with = "time::serde::iso8601"))]
    pub last_synced_utc: OffsetDateTime,
}

// Exported methods
#[cfg_attr(feature = "uniffi-bindings", uniffi::export)]
#[allow(unused)]
impl VpnAccountSummary {
    /// Returns true if the account itself is active.
    pub fn is_account_active(&self) -> bool {
        matches!(self.account_status, VpnAccountStatus::Active)
    }

    /// Returns true if there is a subscription that exists but is not yet active
    /// (e.g. a cash payment still processing).
    pub fn is_subscription_pending(&self) -> bool {
        matches!(
            self.subscription.as_ref().map(|sub| &sub.status),
            Some(NymVpnSubscriptionStatus::Pending)
        )
    }

    /// Returns true if subscription is active
    pub fn is_subscription_active(&self) -> bool {
        if let Some(subscription) = &self.subscription {
            match subscription.status {
                NymVpnSubscriptionStatus::Active => {
                    subscription.subscription.valid_until_utc
                        > OffsetDateTime::now_utc().unix_timestamp()
                }
                _ => false,
            }
        } else {
            false
        }
    }

    pub fn fair_usage_left(&self) -> bool {
        if !self.is_subscription_active() {
            return false;
        }
        if self.fair_usage_data_unavailable {
            // Data gap from the API - fail-open so users are not blocked by infrastructure errors.
            return true;
        }
        self.traffic_limit_gb > 0 && self.traffic_used_gb < self.traffic_limit_gb
    }

    pub fn is_linked(&self) -> bool {
        self.auth_methods
            .iter()
            .any(|method| method.kind == "privy_secp256k1")
    }

    // Stale if explicitly flagged, older than max_age, or holding a depleted snapshot synced
    // before an elapsed daily reset boundary (fair usage left + reset time passed).
    pub fn is_stale(&self, max_age: Duration) -> bool {
        self.is_stale_at(OffsetDateTime::now_utc(), max_age)
    }

    pub(crate) fn is_stale_at(&self, now: OffsetDateTime, max_age: Duration) -> bool {
        self.stale
            || self.fair_usage_depleted_past_reset(now)
            || now.unix_timestamp() - self.last_synced_utc.unix_timestamp()
                > max_age.as_secs() as i64
    }

    fn fair_usage_depleted_past_reset(&self, now: OffsetDateTime) -> bool {
        !self.fair_usage_left()
            && self
                .traffic_reset_time
                .is_some_and(|reset_time| reset_time <= now && self.last_synced_utc <= reset_time)
    }
}

#[cfg(feature = "nym-type-conversions")]
impl VpnAccountSummary {
    pub fn from_parts(
        api_summary: &nym_vpn_api_client::response::NymVpnAccountSummaryWithDeviceResponse,
        account_mode: nym_vpn_api_client::types::VpnAccountMode,
        remote_time: nym_vpn_api_client::types::VpnApiTime,
    ) -> Result<Self, nym_vpn_api_client::error::VpnApiClientError> {
        let account_summary = &api_summary.account_summary;
        let traffic_reset_time = account_summary
            .fair_usage
            .resetsOnUtc
            .as_deref()
            .and_then(|t| parse_timestamp(t, "fair_usage.resetsOnUtc"));

        let auth_methods = account_summary
            .account
            .auth_methods
            .iter()
            .cloned()
            .map(VpnAccountAuthMethod::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        // Active wins over pending; `is_subscription_active()` ignores wire `is_active`.
        let subscription = if let Some(active) = account_summary.subscription.active.as_ref() {
            Some(Subscription {
                status: NymVpnSubscriptionStatus::Active,
                subscription: NymVpnSubscription::try_from(active)?,
            })
        } else if let Some(pending) = account_summary.subscription.pending.as_ref() {
            Some(Subscription {
                status: NymVpnSubscriptionStatus::Pending,
                subscription: NymVpnSubscription::try_from(pending)?,
            })
        } else {
            None
        };

        Ok(Self {
            traffic_used_gb: account_summary.fair_usage.usedGB,
            traffic_limit_gb: account_summary.fair_usage.limitGB,
            traffic_reset_time,
            fair_usage_data_unavailable: account_summary.fair_usage.data_unavailable,
            account_addr: account_summary.account.account_addr.clone(),
            canonical_account_addr: account_summary.account.canonical_account_addr.clone(),
            auth_methods,
            account_mode: Some(account_mode.into()),
            subscription,
            is_subscription_stacked: account_summary.subscription.is_stacked,
            account_status: account_summary.account.status.clone().into(),
            remaining_devices: account_summary.devices.remaining,
            is_device_active: api_summary.active_device.is_some(),
            time_synced: remote_time.is_acceptable_synced(),
            stale: false,
            last_synced_utc: OffsetDateTime::now_utc(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct VpnAccountAuthMethod {
    pub id: String,
    pub pubkey: String,
    pub kind: String,
    pub label: String,
    pub status: VpnAccountStatus,

    #[cfg_attr(feature = "typescript-bindings", ts(as = "String"))]
    #[cfg_attr(feature = "serde", serde(with = "time::serde::iso8601"))]
    pub created: OffsetDateTime,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub enum NymVpnSubscriptionKind {
    OneMonth,
    OneYear,
    TwoYears,
    Freepass,
    #[cfg_attr(feature = "serde", serde(untagged))]
    Other(String),
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_vpn_api_client::response::NymVpnSubscriptionKind> for NymVpnSubscriptionKind {
    fn from(value: nym_vpn_api_client::response::NymVpnSubscriptionKind) -> Self {
        match value {
            nym_vpn_api_client::response::NymVpnSubscriptionKind::OneMonth => {
                NymVpnSubscriptionKind::OneMonth
            }
            nym_vpn_api_client::response::NymVpnSubscriptionKind::OneYear => {
                NymVpnSubscriptionKind::OneYear
            }
            nym_vpn_api_client::response::NymVpnSubscriptionKind::TwoYears => {
                NymVpnSubscriptionKind::TwoYears
            }
            nym_vpn_api_client::response::NymVpnSubscriptionKind::Freepass => {
                NymVpnSubscriptionKind::Freepass
            }
            nym_vpn_api_client::response::NymVpnSubscriptionKind::Other(value) => {
                NymVpnSubscriptionKind::Other(value)
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub enum NymVpnSubscriptionStatus {
    Pending,
    Active,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct NymVpnSubscription {
    pub created_on_utc: String,
    pub last_updated_utc: String,
    pub id: String,
    pub valid_until_utc: i64,
    pub valid_from_utc: i64,
    pub status: String,
    pub kind: NymVpnSubscriptionKind,
    pub is_recurring: bool,
}

#[cfg(feature = "nym-type-conversions")]
impl TryFrom<&nym_vpn_api_client::response::NymVpnSubscription> for NymVpnSubscription {
    type Error = nym_vpn_api_client::error::VpnApiClientError;

    fn try_from(
        value: &nym_vpn_api_client::response::NymVpnSubscription,
    ) -> Result<Self, Self::Error> {
        let valid_until_utc =
            parse_timestamp(&value.valid_until_utc, "subscription.valid_until_utc")
                .map(|t| t.unix_timestamp())
                .ok_or_else(|| {
                    nym_vpn_api_client::error::VpnApiClientError::PayloadError(format!(
                        "invalid subscription.valid_until_utc: {}",
                        value.valid_until_utc
                    ))
                })?;
        let valid_from_utc = parse_timestamp(&value.valid_from_utc, "subscription.valid_from_utc")
            .map(|t| t.unix_timestamp())
            .ok_or_else(|| {
                nym_vpn_api_client::error::VpnApiClientError::PayloadError(format!(
                    "invalid subscription.valid_from_utc: {}",
                    value.valid_from_utc
                ))
            })?;
        Ok(Self {
            created_on_utc: value.created_on_utc.clone(),
            last_updated_utc: value.last_updated_utc.clone(),
            id: value.id.clone(),
            valid_until_utc,
            valid_from_utc,
            status: format!("{:?}", value.status).to_lowercase(),
            kind: value.kind.clone().into(),
            is_recurring: value.is_recurring,
        })
    }
}

#[cfg(feature = "nym-type-conversions")]
fn parse_timestamp(raw: &str, field: &'static str) -> Option<OffsetDateTime> {
    let normalized = raw.replace(' ', "T");
    match OffsetDateTime::parse(&normalized, &time::format_description::well_known::Rfc3339) {
        Ok(t) => Some(t),
        Err(err) => {
            tracing::warn!("failed to parse {field} {raw:?}: {err}");
            None
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub struct Subscription {
    pub status: NymVpnSubscriptionStatus,
    pub subscription: NymVpnSubscription,
}

#[cfg(feature = "nym-type-conversions")]
impl TryFrom<nym_vpn_api_client::response::NymVpnAccountAuthMethodResponse>
    for VpnAccountAuthMethod
{
    type Error = nym_vpn_api_client::error::VpnApiClientError;

    fn try_from(
        value: nym_vpn_api_client::response::NymVpnAccountAuthMethodResponse,
    ) -> Result<Self, Self::Error> {
        let created = parse_timestamp(&value.created, "auth_method.created").ok_or_else(|| {
            nym_vpn_api_client::error::VpnApiClientError::PayloadError(format!(
                "invalid auth_method.created time format: {}",
                value.created
            ))
        })?;

        Ok(Self {
            id: value.id,
            pubkey: value.pubkey,
            kind: value.kind,
            label: value.label,
            status: value.status.into(),
            created,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub enum VpnAccountStatus {
    Active,
    Inactive,
    DeleteMe,
}

impl std::fmt::Display for VpnAccountStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Mirror the wire representation (`strum::Display` on the API enum) so error messages
        // remain stable: "Active" / "Inactive" / "DeleteMe".
        let s = match self {
            VpnAccountStatus::Active => "Active",
            VpnAccountStatus::Inactive => "Inactive",
            VpnAccountStatus::DeleteMe => "DeleteMe",
        };
        f.write_str(s)
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_vpn_api_client::response::NymVpnAccountStatusResponse> for VpnAccountStatus {
    fn from(value: nym_vpn_api_client::response::NymVpnAccountStatusResponse) -> Self {
        match value {
            nym_vpn_api_client::response::NymVpnAccountStatusResponse::Active => {
                VpnAccountStatus::Active
            }
            nym_vpn_api_client::response::NymVpnAccountStatusResponse::Inactive => {
                VpnAccountStatus::Inactive
            }
            nym_vpn_api_client::response::NymVpnAccountStatusResponse::DeleteMe => {
                VpnAccountStatus::DeleteMe
            }
        }
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
pub enum StoredAccountMode {
    /// Account works in the API mode, i.e. the subscription is managed
    /// by the VPN API which provides required ticketbooks
    #[default]
    Api,

    /// Account works in the decentralised mode, i.e. there is no associated subscription
    /// and the account uses its own funds for obtaining required ticketbooks
    Decentralised,

    /// Account works in the API mode, but the mnemonic is derived from the Privy
    /// wallet private key.
    Privy,
}

#[cfg(feature = "nym-type-conversions")]
impl From<StoredAccountMode> for nym_vpn_api_client::types::VpnAccountMode {
    fn from(mode: StoredAccountMode) -> Self {
        use nym_vpn_api_client::types::VpnAccountMode;
        match mode {
            StoredAccountMode::Api => VpnAccountMode::Api,
            StoredAccountMode::Decentralised => VpnAccountMode::Decentralised,
            StoredAccountMode::Privy => VpnAccountMode::Privy,
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_vpn_api_client::types::VpnAccountMode> for StoredAccountMode {
    fn from(mode: nym_vpn_api_client::types::VpnAccountMode) -> Self {
        use nym_vpn_api_client::types::VpnAccountMode;
        match mode {
            VpnAccountMode::Api => StoredAccountMode::Api,
            VpnAccountMode::Decentralised => StoredAccountMode::Decentralised,
            VpnAccountMode::Privy => StoredAccountMode::Privy,
        }
    }
}

// Regression tests for `VpnAccountSummary::try_from` and subscription serde.
#[cfg(all(test, feature = "nym-type-conversions"))]
mod tests {
    use nym_vpn_api_client::{
        response::{
            NymVpnAccountResponse, NymVpnAccountStatusResponse, NymVpnAccountSummaryDevices,
            NymVpnAccountSummaryFairUsage, NymVpnAccountSummaryResponse,
            NymVpnAccountSummarySubscription, NymVpnAccountSummaryWithDeviceResponse,
            NymVpnSubscription as ApiNymVpnSubscription,
            NymVpnSubscriptionKind as ApiNymVpnSubscriptionKind,
            NymVpnSubscriptionStatus as ApiNymVpnSubscriptionStatus,
        },
        types::{VpnAccountMode, VpnApiTime},
    };
    use tracing_test::traced_test;

    use super::*;

    /// A `VpnApiTime` reporting zero skew, so summaries built in tests count as
    /// time-synced.
    fn synced_api_time() -> VpnApiTime {
        let now = OffsetDateTime::now_utc();
        VpnApiTime::from_estimated_remote_time(now, now)
    }

    /// Build a `VpnAccountSummary` the way production code does: wrap the bare
    /// API summary in a with-device response (no active device) and run it
    /// through `from_parts` in API mode with a synced clock.
    fn build(
        summary: &NymVpnAccountSummaryResponse,
    ) -> Result<VpnAccountSummary, nym_vpn_api_client::error::VpnApiClientError> {
        let with_device = NymVpnAccountSummaryWithDeviceResponse {
            account_summary: summary.clone(),
            active_device: None,
        };
        VpnAccountSummary::from_parts(&with_device, VpnAccountMode::Api, synced_api_time())
    }

    fn far_future_active_subscription() -> ApiNymVpnSubscription {
        ApiNymVpnSubscription {
            created_on_utc: "2024-01-01T00:00:00Z".into(),
            last_updated_utc: "2024-01-01T00:00:00Z".into(),
            id: "sub_test".into(),
            valid_from_utc: "2024-01-01T00:00:00Z".into(),
            valid_until_utc: "2099-01-01T00:00:00Z".into(),
            status: ApiNymVpnSubscriptionStatus::Active,
            kind: ApiNymVpnSubscriptionKind::OneMonth,
            is_recurring: false,
        }
    }

    fn base_summary() -> NymVpnAccountSummaryResponse {
        NymVpnAccountSummaryResponse {
            account: NymVpnAccountResponse {
                created_on_utc: "2024-01-01T00:00:00Z".into(),
                last_updated_utc: "2024-01-01T00:00:00Z".into(),
                account_addr: "n1addr".into(),
                status: NymVpnAccountStatusResponse::Active,
                canonical_account_addr: None,
                auth_methods: vec![],
            },
            subscription: NymVpnAccountSummarySubscription {
                is_active: false,
                active: None,
                pending: None,
                is_stacked: false,
            },
            devices: NymVpnAccountSummaryDevices {
                active: 0,
                max: 10,
                remaining: 10,
            },
            fair_usage: NymVpnAccountSummaryFairUsage {
                usedGB: 0,
                limitGB: 2000,
                resetsOnUtc: None,
                data_unavailable: false,
            },
        }
    }

    #[test]
    fn deserializes_when_pending_is_explicit_null() {
        let json = r#"{
            "isActive": true,
            "active": null,
            "pending": null,
            "isStacked": false
        }"#;

        let parsed: NymVpnAccountSummarySubscription =
            serde_json::from_str(json).expect("must tolerate pending: null");
        assert!(parsed.is_active);
        assert!(parsed.pending.is_none());
        assert!(parsed.active.is_none());
    }

    #[test]
    fn try_from_succeeds_when_resets_on_utc_is_malformed() {
        // A bad reset timestamp must fall back to None, not fail the summary.
        let mut summary = base_summary();
        summary.fair_usage.resetsOnUtc = Some("not a date at all".into());

        let parsed = build(&summary).expect("must not fail on bad reset timestamp");
        assert!(parsed.traffic_reset_time.is_none());
    }

    #[test]
    fn try_from_accepts_space_separated_resets_on_utc() {
        let mut summary = base_summary();
        summary.fair_usage.resetsOnUtc = Some("2025-08-20 13:46:26.572Z".into());

        let parsed = build(&summary).expect("space-separated must parse");
        assert!(parsed.traffic_reset_time.is_some());
    }

    #[test]
    fn is_subscription_active_remains_true_when_traffic_reset_time_malformed() {
        // NYM-1156: bad reset timestamp must not flip a valid sub to inactive.
        let mut summary = base_summary();
        summary.fair_usage.resetsOnUtc = Some("not a date at all".into());
        summary.subscription = NymVpnAccountSummarySubscription {
            is_active: true,
            active: Some(far_future_active_subscription()),
            pending: None,
            is_stacked: false,
        };

        let parsed = build(&summary).expect("must not fail on bad reset timestamp");
        assert!(
            parsed.is_subscription_active(),
            "subscription active in 2099 must still report active"
        );
    }

    #[test]
    fn malformed_valid_until_utc_fails_summary() {
        let mut summary = base_summary();
        let mut sub = far_future_active_subscription();
        sub.valid_until_utc = "not a date at all".into();
        summary.subscription = NymVpnAccountSummarySubscription {
            is_active: true,
            active: Some(sub),
            pending: None,
            is_stacked: false,
        };

        let err = build(&summary)
            .expect_err("malformed subscription.valid_until_utc must fail the whole summary");
        assert!(
            err.to_string()
                .contains("invalid subscription.valid_until_utc"),
            "expected PayloadError for bad valid_until_utc, got {err:?}"
        );
    }

    #[traced_test]
    #[test]
    fn warn_emitted_when_resets_on_utc_malformed() {
        let mut summary = base_summary();
        summary.fair_usage.resetsOnUtc = Some("not a date".into());
        let _ = build(&summary).unwrap();
        assert!(
            logs_contain("failed to parse fair_usage.resetsOnUtc"),
            "soft-fail of resetsOnUtc must emit a tracing::warn!"
        );
    }

    #[traced_test]
    #[test]
    fn warn_emitted_when_subscription_valid_until_malformed() {
        let mut summary = base_summary();
        let mut sub = far_future_active_subscription();
        sub.valid_until_utc = "not a date".into();
        summary.subscription = NymVpnAccountSummarySubscription {
            is_active: true,
            active: Some(sub),
            pending: None,
            is_stacked: false,
        };
        let _ = build(&summary).expect_err("malformed valid_until_utc");
        assert!(
            logs_contain("failed to parse subscription.valid_until_utc"),
            "parse attempt for subscription.valid_until_utc must emit a tracing::warn!"
        );
    }

    #[test]
    fn malformed_pending_subscription_fails_summary() {
        let mut summary = base_summary();
        let mut sub = far_future_active_subscription();
        sub.valid_until_utc = "not a date at all".into();
        summary.subscription = NymVpnAccountSummarySubscription {
            is_active: false,
            active: None,
            pending: Some(sub),
            is_stacked: false,
        };

        let err = build(&summary).expect_err(
            "malformed pending subscription.valid_until_utc must fail the whole summary",
        );
        assert!(
            err.to_string()
                .contains("invalid subscription.valid_until_utc"),
            "expected PayloadError for bad valid_until_utc, got {err:?}"
        );
    }

    #[test]
    fn well_formed_pending_subscription_parses() {
        // Counterpart that pins the happy path: a valid pending sub must
        // round-trip and surface as Pending status (not Active).
        let mut summary = base_summary();
        summary.subscription = NymVpnAccountSummarySubscription {
            is_active: false,
            active: None,
            pending: Some(far_future_active_subscription()),
            is_stacked: false,
        };

        let parsed = build(&summary).expect("must parse");
        assert!(
            !parsed.is_subscription_active(),
            "Pending status must not report as active"
        );
        let sub = parsed
            .subscription
            .expect("pending subscription must be kept");
        assert!(matches!(sub.status, NymVpnSubscriptionStatus::Pending));
    }
}

#[cfg(test)]
mod fair_usage_left_semantics_tests {
    use super::*;

    fn active_subscription_valid_for_days(days: i64) -> Subscription {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        Subscription {
            status: NymVpnSubscriptionStatus::Active,
            subscription: NymVpnSubscription {
                created_on_utc: "2024-01-01T00:00:00Z".into(),
                last_updated_utc: "2024-01-01T00:00:00Z".into(),
                id: "sub_test".into(),
                valid_from_utc: now - 86_400,
                valid_until_utc: now + days * 86_400,
                status: "active".into(),
                kind: NymVpnSubscriptionKind::OneMonth,
                is_recurring: false,
            },
        }
    }

    fn expired_active_subscription() -> Subscription {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        Subscription {
            status: NymVpnSubscriptionStatus::Active,
            subscription: NymVpnSubscription {
                created_on_utc: "2024-01-01T00:00:00Z".into(),
                last_updated_utc: "2024-01-01T00:00:00Z".into(),
                id: "sub_expired".into(),
                valid_from_utc: now - 2 * 86_400,
                valid_until_utc: now - 1,
                status: "active".into(),
                kind: NymVpnSubscriptionKind::OneMonth,
                is_recurring: false,
            },
        }
    }

    fn pending_subscription() -> Subscription {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        Subscription {
            status: NymVpnSubscriptionStatus::Pending,
            subscription: NymVpnSubscription {
                created_on_utc: "2024-01-01T00:00:00Z".into(),
                last_updated_utc: "2024-01-01T00:00:00Z".into(),
                id: "sub_pending".into(),
                valid_from_utc: now - 86_400,
                valid_until_utc: now + 365 * 86_400,
                status: "pending".into(),
                kind: NymVpnSubscriptionKind::OneMonth,
                is_recurring: false,
            },
        }
    }

    fn bare_summary(
        subscription: Option<Subscription>,
        traffic_limit_gb: u64,
        traffic_used_gb: u64,
    ) -> VpnAccountSummary {
        VpnAccountSummary {
            traffic_used_gb,
            traffic_limit_gb,
            traffic_reset_time: None,
            fair_usage_data_unavailable: false,
            account_addr: "n1test".into(),
            canonical_account_addr: None,
            auth_methods: vec![],
            account_mode: None,
            subscription,
            is_subscription_stacked: false,
            account_status: VpnAccountStatus::Active,
            remaining_devices: 10,
            is_device_active: false,
            time_synced: true,
            stale: false,
            last_synced_utc: OffsetDateTime::now_utc(),
        }
    }

    #[test]
    fn fair_usage_left_false_without_active_subscription_even_when_zeros() {
        let s = bare_summary(None, 0, 0);
        assert!(
            !s.fair_usage_left(),
            "no subscription: ambiguous zeros must not mean quota available"
        );
    }

    #[test]
    fn fair_usage_left_false_when_subscription_pending_even_with_positive_limits() {
        let s = bare_summary(Some(pending_subscription()), 2000, 0);
        assert!(!s.fair_usage_left());
    }

    #[test]
    fn fair_usage_left_false_when_active_and_limit_zero_with_reliable_usage() {
        let s = bare_summary(Some(active_subscription_valid_for_days(30)), 0, 0);
        assert!(
            !s.fair_usage_left(),
            "limit 0 with reliable fair-usage data means no quota remaining"
        );
    }

    #[test]
    fn fair_usage_left_false_when_active_and_exhausted() {
        let s = bare_summary(Some(active_subscription_valid_for_days(30)), 2000, 2000);
        assert!(!s.fair_usage_left());
    }

    #[test]
    fn fair_usage_left_false_when_active_and_over_limit() {
        let s = bare_summary(Some(active_subscription_valid_for_days(30)), 2000, 2100);
        assert!(!s.fair_usage_left());
    }

    #[test]
    fn fair_usage_left_true_when_active_and_under_cap() {
        let s = bare_summary(Some(active_subscription_valid_for_days(30)), 2000, 100);
        assert!(s.fair_usage_left());
    }

    #[test]
    fn is_stale_true_after_traffic_reset_time_even_when_recently_synced() {
        let now = OffsetDateTime::from_unix_timestamp(1_700_000_000).expect("valid timestamp");
        let reset_time = now - time::Duration::seconds(1);
        let mut s = bare_summary(Some(active_subscription_valid_for_days(30)), 2000, 2000);
        s.last_synced_utc = reset_time - time::Duration::seconds(1);
        s.traffic_reset_time = Some(reset_time);

        assert!(s.is_stale_at(now, Duration::from_secs(24 * 60 * 60)));
    }

    #[test]
    fn is_stale_false_after_traffic_reset_time_when_synced_after_reset() {
        let now = OffsetDateTime::from_unix_timestamp(1_700_000_000).expect("valid timestamp");
        let reset_time = now - time::Duration::seconds(1);
        let mut s = bare_summary(Some(active_subscription_valid_for_days(30)), 2000, 2000);
        s.last_synced_utc = now;
        s.traffic_reset_time = Some(reset_time);

        assert!(!s.is_stale_at(now, Duration::from_secs(24 * 60 * 60)));
    }

    #[test]
    fn is_stale_false_before_traffic_reset_time_when_recently_synced() {
        let now = OffsetDateTime::from_unix_timestamp(1_700_000_000).expect("valid timestamp");
        let mut s = bare_summary(Some(active_subscription_valid_for_days(30)), 2000, 100);
        s.last_synced_utc = now;
        s.traffic_reset_time = Some(now + time::Duration::hours(1));

        assert!(!s.is_stale_at(now, Duration::from_secs(24 * 60 * 60)));
    }

    #[test]
    fn is_stale_false_when_depleted_but_reset_still_in_future() {
        let now = OffsetDateTime::from_unix_timestamp(1_700_000_000).expect("valid timestamp");
        let mut s = bare_summary(Some(active_subscription_valid_for_days(30)), 2000, 2000);
        s.last_synced_utc = now;
        s.traffic_reset_time = Some(now + time::Duration::hours(1));

        assert!(!s.is_stale_at(now, Duration::from_secs(24 * 60 * 60)));
    }

    #[test]
    fn fair_usage_left_false_when_status_active_but_valid_until_expired() {
        // status==Active but valid_until_utc is in the past (clock skew / API inconsistency).
        // fair_usage_left() must return false; the ZK_NYM_STATE caller has no prior wire guard.
        let s = bare_summary(Some(expired_active_subscription()), 2000, 0);
        assert!(
            !s.fair_usage_left(),
            "expired valid_until_utc must block zk-nym issuance regardless of status field"
        );
    }

    #[test]
    fn fair_usage_left_true_when_data_unavailable_and_sub_active() {
        // API returned dataUnavailable:true (database down). Must fail-open so users with
        // a valid subscription are not blocked by an infrastructure outage.
        let mut s = bare_summary(Some(active_subscription_valid_for_days(30)), 0, 0);
        s.fair_usage_data_unavailable = true;
        assert!(
            s.fair_usage_left(),
            "data_unavailable with active sub must fail-open, not raise BandwidthExceeded"
        );
    }

    #[test]
    fn fair_usage_left_false_when_data_unavailable_but_no_sub() {
        // Edge case: data_unavailable=true but no subscription → still blocked.
        let mut s = bare_summary(None, 0, 0);
        s.fair_usage_data_unavailable = true;
        assert!(
            !s.fair_usage_left(),
            "data_unavailable without a subscription must not grant access"
        );
    }
}
