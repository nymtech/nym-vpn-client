// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{fmt, sync::Arc};

use nym_vpn_api_client::response::{
    NymVpnAccountResponse, NymVpnAccountStatusResponse, NymVpnAccountSummaryDevices,
    NymVpnAccountSummaryFairUsage, NymVpnAccountSummaryResponse, NymVpnAccountSummarySubscription,
    NymVpnDeviceStatus, NymVpnSubscriptionStatus,
};
use serde::Serialize;
use tokio::sync::MutexGuard;

use crate::commands::AccountCommandError;

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
    //DeviceInactive,
    //DeviceDeleted,
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
            // ReadyToRegisterDevice::DeviceInactive => write!(f, "device inactive"),
            // ReadyToRegisterDevice::DeviceDeleted => write!(f, "device marked for deletion"),
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ReadyToConnect {
    Ready,
    NoMnemonicStored,
    AccountNotSynced,
    AccountNotRegistered,
    AccountNotActive,
    NoActiveSubscription,
    DeviceNotRegistered,
    DeviceNotActive,
    DeviceRegistrationFailed {
        message: String,
        message_id: Option<String>,
        code_reference_id: Option<String>,
    },
    //NoCredentialsAvailable,
}

impl fmt::Display for ReadyToConnect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadyToConnect::Ready => write!(f, "ready to connect"),
            ReadyToConnect::NoMnemonicStored => write!(f, "no mnemonic stored"),
            ReadyToConnect::AccountNotSynced => write!(f, "account not synced"),
            ReadyToConnect::AccountNotRegistered => write!(f, "account not registered"),
            ReadyToConnect::AccountNotActive => write!(f, "account not active"),
            ReadyToConnect::NoActiveSubscription => write!(f, "no active subscription"),
            ReadyToConnect::DeviceNotRegistered => write!(f, "device not registered"),
            ReadyToConnect::DeviceNotActive => write!(f, "device not active"),
            ReadyToConnect::DeviceRegistrationFailed { message, .. } => {
                write!(f, "device registration failed: {}", message)
            }
        }
    }
}

impl SharedAccountState {
    pub(crate) fn new() -> Self {
        SharedAccountState {
            inner: Arc::new(tokio::sync::Mutex::new(AccountStateSummary::default())),
        }
    }

    pub async fn lock(&self) -> MutexGuard<'_, AccountStateSummary> {
        self.inner.lock().await
    }

    pub async fn reset(&self) {
        let mut guard = self.inner.lock().await;
        *guard = AccountStateSummary::default();
    }

    pub(crate) async fn set_mnemonic(&self, state: MnemonicState) {
        let mut guard = self.inner.lock().await;
        if guard.mnemonic.as_ref() != Some(&state) {
            tracing::info!("Setting mnemonic state to {:?}", state);
        }
        guard.mnemonic = Some(state);
    }

    pub(crate) async fn set_account_registered(&self, active: AccountRegistered) {
        let mut guard = self.inner.lock().await;
        if guard.account_registered.as_ref() != Some(&active) {
            tracing::info!("Setting account to {:?}", active);
        }
        guard.account_registered = Some(active);
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

    pub(crate) async fn set_device_registration(&self, registration: DeviceRegistration) {
        let mut guard = self.inner.lock().await;
        if guard.device_registration.as_ref() != Some(&registration) {
            tracing::info!("Setting device registration result to {:?}", registration);
        }
        guard.device_registration = Some(registration);
    }

    pub(crate) async fn set_pending_zk_nym(&self, pending: bool) {
        let mut guard = self.inner.lock().await;
        if guard.pending_zk_nym != pending {
            tracing::debug!("Setting pending zk-nym to {}", pending);
            guard.pending_zk_nym = pending;
        }
    }

    pub(crate) async fn is_ready_to_register_device(&self) -> ReadyToRegisterDevice {
        self.lock().await.is_ready_to_register_device()
    }

    pub(crate) async fn is_ready_to_request_zk_nym(&self) -> ReadyToRequestZkNym {
        self.lock().await.is_ready_to_request_zk_nym()
    }

    pub async fn is_ready_to_connect(&self, credential_mode: bool) -> ReadyToConnect {
        self.lock().await.is_ready_to_connect(credential_mode)
    }
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Serialize)]
pub enum WaitForRegistrationError {
    #[error("device registration failed: {message}, message_id: {message_id:?}")]
    Failed {
        message: String,
        message_id: Option<String>,
    },

    #[error("timed out waiting for device registration")]
    Timeout,

    #[error("tried to wait for device registration that hasn't started")]
    TryToWaitForDeviceRegistrationThatHasntStarted,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
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
    pub device_registration: Option<DeviceRegistration>,

    // If there are any pending zk-nym requests. This is not stopping from trying to connect, but
    // it might be useful to hold off.
    pub pending_zk_nym: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum AccountRegistered {
    NotRegistered,
    Registered,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum AccountState {
    // The account is registered but not active
    Inactive,

    // The account is registered and active
    Active,

    // The account is marked for deletion
    DeleteMe,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DeviceSummary {
    pub active: u64,
    pub max: u64,
    pub remaining: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FairUsage {
    pub used_gb: Option<f64>,
    pub limit_gb: Option<f64>,
    pub resets_on_utc: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum DeviceRegistration {
    // The device registration is in progress
    InProgress,

    // The device registration was successful
    Success,

    // The device registration failed
    Failed {
        message: String,
        message_id: Option<String>,
        code_reference_id: Option<String>,
    },
}

impl AccountStateSummary {
    pub(crate) fn is_ready_to_register_device(&self) -> ReadyToRegisterDevice {
        match self.device {
            Some(DeviceState::NotRegistered) => {}
            Some(DeviceState::Inactive) => {}
            Some(DeviceState::Active) => return ReadyToRegisterDevice::DeviceAlreadyRegistered,
            Some(DeviceState::DeleteMe) => {}
            None => {}
        }

        match self.device_registration {
            Some(DeviceRegistration::InProgress) => return ReadyToRegisterDevice::InProgress,
            Some(DeviceRegistration::Success) => {}
            Some(DeviceRegistration::Failed { .. }) => {}
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

            match account_summary.subscription {
                SubscriptionState::NotActive => return ReadyToRegisterDevice::NoActiveSubscription,
                SubscriptionState::Pending => return ReadyToRegisterDevice::NoActiveSubscription,
                SubscriptionState::Complete => return ReadyToRegisterDevice::NoActiveSubscription,
                SubscriptionState::Active => {}
            }

            if account_summary.device_summary.remaining == 0 {
                return ReadyToRegisterDevice::MaxDevicesReached(
                    account_summary.device_summary.max,
                );
            }
        }

        ReadyToRegisterDevice::Ready
    }

    pub(crate) fn is_ready_to_request_zk_nym(&self) -> ReadyToRequestZkNym {
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

        if self.pending_zk_nym {
            return ReadyToRequestZkNym::InProgress;
        }

        ReadyToRequestZkNym::Ready
    }

    // If we are ready right right now.
    pub(crate) fn is_ready_to_connect(&self, credential_mode: bool) -> ReadyToConnect {
        match self.mnemonic {
            Some(MnemonicState::NotStored) => return ReadyToConnect::NoMnemonicStored,
            Some(MnemonicState::Stored { .. }) => {}
            None => return ReadyToConnect::NoMnemonicStored,
        }

        match self.account_registered {
            Some(AccountRegistered::Registered) => {}
            Some(AccountRegistered::NotRegistered) => return ReadyToConnect::AccountNotRegistered,
            None => return ReadyToConnect::AccountNotSynced,
        }

        if let Some(ref account_summary) = self.account_summary {
            match account_summary.account {
                AccountState::Inactive => return ReadyToConnect::AccountNotActive,
                AccountState::DeleteMe => return ReadyToConnect::AccountNotActive,
                AccountState::Active => {}
            }

            match account_summary.subscription {
                SubscriptionState::NotActive => return ReadyToConnect::NoActiveSubscription,
                SubscriptionState::Pending => return ReadyToConnect::NoActiveSubscription,
                SubscriptionState::Complete => return ReadyToConnect::NoActiveSubscription,
                SubscriptionState::Active => {}
            }
        }

        match self.device {
            Some(DeviceState::Active) => {}
            Some(DeviceState::NotRegistered) => return ReadyToConnect::DeviceNotRegistered,
            Some(DeviceState::Inactive) => return ReadyToConnect::DeviceNotActive,
            Some(DeviceState::DeleteMe) => return ReadyToConnect::DeviceNotActive,
            None => return ReadyToConnect::DeviceNotRegistered,
        }

        if credential_mode {
            //if !local_credentials_available {
            //    return ReadyToConnect::NoCredentialsAvailable
            //}
        }

        ReadyToConnect::Ready
    }
}

impl fmt::Display for AccountStateSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AccountState {{ mnemonic: {}, account_registered: {}, account_summary: {}, device: {}, pending_zk_nym: {} }}",
            debug_or_unknown(self.mnemonic.as_ref()),
            debug_or_unknown(self.account_registered.as_ref()),
            debug_or_unknown(self.account_summary.as_ref()),
            debug_or_unknown(self.device.as_ref()),
            self.pending_zk_nym,
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
            tracing::warn!("Subscription state is not active, but no active field is present");
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

impl From<&AccountCommandError> for DeviceRegistration {
    fn from(err: &AccountCommandError) -> Self {
        match err {
            AccountCommandError::UpdateAccountEndpointFailure {
                message,
                message_id,
                code_reference_id,
                base_url: _,
            }
            | AccountCommandError::UpdateDeviceEndpointFailure {
                message,
                message_id,
                code_reference_id,
            }
            | AccountCommandError::RegisterDeviceEndpointFailure {
                message,
                message_id,
                code_reference_id,
            } => DeviceRegistration::Failed {
                message: message.clone(),
                message_id: message_id.clone(),
                code_reference_id: code_reference_id.clone(),
            },
            AccountCommandError::General(_)
            | AccountCommandError::Internal(_)
            | AccountCommandError::NoAccountStored
            | AccountCommandError::NoDeviceStored => DeviceRegistration::Failed {
                message: err.to_string(),
                message_id: None,
                code_reference_id: None,
            },
        }
    }
}
