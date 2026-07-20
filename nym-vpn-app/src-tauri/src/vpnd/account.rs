use crate::error::BackendError;

use nym_vpn_lib_types as lib;
use serde::Serialize;
use tracing::{debug, instrument};
use tracing::{info, warn};
use ts_rs::TS;

#[derive(strum::AsRefStr, Default, Serialize, Clone, Debug, TS)]
#[ts(export, export_to = "tauri.ts", rename = "TAccountState")]
#[serde(rename_all = "kebab-case")]
pub enum AccountState {
    #[default]
    Ready,
    LoggedOut,
    Syncing,
    Offline,
    Decentralised,
    BandwidthExceeded,
    StatusNotActive,
    NoSubscription,
    PendingSubscription,
    MaxDeviceReached,
    Error(BackendError),
}

impl AccountState {
    pub fn from_lib(state: lib::AccountControllerState) -> AccountState {
        match state {
            lib::AccountControllerState::LoggedOut => AccountState::LoggedOut,
            lib::AccountControllerState::PendingSubscription => AccountState::PendingSubscription,
            lib::AccountControllerState::Syncing => AccountState::Syncing,
            lib::AccountControllerState::ReadyToConnect => AccountState::Ready,
            lib::AccountControllerState::Decentralised => AccountState::Decentralised,
            lib::AccountControllerState::Offline => AccountState::Offline,
            lib::AccountControllerState::Error(error) => match error {
                lib::AccountControllerErrorStateReason::BandwidthExceeded {
                    context: _, /* TODO */
                } => AccountState::BandwidthExceeded,
                lib::AccountControllerErrorStateReason::AccountStatusNotActive {
                    status: _, /* TODO */
                } => AccountState::StatusNotActive,
                lib::AccountControllerErrorStateReason::InactiveSubscription => {
                    AccountState::NoSubscription
                }
                lib::AccountControllerErrorStateReason::MaxDeviceReached => {
                    AccountState::MaxDeviceReached
                }
                _ => AccountState::Error(error.into()),
            },
        }
    }
}

#[instrument]
pub fn log_account_state(state: &lib::AccountControllerState) {
    match state {
        lib::AccountControllerState::Error(e) => match e {
            lib::AccountControllerErrorStateReason::Storage { context, details } => {
                warn!("account state error: {e:?}, context: {context}, details: {details}")
            }
            lib::AccountControllerErrorStateReason::ApiFailure { context, details }
            | lib::AccountControllerErrorStateReason::Internal { context, details } => {
                warn!("account state error: {e:?}, context: {context}, details: {details}",)
            }
            lib::AccountControllerErrorStateReason::DeviceTimeDesynced => {
                warn!("account state error: {e:?}")
            }
            _ => info!("account state error: {e:?}"),
        },
        _ => debug!("account state: [{state:?}]"),
    }
}

#[derive(Serialize, Clone, Debug, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts", rename = "TAccountMode")]
#[serde(rename_all = "kebab-case")]
pub enum StoredAccountMode {
    Privy,
    Decentralised,
    Api,
}

impl From<lib::StoredAccountMode> for StoredAccountMode {
    fn from(mode: lib::StoredAccountMode) -> Self {
        match mode {
            lib::StoredAccountMode::Privy => StoredAccountMode::Privy,
            lib::StoredAccountMode::Decentralised => StoredAccountMode::Decentralised,
            lib::StoredAccountMode::Api => StoredAccountMode::Api,
        }
    }
}

#[derive(Serialize, Clone, Debug, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts", rename = "TSubscriptionStatus")]
#[serde(rename_all = "camelCase")]
pub enum SubscriptionStatus {
    Pending,
    Active,
}

impl From<lib::NymVpnSubscriptionStatus> for SubscriptionStatus {
    fn from(status: lib::NymVpnSubscriptionStatus) -> Self {
        match status {
            lib::NymVpnSubscriptionStatus::Pending => SubscriptionStatus::Pending,
            lib::NymVpnSubscriptionStatus::Active => SubscriptionStatus::Active,
        }
    }
}

#[derive(Serialize, Clone, Debug, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts", rename = "TNymVpnSubscription")]
#[serde(rename_all = "camelCase")]
pub struct NymVpnSubscription {
    pub created_on_utc: String,
    pub last_updated_utc: String,
    pub id: String,
    pub valid_until_utc: i64,
    pub valid_from_utc: i64,
    pub status: String,
    pub kind: VpnSubscriptionKind,
    pub is_recurring: bool,
}

impl From<lib::NymVpnSubscription> for NymVpnSubscription {
    fn from(sub: lib::NymVpnSubscription) -> Self {
        NymVpnSubscription {
            created_on_utc: sub.created_on_utc,
            last_updated_utc: sub.last_updated_utc,
            id: sub.id,
            valid_until_utc: sub.valid_until_utc,
            valid_from_utc: sub.valid_from_utc,
            status: sub.status,
            kind: VpnSubscriptionKind::from(sub.kind),
            is_recurring: sub.is_recurring,
        }
    }
}

#[derive(Serialize, Clone, Debug, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts", rename = "TSubscription")]
#[serde(rename_all = "camelCase")]
pub struct Subscription {
    pub status: SubscriptionStatus,
    pub subscription: NymVpnSubscription,
}

impl From<lib::Subscription> for Subscription {
    fn from(sub: lib::Subscription) -> Self {
        Subscription {
            status: SubscriptionStatus::from(sub.status),
            subscription: NymVpnSubscription::from(sub.subscription),
        }
    }
}

#[derive(Serialize, Clone, Debug, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts", rename = "TAccountSummary")]
#[serde(rename_all = "camelCase")]
pub struct AccountSummary {
    pub traffic_used_gb: u64,
    pub traffic_limit_gb: u64,
    pub traffic_reset_time: Option<i64>,
    pub fair_usage_data_unavailable: bool,
    pub account_addr: String,
    pub canonical_account_addr: Option<String>,
    pub auth_methods: Vec<AuthMethod>,
    pub is_linked: bool,
    pub fair_usage_left: bool,
    pub is_subscription_active: bool,
    pub subscription: Option<Subscription>,
    pub is_subscription_stacked: bool,
}

impl From<lib::VpnAccountSummary> for AccountSummary {
    fn from(summary: lib::VpnAccountSummary) -> Self {
        let is_linked = summary.is_linked();
        let fair_usage_left = summary.fair_usage_left();
        let is_subscription_active = summary.is_subscription_active();

        AccountSummary {
            traffic_used_gb: summary.traffic_used_gb,
            traffic_limit_gb: summary.traffic_limit_gb,
            traffic_reset_time: summary.traffic_reset_time.map(|dt| dt.unix_timestamp()),
            fair_usage_data_unavailable: summary.fair_usage_data_unavailable,
            account_addr: summary.account_addr,
            canonical_account_addr: summary.canonical_account_addr,
            auth_methods: summary
                .auth_methods
                .into_iter()
                .map(AuthMethod::from)
                .collect(),
            is_linked,
            fair_usage_left,
            is_subscription_active,
            subscription: summary.subscription.map(Subscription::from),
            is_subscription_stacked: summary.is_subscription_stacked,
        }
    }
}

#[derive(Serialize, Clone, Debug, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts", rename = "TAuthMethod")]
#[serde(rename_all = "kebab-case")]
pub struct AuthMethod {
    pub id: String,
    pub pubkey: String,
    pub kind: String,
    pub label: String,
    pub status: VpnAccountStatus,
}

impl From<lib::VpnAccountAuthMethod> for AuthMethod {
    fn from(auth_method: lib::VpnAccountAuthMethod) -> Self {
        AuthMethod {
            id: auth_method.id,
            pubkey: auth_method.pubkey,
            kind: auth_method.kind,
            label: auth_method.label,
            status: auth_method.status.into(),
        }
    }
}

#[derive(Serialize, Clone, Debug, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts", rename = "TVpnAccountStatus")]
#[serde(rename_all = "kebab-case")]
pub enum VpnAccountStatus {
    Active,
    Inactive,
    DeleteMe,
}

impl From<lib::VpnAccountStatus> for VpnAccountStatus {
    fn from(status: lib::VpnAccountStatus) -> Self {
        match status {
            lib::VpnAccountStatus::Active => VpnAccountStatus::Active,
            lib::VpnAccountStatus::Inactive => VpnAccountStatus::Inactive,
            lib::VpnAccountStatus::DeleteMe => VpnAccountStatus::DeleteMe,
        }
    }
}

#[derive(Serialize, Clone, Debug, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts", rename = "TVpnSubscriptionKind")]
#[serde(rename_all = "kebab-case")]
pub enum VpnSubscriptionKind {
    OneMonth,
    OneYear,
    TwoYears,
    Freepass,
    Other(String),
}

impl From<lib::NymVpnSubscriptionKind> for VpnSubscriptionKind {
    fn from(kind: lib::NymVpnSubscriptionKind) -> Self {
        match kind {
            lib::NymVpnSubscriptionKind::OneMonth => VpnSubscriptionKind::OneMonth,
            lib::NymVpnSubscriptionKind::OneYear => VpnSubscriptionKind::OneYear,
            lib::NymVpnSubscriptionKind::TwoYears => VpnSubscriptionKind::TwoYears,
            lib::NymVpnSubscriptionKind::Freepass => VpnSubscriptionKind::Freepass,
            lib::NymVpnSubscriptionKind::Other(value) => VpnSubscriptionKind::Other(value),
        }
    }
}

#[derive(Serialize, Clone, Debug, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts", rename = "TAutologinResponse")]
#[serde(rename_all = "kebab-case")]
pub struct AutologinResponse {
    pub url: String,
    pub pin_code: String,
}

impl From<lib::AutologinResponse> for AutologinResponse {
    fn from(response: lib::AutologinResponse) -> Self {
        AutologinResponse {
            url: response.url,
            pin_code: response.pin_code,
        }
    }
}
