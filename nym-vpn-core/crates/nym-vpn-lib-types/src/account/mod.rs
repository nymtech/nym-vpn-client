// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod controller_error;
pub mod controller_event;
pub mod controller_state;
pub mod deeplink;
pub mod request_zknym;
pub mod ticketbooks;

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
    #[cfg(target_os = "android")]
    pub purchase_token: String,
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

    pub account_addr: String,
    pub canonical_account_addr: Option<String>,
    pub auth_methods: Vec<VpnAccountAuthMethod>,
    pub account_mode: Option<StoredAccountMode>,
    pub subscription: Option<Subscription>,
    pub is_subscription_stacked: bool,
}

// Exported methods
#[cfg_attr(feature = "uniffi-bindings", uniffi::export)]
#[allow(unused)]
impl VpnAccountSummary {
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
        self.traffic_used_gb != self.traffic_limit_gb
    }

    pub fn is_linked(&self) -> bool {
        self.auth_methods
            .iter()
            .any(|method| method.kind == "privy_secp256k1")
    }
}

#[cfg(feature = "nym-type-conversions")]
impl TryFrom<&nym_vpn_api_client::response::NymVpnAccountSummaryResponse> for VpnAccountSummary {
    type Error = nym_vpn_api_client::error::VpnApiClientError;

    fn try_from(
        value: &nym_vpn_api_client::response::NymVpnAccountSummaryResponse,
    ) -> Result<Self, Self::Error> {
        let traffic_reset_time_str = value.fair_usage.resetsOnUtc.clone();
        let traffic_reset_time = traffic_reset_time_str
            .as_ref()
            .map(|time| OffsetDateTime::parse(time, &time::format_description::well_known::Rfc3339))
            .transpose()
            .map_err(|_| {
                nym_vpn_api_client::error::VpnApiClientError::PayloadError(format!(
                    "invalid fair_usage reset_time_utc time format: {}",
                    traffic_reset_time_str.unwrap()
                ))
            })?;

        let auth_methods = value
            .account
            .auth_methods
            .iter()
            .cloned()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()?;

        let subscription = if let Some(active) = &value.subscription.active {
            Some(Subscription {
                status: NymVpnSubscriptionStatus::Active,
                subscription: NymVpnSubscription::from(active),
            })
        } else {
            value
                .subscription
                .pending
                .as_ref()
                .map(|pending| Subscription {
                    status: NymVpnSubscriptionStatus::Pending,
                    subscription: NymVpnSubscription::from(pending),
                })
        };

        Ok(Self {
            traffic_used_gb: value.fair_usage.usedGB,
            traffic_limit_gb: value.fair_usage.limitGB,
            traffic_reset_time,
            account_addr: value.account.account_addr.clone(),
            canonical_account_addr: value.account.canonical_account_addr.clone(),
            auth_methods,
            account_mode: None,
            subscription,
            is_subscription_stacked: value.subscription.is_stacked,
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
    #[serde(untagged)]
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
impl From<&nym_vpn_api_client::response::NymVpnSubscription> for NymVpnSubscription {
    fn from(value: &nym_vpn_api_client::response::NymVpnSubscription) -> Self {
        let valid_until_utc = {
            let s = value.valid_until_utc.replace(' ', "T");
            OffsetDateTime::parse(&s, &time::format_description::well_known::Rfc3339)
                .map(|t| t.unix_timestamp())
                .unwrap_or(0)
        };
        Self {
            created_on_utc: value.created_on_utc.clone(),
            last_updated_utc: value.last_updated_utc.clone(),
            id: value.id.clone(),
            valid_until_utc,
            valid_from_utc: {
                let s = value.valid_from_utc.replace(' ', "T");
                OffsetDateTime::parse(&s, &time::format_description::well_known::Rfc3339)
                    .map(|t| t.unix_timestamp())
                    .unwrap_or(0)
            },
            status: format!("{:?}", value.status).to_lowercase(),
            kind: value.kind.clone().into(),
            is_recurring: value.is_recurring,
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
        let created = OffsetDateTime::parse(
            &value.created,
            &time::format_description::well_known::Rfc3339,
        )
        .map_err(|_| {
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
pub enum StoredAccountMode {
    Api,
    Decentralised,
    Privy,
}

#[cfg(feature = "nym-type-conversions")]
impl From<nym_vpn_store::types::StoredAccountMode> for StoredAccountMode {
    fn from(value: nym_vpn_store::types::StoredAccountMode) -> Self {
        match value {
            nym_vpn_store::types::StoredAccountMode::Api => StoredAccountMode::Api,
            nym_vpn_store::types::StoredAccountMode::Decentralised => {
                StoredAccountMode::Decentralised
            }
            nym_vpn_store::types::StoredAccountMode::Privy => StoredAccountMode::Privy,
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<StoredAccountMode> for nym_vpn_store::types::StoredAccountMode {
    fn from(value: StoredAccountMode) -> Self {
        match value {
            StoredAccountMode::Api => nym_vpn_store::types::StoredAccountMode::Api,
            StoredAccountMode::Decentralised => {
                nym_vpn_store::types::StoredAccountMode::Decentralised
            }
            StoredAccountMode::Privy => nym_vpn_store::types::StoredAccountMode::Privy,
        }
    }
}
