// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{fmt, sync::Arc, time::Duration};

use nym_vpn_api_client::response::{
    NymVpnAccountResponse, NymVpnAccountStatusResponse, NymVpnAccountSummarySubscription,
    NymVpnDeviceStatus, NymVpnSubscriptionStatus,
};
use serde::Serialize;
use tokio::sync::MutexGuard;

#[derive(Clone)]
pub struct SharedAccountState {
    inner: Arc<tokio::sync::Mutex<AccountStateSummary>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ReadyToRegisterDevice {
    Ready,
    NoMnemonicStored,
    AccountNotActive,
    NoActiveSubscription,
    DeviceAlreadyRegistered,
    DeviceInactive,
    DeviceDeleted,
}

impl fmt::Display for ReadyToRegisterDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadyToRegisterDevice::Ready => write!(f, "ready to register device"),
            ReadyToRegisterDevice::NoMnemonicStored => write!(f, "no mnemonic stored"),
            ReadyToRegisterDevice::AccountNotActive => write!(f, "account not active"),
            ReadyToRegisterDevice::NoActiveSubscription => write!(f, "no active subscription"),
            ReadyToRegisterDevice::DeviceAlreadyRegistered => {
                write!(f, "device already registered")
            }
            ReadyToRegisterDevice::DeviceInactive => write!(f, "device inactive"),
            ReadyToRegisterDevice::DeviceDeleted => write!(f, "device marked for deletion"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ReadyToConnect {
    Ready,
    NoMnemonicStored,
    AccountNotActive,
    NoActiveSubscription,
    DeviceNotRegistered,
    DeviceNotActive,
}

impl fmt::Display for ReadyToConnect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadyToConnect::Ready => write!(f, "ready to connect"),
            ReadyToConnect::NoMnemonicStored => write!(f, "no mnemonic stored"),
            ReadyToConnect::AccountNotActive => write!(f, "account not active"),
            ReadyToConnect::NoActiveSubscription => write!(f, "no active subscription"),
            ReadyToConnect::DeviceNotRegistered => write!(f, "device not registered"),
            ReadyToConnect::DeviceNotActive => write!(f, "device not active"),
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

    pub async fn is_ready_to_connect(&self) -> ReadyToConnect {
        self.lock().await.is_ready_now()
    }

    // Wait until the account status has been fetched from the API.
    // Returns:
    //  - Some: is the readyness status,
    //  - None: timeout waiting for the status from the API.
    pub async fn wait_for_ready_to_connect(&self, timeout: Duration) -> Option<ReadyToConnect> {
        tracing::info!("Waiting for account state to be ready to connect");
        let start = tokio::time::Instant::now();
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            if start.elapsed() > timeout {
                tracing::error!("Timed out waiting for account state to be ready to connect");
                return None;
            }
            if let Some(ready_to_connect) = self.lock().await.is_ready() {
                tracing::info!("Account readyness status: {}", ready_to_connect);
                return Some(ready_to_connect);
            }
        }
    }

    pub(crate) async fn is_ready_to_register_device(&self) -> ReadyToRegisterDevice {
        let state = self.lock().await.clone();
        if !state
            .mnemonic
            .map(|m| matches!(m, MnemonicState::Stored { .. }))
            .unwrap_or(false)
        {
            return ReadyToRegisterDevice::NoMnemonicStored;
        }
        if state.account != Some(AccountState::Active) {
            // if state.account.map(|a| !a.is_active()).unwrap_or(false) {
            return ReadyToRegisterDevice::AccountNotActive;
        }
        if state.subscription != Some(SubscriptionState::Active) {
            return ReadyToRegisterDevice::NoActiveSubscription;
        }
        if state.device == Some(DeviceState::Active) {
            return ReadyToRegisterDevice::DeviceAlreadyRegistered;
        }
        if state.device == Some(DeviceState::Inactive) {
            return ReadyToRegisterDevice::DeviceInactive;
        }
        if state.device == Some(DeviceState::DeleteMe) {
            return ReadyToRegisterDevice::DeviceDeleted;
        }
        ReadyToRegisterDevice::Ready
    }

    pub(crate) async fn set_mnemonic(&self, state: MnemonicState) {
        let mut guard = self.inner.lock().await;
        if guard.mnemonic.as_ref() != Some(&state) {
            tracing::info!("Setting mnemonic state to {:?}", state);
        }
        guard.mnemonic = Some(state);
    }

    pub(crate) async fn set_account(&self, state: AccountState) {
        let mut guard = self.inner.lock().await;
        if guard.account.as_ref() != Some(&state) {
            tracing::info!("Setting account state to {:?}", state);
        }
        guard.account = Some(state);
    }

    pub(crate) async fn set_subscription(&self, state: SubscriptionState) {
        let mut guard = self.inner.lock().await;
        if guard.subscription.as_ref() != Some(&state) {
            tracing::info!("Setting subscription state to {:?}", state);
        }
        guard.subscription = Some(state);
    }

    pub(crate) async fn set_device(&self, state: DeviceState) {
        let mut guard = self.inner.lock().await;
        if guard.device.as_ref() != Some(&state) {
            tracing::info!("Setting device state to {:?}", state);
        }
        guard.device = Some(state);
    }

    pub(crate) async fn set_pending_zk_nym(&self, pending: bool) {
        let mut guard = self.inner.lock().await;
        if guard.pending_zk_nym != pending {
            tracing::debug!("Setting pending zk-nym to {}", pending);
            guard.pending_zk_nym = pending;
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct AccountStateSummary {
    // The locally stored recovery phrase that is deeply tied to the account
    pub mnemonic: Option<MnemonicState>,

    // The state of the account on the remote server
    pub account: Option<AccountState>,

    // The state of the subscription on the remote server
    pub subscription: Option<SubscriptionState>,

    // The state of the device on the remote server
    pub device: Option<DeviceState>,

    // If there are any pending zk-nym requests. This is not stopping from trying to connect, but
    // it might be useful to hold off.
    pub pending_zk_nym: bool,
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
    // The account is not registered on the remote server
    NotRegistered,

    // The account is registered but not active
    Inactive,

    // The account is registered and active
    Active,

    // The account is marked for deletion
    DeleteMe,
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

impl AccountStateSummary {
    pub fn account_id(&self) -> Option<String> {
        match &self.mnemonic {
            Some(MnemonicState::Stored { id }) => Some(id.clone()),
            _ => None,
        }
    }

    // If we are ready right right now.
    fn is_ready_now(&self) -> ReadyToConnect {
        if !self
            .mnemonic
            .as_ref()
            .map(|m| matches!(m, MnemonicState::Stored { .. }))
            .unwrap_or(false)
        {
            return ReadyToConnect::NoMnemonicStored;
        }
        if self.account != Some(AccountState::Active) {
            return ReadyToConnect::AccountNotActive;
        }
        if self.subscription != Some(SubscriptionState::Active) {
            return ReadyToConnect::NoActiveSubscription;
        }
        match self.device {
            None => return ReadyToConnect::DeviceNotRegistered,
            Some(DeviceState::NotRegistered) => return ReadyToConnect::DeviceNotRegistered,
            Some(DeviceState::Inactive) => return ReadyToConnect::DeviceNotActive,
            _ => {}
        }
        ReadyToConnect::Ready
    }

    // If we know if we are ready
    // - Some: we know if we are ready
    // - None: we don't yet know if we are ready
    fn is_ready(&self) -> Option<ReadyToConnect> {
        match self.mnemonic {
            Some(MnemonicState::NotStored) => return Some(ReadyToConnect::NoMnemonicStored),
            Some(MnemonicState::Stored { .. }) => {}
            None => return None,
        }
        match self.account {
            Some(AccountState::NotRegistered) => return Some(ReadyToConnect::AccountNotActive),
            Some(AccountState::Inactive { .. }) => return Some(ReadyToConnect::AccountNotActive),
            Some(AccountState::DeleteMe { .. }) => return Some(ReadyToConnect::AccountNotActive),
            Some(AccountState::Active { .. }) => {}
            None => return None,
        }
        match self.subscription {
            Some(SubscriptionState::NotActive) => {
                return Some(ReadyToConnect::NoActiveSubscription)
            }
            Some(SubscriptionState::Pending) => return Some(ReadyToConnect::NoActiveSubscription),
            Some(SubscriptionState::Complete) => return Some(ReadyToConnect::NoActiveSubscription),
            Some(SubscriptionState::Active) => {}
            None => return None,
        }
        match self.device {
            Some(DeviceState::NotRegistered) => return Some(ReadyToConnect::DeviceNotRegistered),
            Some(DeviceState::Inactive) => return Some(ReadyToConnect::DeviceNotActive),
            Some(DeviceState::DeleteMe) => return Some(ReadyToConnect::DeviceNotActive),
            Some(DeviceState::Active) => {}
            None => return None,
        }
        if self.pending_zk_nym {
            return None;
        }
        Some(ReadyToConnect::Ready)
    }
}

impl fmt::Display for AccountStateSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AccountState {{ Mnemonic: {}, Account: {}, Subscription: {}, Device: {}, Pending ZK-Nym: {} }}",
            debug_or_unknown(self.mnemonic.as_ref()),
            debug_or_unknown(self.account.as_ref()),
            debug_or_unknown(self.subscription.as_ref()),
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

impl From<NymVpnDeviceStatus> for DeviceState {
    fn from(status: NymVpnDeviceStatus) -> Self {
        match status {
            NymVpnDeviceStatus::Active => DeviceState::Active,
            NymVpnDeviceStatus::Inactive => DeviceState::Inactive,
            NymVpnDeviceStatus::DeleteMe => DeviceState::DeleteMe,
        }
    }
}
