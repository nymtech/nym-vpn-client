// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{fmt, sync::Arc};

use nym_vpn_api_client::response::{
    NymVpnAccountResponse, NymVpnAccountStatusResponse, NymVpnAccountSummaryDevices,
    NymVpnAccountSummaryFairUsage, NymVpnAccountSummaryResponse, NymVpnAccountSummarySubscription,
    NymVpnDeviceStatus, NymVpnSubscriptionStatus,
};
use nym_vpn_lib_types::{RegisterDeviceError, RequestZkNymError, RequestZkNymSuccess};
use serde::Serialize;
use tokio::sync::MutexGuard;

#[derive(Clone)]
pub struct SharedAccountState {
    inner: Arc<tokio::sync::Mutex<AccountStateSummary>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ReadyToRegisterDevice {
    Ready,
    InProgress,
    NoMnemonicStored,
    AccountNotSynced,
    AccountNotRegistered,
    AccountNotActive,
    NoActiveSubscription,
    DeviceAlreadyRegistered,
    MaxDevicesReached(u64),
}

impl fmt::Display for ReadyToRegisterDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadyToRegisterDevice::Ready => write!(f, "ready to register device"),
            ReadyToRegisterDevice::InProgress => write!(f, "device registration in progress"),
            ReadyToRegisterDevice::NoMnemonicStored => write!(f, "no mnemonic stored"),
            ReadyToRegisterDevice::AccountNotSynced => write!(f, "account not synced"),
            ReadyToRegisterDevice::AccountNotRegistered => write!(f, "account not registered"),
            ReadyToRegisterDevice::AccountNotActive => write!(f, "account not active"),
            ReadyToRegisterDevice::NoActiveSubscription => write!(f, "no active subscription"),
            ReadyToRegisterDevice::DeviceAlreadyRegistered => {
                write!(f, "device already registered")
            }
            ReadyToRegisterDevice::MaxDevicesReached(max) => {
                write!(f, "maximum number of devices reached: {max}")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ReadyToRequestZkNym {
    Ready,
    InProgress,
    NoMnemonicStored,
    AccountNotSynced,
    AccountNotRegistered,
    AccountNotActive,
    DeviceNotSynced,
    NoActiveSubscription,
    DeviceNotRegistered,
    DeviceNotActive,
}

impl fmt::Display for ReadyToRequestZkNym {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadyToRequestZkNym::Ready => write!(f, "ready to request zk-nym"),
            ReadyToRequestZkNym::InProgress => write!(f, "zk-nym request in progress"),
            ReadyToRequestZkNym::NoMnemonicStored => write!(f, "no mnemonic stored"),
            ReadyToRequestZkNym::AccountNotSynced => write!(f, "account not synced"),
            ReadyToRequestZkNym::AccountNotRegistered => write!(f, "account not registered"),
            ReadyToRequestZkNym::AccountNotActive => write!(f, "account not active"),
            ReadyToRequestZkNym::DeviceNotSynced => write!(f, "device not synced"),
            ReadyToRequestZkNym::NoActiveSubscription => write!(f, "no active subscription"),
            ReadyToRequestZkNym::DeviceNotRegistered => write!(f, "device not registered"),
            ReadyToRequestZkNym::DeviceNotActive => write!(f, "device not active"),
        }
    }
}

impl SharedAccountState {
    pub(crate) fn new(state: MnemonicState) -> Self {
        let mut summary = AccountStateSummary::default();
        tracing::debug!("Initial mnemonic state: {state}");
        summary.mnemonic = Some(state);
        SharedAccountState {
            inner: Arc::new(tokio::sync::Mutex::new(summary)),
        }
    }

    pub async fn lock(&self) -> MutexGuard<'_, AccountStateSummary> {
        self.inner.lock().await
    }

    pub async fn reset(&self) {
        tracing::trace!("Resetting account state");
        let mut guard = self.inner.lock().await;
        *guard = AccountStateSummary::default();
    }

    pub async fn reset_to(&self, state: MnemonicState) {
        tracing::trace!("Resetting account state to {state:?}");
        let mut guard = self.inner.lock().await;
        if guard.mnemonic.as_ref() != Some(&state) {
            tracing::info!("Setting mnemonic state to {state:?}");
        }
        *guard = AccountStateSummary {
            mnemonic: Some(state),
            ..Default::default()
        };
    }

    pub(crate) async fn set_mnemonic(&self, state: MnemonicState) {
        let mut guard = self.inner.lock().await;
        if guard.mnemonic.as_ref() != Some(&state) {
            tracing::info!("Setting mnemonic state to {:?}", state);
        }
        guard.mnemonic = Some(state);
    }

    // Set the account status. We can only promote the status to a more active state, not downgrade
    pub(crate) async fn promote_account_registered(&self, active: AccountRegistered) {
        let mut guard = self.inner.lock().await;

        match guard.account_registered {
            // If the account is already registered, we don't want to change it
            Some(AccountRegistered::Registered) => (),

            // Log only if we are making a change
            Some(ref current) if current != &active => {
                tracing::info!("Setting account to {:?}", active);
                guard.account_registered = Some(active);
            }

            // If there is no change, we don't want to log anything
            Some(_) => (),

            // If there is no existing status set, we can set it
            None => {
                tracing::info!("Setting account to {:?}", active);
                guard.account_registered = Some(active);
            }
        }
    }

    pub(crate) async fn set_account_summary(&self, summary: AccountSummary) {
        let mut guard = self.inner.lock().await;
        if guard.account_summary.as_ref() != Some(&summary) {
            tracing::info!("Setting account summary to {:?}", summary);
        }
        guard.account_summary = Some(summary);
    }

    pub(crate) async fn set_device(&self, state: DeviceState) {
        let mut guard = self.inner.lock().await;
        if guard.device.as_ref() != Some(&state) {
            tracing::info!("Setting device state to {:?}", state);
        }
        guard.device = Some(state);
    }

    pub(crate) async fn set_device_registration(&self, registration: RegisterDeviceResult) {
        let mut guard = self.inner.lock().await;
        if guard.register_device_result.as_ref() != Some(&registration) {
            tracing::debug!("Setting device registration result to {:?}", registration);
        }
        guard.register_device_result = Some(registration);
    }

    pub(crate) async fn set_zk_nym_request(&self, request: RequestZkNymResult) {
        let mut guard = self.inner.lock().await;
        if guard.request_zk_nym_result.as_ref() != Some(&request) {
            tracing::debug!("Setting zk-nym request result to {request:?}");
        }
        guard.request_zk_nym_result = Some(request);
    }

    pub(crate) async fn is_zk_nym_request_in_progress(&self) -> bool {
        self.lock()
            .await
            .request_zk_nym_result
            .as_ref()
            .map(|r| matches!(r, RequestZkNymResult::InProgress))
            .unwrap_or(false)
    }

    pub async fn is_account_stored(&self) -> bool {
        self.lock()
            .await
            .mnemonic
            .clone()
            .map(|m| m.is_stored())
            .unwrap_or(false)
    }

    pub async fn get_account_id(&self) -> Option<String> {
        self.lock().await.mnemonic.clone().and_then(|m| m.id())
    }

    pub(crate) async fn ready_to_register_device(&self) -> ReadyToRegisterDevice {
        self.lock().await.ready_to_register_device()
    }

    pub(crate) async fn ready_to_request_zk_nym(&self) -> ReadyToRequestZkNym {
        self.lock().await.ready_to_request_zk_nym()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AccountStateSummary {
    // The locally stored recovery phrase that is deeply tied to the account
    pub mnemonic: Option<MnemonicState>,

    // If the account is active on nym-vpn-api
    pub account_registered: Option<AccountRegistered>,

    // The summary of the account on nym-vpn-api
    pub account_summary: Option<AccountSummary>,

    // The state of the device as reported by nym-vpn-api
    pub device: Option<DeviceState>,

    // The result of the latest registration attempt, if any
    pub register_device_result: Option<RegisterDeviceResult>,

    // The result of the latest zk-nym request, if any
    pub request_zk_nym_result: Option<RequestZkNymResult>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum AccountRegistered {
    NotRegistered,
    Registered,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AccountSummary {
    pub account: AccountState,
    pub subscription: SubscriptionState,
    pub device_summary: DeviceSummary,
    pub fair_usage: FairUsage,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum MnemonicState {
    // The recovery phrase is not stored locally, or at least not confirmed to be stored
    NotStored,

    // The recovery phrase is stored locally
    Stored { id: String },
}

impl MnemonicState {
    pub fn is_stored(&self) -> bool {
        matches!(self, MnemonicState::Stored { .. })
    }

    pub fn id(&self) -> Option<String> {
        match self {
            MnemonicState::Stored { id } => Some(id.clone()),
            MnemonicState::NotStored => None,
        }
    }
}

impl fmt::Display for MnemonicState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MnemonicState::NotStored => write!(f, "not stored"),
            MnemonicState::Stored { id } => write!(f, "stored with id {id}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AccountState {
    // The account is registered but not active
    Inactive,

    // The account is registered and active
    Active,

    // The account is marked for deletion
    DeleteMe,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceSummary {
    pub active: u64,
    pub max: u64,
    pub remaining: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FairUsage {
    pub used_gb: Option<f64>,
    pub limit_gb: Option<f64>,
    pub resets_on_utc: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SubscriptionState {
    // There is no active subscription
    NotActive,

    // The subscription is pending
    Pending,

    // The subscription is complete
    Complete,

    // The subscription is active
    Active,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeviceState {
    // The device is not registered on the remote server
    NotRegistered,

    // The device is registered but not active
    Inactive,

    // The device is registered and active
    Active,

    // The device is marked for deletion
    DeleteMe,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RegisterDeviceResult {
    // The device registration is in progress
    InProgress,

    // The device registration was successful
    Success,

    // The device registration failed
    Failed(RegisterDeviceError),
}

#[derive(Debug, Clone, PartialEq)]
pub enum RequestZkNymResult {
    // The zk-nym request is in progress
    InProgress,

    // The the last zk-nym request finished
    Done {
        successes: Vec<RequestZkNymSuccess>,
        failures: Vec<RequestZkNymError>,
    },

    // The last zk-nym request failed before any requests were made
    Error(RequestZkNymError),
}

impl From<Vec<Result<RequestZkNymSuccess, RequestZkNymError>>> for RequestZkNymResult {
    fn from(results: Vec<Result<RequestZkNymSuccess, RequestZkNymError>>) -> Self {
        let (successes, failures): (Vec<_>, Vec<_>) = results.into_iter().partition(Result::is_ok);

        let successes = successes.into_iter().map(Result::unwrap).collect();
        let failures = failures.into_iter().map(Result::unwrap_err).collect();

        RequestZkNymResult::Done {
            successes,
            failures,
        }
    }
}

impl From<RequestZkNymError> for RequestZkNymResult {
    fn from(err: RequestZkNymError) -> Self {
        RequestZkNymResult::Error(err)
    }
}

impl AccountStateSummary {
    pub(crate) fn ready_to_register_device(&self) -> ReadyToRegisterDevice {
        match self.device {
            Some(DeviceState::NotRegistered) => {}
            Some(DeviceState::Inactive) => {}
            Some(DeviceState::Active) => return ReadyToRegisterDevice::DeviceAlreadyRegistered,
            Some(DeviceState::DeleteMe) => {}
            None => {}
        }

        match self.register_device_result {
            Some(RegisterDeviceResult::InProgress) => return ReadyToRegisterDevice::InProgress,
            Some(RegisterDeviceResult::Success) => {}
            Some(RegisterDeviceResult::Failed { .. }) => {}
            None => {}
        }

        match self.mnemonic {
            Some(MnemonicState::NotStored) => return ReadyToRegisterDevice::NoMnemonicStored,
            Some(MnemonicState::Stored { .. }) => {}
            None => return ReadyToRegisterDevice::NoMnemonicStored,
        }

        match self.account_registered {
            Some(AccountRegistered::Registered) => {}
            Some(AccountRegistered::NotRegistered) => {
                return ReadyToRegisterDevice::AccountNotRegistered
            }
            None => return ReadyToRegisterDevice::AccountNotSynced,
        }

        if let Some(ref account_summary) = self.account_summary {
            match account_summary.account {
                AccountState::Inactive => return ReadyToRegisterDevice::AccountNotActive,
                AccountState::DeleteMe => return ReadyToRegisterDevice::AccountNotActive,
                AccountState::Active => {}
            }

            if account_summary.device_summary.remaining == 0 {
                return ReadyToRegisterDevice::MaxDevicesReached(
                    account_summary.device_summary.max,
                );
            }

            // We ignore the subscription state, as the device registration is not dependent on it
        }

        ReadyToRegisterDevice::Ready
    }

    pub(crate) fn ready_to_request_zk_nym(&self) -> ReadyToRequestZkNym {
        match self.request_zk_nym_result {
            Some(RequestZkNymResult::InProgress) => return ReadyToRequestZkNym::InProgress,
            Some(RequestZkNymResult::Done { .. }) => {}
            Some(RequestZkNymResult::Error(_)) => {}
            None => {}
        }

        match self.mnemonic {
            Some(MnemonicState::NotStored) => return ReadyToRequestZkNym::NoMnemonicStored,
            Some(MnemonicState::Stored { .. }) => {}
            None => return ReadyToRequestZkNym::NoMnemonicStored,
        }

        match self.account_registered {
            Some(AccountRegistered::Registered) => {}
            Some(AccountRegistered::NotRegistered) => {
                return ReadyToRequestZkNym::AccountNotRegistered
            }
            None => return ReadyToRequestZkNym::AccountNotSynced,
        }

        if let Some(ref account_summary) = self.account_summary {
            match account_summary.account {
                AccountState::Inactive => return ReadyToRequestZkNym::AccountNotActive,
                AccountState::DeleteMe => return ReadyToRequestZkNym::AccountNotActive,
                AccountState::Active => {}
            }

            match account_summary.subscription {
                SubscriptionState::NotActive => return ReadyToRequestZkNym::NoActiveSubscription,
                SubscriptionState::Pending => return ReadyToRequestZkNym::NoActiveSubscription,
                SubscriptionState::Complete => return ReadyToRequestZkNym::NoActiveSubscription,
                SubscriptionState::Active => {}
            }
        }

        match self.device {
            Some(DeviceState::Active) => {}
            Some(DeviceState::NotRegistered) => return ReadyToRequestZkNym::DeviceNotRegistered,
            Some(DeviceState::Inactive) => return ReadyToRequestZkNym::DeviceNotActive,
            Some(DeviceState::DeleteMe) => return ReadyToRequestZkNym::DeviceNotActive,
            None => return ReadyToRequestZkNym::DeviceNotSynced,
        }

        ReadyToRequestZkNym::Ready
    }
}

impl fmt::Display for AccountStateSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AccountState {{ mnemonic: {}, account_registered: {}, account_summary: {}, device: {} }}",
            debug_or_unknown(self.mnemonic.as_ref()),
            debug_or_unknown(self.account_registered.as_ref()),
            debug_or_unknown(self.account_summary.as_ref()),
            debug_or_unknown(self.device.as_ref()),
        )
    }
}

fn debug_or_unknown(state: Option<&impl fmt::Debug>) -> String {
    state
        .map(|s| format!("{:?}", s))
        .unwrap_or_else(|| "Unknown".to_string())
}

impl From<NymVpnAccountResponse> for AccountState {
    fn from(account: NymVpnAccountResponse) -> Self {
        match account.status {
            NymVpnAccountStatusResponse::Active => AccountState::Active,
            NymVpnAccountStatusResponse::Inactive => AccountState::Inactive,
            NymVpnAccountStatusResponse::DeleteMe => AccountState::DeleteMe,
        }
    }
}

impl From<NymVpnAccountSummarySubscription> for SubscriptionState {
    fn from(subscription: NymVpnAccountSummarySubscription) -> Self {
        if subscription.is_active {
            SubscriptionState::Active
        } else if let Some(subscription) = subscription.active {
            match subscription.status {
                NymVpnSubscriptionStatus::Pending => SubscriptionState::Pending,
                NymVpnSubscriptionStatus::Complete => SubscriptionState::Complete,
                NymVpnSubscriptionStatus::Active => SubscriptionState::Active,
            }
        } else {
            SubscriptionState::NotActive
        }
    }
}

impl From<NymVpnAccountSummaryResponse> for AccountSummary {
    fn from(summary: NymVpnAccountSummaryResponse) -> Self {
        Self {
            account: AccountState::from(summary.account),
            subscription: SubscriptionState::from(summary.subscription),
            device_summary: DeviceSummary::from(summary.devices),
            fair_usage: FairUsage::from(summary.fair_usage),
        }
    }
}

impl From<NymVpnAccountSummaryDevices> for DeviceSummary {
    fn from(devices: NymVpnAccountSummaryDevices) -> Self {
        DeviceSummary {
            active: devices.active,
            max: devices.max,
            remaining: devices.remaining,
        }
    }
}

impl From<NymVpnAccountSummaryFairUsage> for FairUsage {
    fn from(fair_usage: NymVpnAccountSummaryFairUsage) -> Self {
        FairUsage {
            used_gb: fair_usage.used_gb,
            limit_gb: fair_usage.limit_gb,
            resets_on_utc: fair_usage.resets_on_utc,
        }
    }
}

impl From<NymVpnDeviceStatus> for DeviceState {
    fn from(status: NymVpnDeviceStatus) -> Self {
        match status {
            NymVpnDeviceStatus::Active => DeviceState::Active,
            NymVpnDeviceStatus::Inactive => DeviceState::Inactive,
            NymVpnDeviceStatus::DeleteMe => DeviceState::DeleteMe,
        }
    }
}
