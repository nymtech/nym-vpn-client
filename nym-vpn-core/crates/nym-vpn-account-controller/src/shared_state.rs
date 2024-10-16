// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use nym_vpn_api_client::response::{
    NymVpnAccountStatusResponse, NymVpnAccountSummarySubscription, NymVpnDeviceStatus,
};

#[derive(Clone)]
pub struct SharedAccountState {
    inner: Arc<tokio::sync::Mutex<AccountState>>,
}

impl SharedAccountState {
    pub(crate) fn new() -> Self {
        SharedAccountState {
            inner: Arc::new(tokio::sync::Mutex::new(AccountState::default())),
        }
    }

    async fn get(&self) -> AccountState {
        self.inner.lock().await.clone()
    }

    #[allow(unused)]
    pub(crate) async fn is_ready_to_connect(&self) -> bool {
        let state = self.get().await;
        state.mnemonic == Some(MnemonicState::Stored)
            && state.account == Some(RemoteAccountState::Active)
            && state.subscription == Some(SubscriptionState::Subscribed)
            && state.device == Some(DeviceState::Active)
    }

    pub(crate) async fn is_ready_to_register_device(&self) -> bool {
        let state = self.get().await;
        state.mnemonic == Some(MnemonicState::Stored)
            && state.account == Some(RemoteAccountState::Active)
            && state.device == Some(DeviceState::NotRegistered)
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
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct AccountState {
    mnemonic: Option<MnemonicState>,
    account: Option<RemoteAccountState>,
    subscription: Option<SubscriptionState>,
    device: Option<DeviceState>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum MnemonicState {
    NotStored,
    Stored,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum RemoteAccountState {
    NotRegistered,
    Inactive,
    Active,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SubscriptionState {
    NotSubscribed,
    Subscribed,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum DeviceState {
    NotRegistered,
    Inactive,
    Active,
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
