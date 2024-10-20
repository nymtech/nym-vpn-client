// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{fmt, sync::Arc};

use nym_vpn_api_client::response::{
    NymVpnAccountStatusResponse, NymVpnAccountSummarySubscription, NymVpnDeviceStatus,
};
use serde::Serialize;
use tokio::sync::MutexGuard;

#[derive(Clone)]
pub struct SharedAccountState {
    inner: Arc<tokio::sync::Mutex<AccountState>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ReadyToRegisterDevice {
    Ready,
    NoMnemonicStored,
    RemoteAccountNotActive,
    NoActiveSubscription,
    DeviceAlreadyRegistered,
    DeviceInactive,
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
            ReadyToConnect::Ready => write!(f, "Ready to connect"),
            ReadyToConnect::NoMnemonicStored => write!(f, "No mnemonic stored"),
            ReadyToConnect::AccountNotActive => write!(f, "Account not active"),
            ReadyToConnect::NoActiveSubscription => write!(f, "No active subscription"),
            ReadyToConnect::DeviceNotRegistered => write!(f, "Device not registered"),
            ReadyToConnect::DeviceNotActive => write!(f, "Device not active"),
        }
    }
}

impl SharedAccountState {
    pub(crate) fn new() -> Self {
        SharedAccountState {
            inner: Arc::new(tokio::sync::Mutex::new(AccountState::default())),
        }
    }

    pub async fn lock(&self) -> MutexGuard<'_, AccountState> {
        self.inner.lock().await
    }

    pub async fn is_ready_to_connect(&self) -> ReadyToConnect {
        let state = self.lock().await.clone();
        if state.mnemonic != Some(MnemonicState::Stored) {
            return ReadyToConnect::NoMnemonicStored;
        }
        if state.account != Some(RemoteAccountState::Active) {
            return ReadyToConnect::AccountNotActive;
        }
        if state.subscription != Some(SubscriptionState::Subscribed) {
            return ReadyToConnect::NoActiveSubscription;
        }
        match state.device {
            None => return ReadyToConnect::DeviceNotRegistered,
            Some(DeviceState::NotRegistered) => return ReadyToConnect::DeviceNotRegistered,
            Some(DeviceState::Inactive) => return ReadyToConnect::DeviceNotActive,
            _ => {}
        }
        ReadyToConnect::Ready
    }

    pub(crate) async fn is_ready_to_register_device(&self) -> ReadyToRegisterDevice {
        let state = self.lock().await.clone();
        if state.mnemonic != Some(MnemonicState::Stored) {
            return ReadyToRegisterDevice::NoMnemonicStored;
        }
        if state.account != Some(RemoteAccountState::Active) {
            return ReadyToRegisterDevice::RemoteAccountNotActive;
        }
        if state.subscription != Some(SubscriptionState::Subscribed) {
            return ReadyToRegisterDevice::NoActiveSubscription;
        }
        if state.device == Some(DeviceState::Active) {
            return ReadyToRegisterDevice::DeviceAlreadyRegistered;
        }
        if state.device == Some(DeviceState::Inactive) {
            return ReadyToRegisterDevice::DeviceInactive;
        }
        ReadyToRegisterDevice::Ready
    }

    pub(crate) async fn set_mnemonic(&self, state: MnemonicState) {
        let mut guard = self.inner.lock().await;
        tracing::info!("Setting mnemonic state to {:?}", state);
        guard.mnemonic = Some(state);
    }

    pub(crate) async fn set_account(&self, state: RemoteAccountState) {
        let mut guard = self.inner.lock().await;
        tracing::info!("Setting account state to {:?}", state);
        guard.account = Some(state);
    }

    pub(crate) async fn set_subscription(&self, state: SubscriptionState) {
        let mut guard = self.inner.lock().await;
        tracing::info!("Setting subscription state to {:?}", state);
        guard.subscription = Some(state);
    }

    pub(crate) async fn set_device(&self, state: DeviceState) {
        let mut guard = self.inner.lock().await;
        tracing::info!("Setting device state to {:?}", state);
        guard.device = Some(state);
    }

    pub(crate) async fn set_pending_zk_nym(&self, pending: bool) {
        let mut guard = self.inner.lock().await;
        if guard.pending_zk_nym != pending {
            tracing::info!("Setting pending zk-nym to {}", pending);
            guard.pending_zk_nym = pending;
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct AccountState {
    // The locally stored recovery phrase that is deeply tied to the account
    mnemonic: Option<MnemonicState>,

    // The state of the account on the remote server
    account: Option<RemoteAccountState>,

    // The state of the subscription on the remote server
    subscription: Option<SubscriptionState>,

    // The state of the device on the remote server
    device: Option<DeviceState>,

    // If there are any pending zk-nym requests. This is not stopping from trying to connect, but
    // it might be useful to hold off.
    pending_zk_nym: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) enum MnemonicState {
    NotStored,
    Stored,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) enum RemoteAccountState {
    NotRegistered,
    Inactive,
    Active,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) enum SubscriptionState {
    NotSubscribed,
    Subscribed,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) enum DeviceState {
    NotRegistered,
    Inactive,
    Active,
}

impl fmt::Display for AccountState {
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

impl From<NymVpnAccountStatusResponse> for RemoteAccountState {
    fn from(status: NymVpnAccountStatusResponse) -> Self {
        match status {
            NymVpnAccountStatusResponse::Active => RemoteAccountState::Active,
            _ => RemoteAccountState::Inactive,
        }
    }
}

impl From<NymVpnAccountSummarySubscription> for SubscriptionState {
    fn from(subscription: NymVpnAccountSummarySubscription) -> Self {
        if subscription.is_active {
            SubscriptionState::Subscribed
        } else {
            SubscriptionState::NotSubscribed
        }
    }
}

impl From<NymVpnDeviceStatus> for DeviceState {
    fn from(status: NymVpnDeviceStatus) -> Self {
        match status {
            NymVpnDeviceStatus::Active => DeviceState::Active,
            _ => DeviceState::Inactive,
        }
    }
}
