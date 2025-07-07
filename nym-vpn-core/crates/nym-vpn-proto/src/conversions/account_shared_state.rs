// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_account_controller::{
    AccountStateSummary,
    shared_state::{
        AccountRegistered, AccountState, AccountSummary, DeviceState, DeviceSummary, FairUsage,
        MnemonicState, RegisterDeviceResult, RequestZkNymResult, SubscriptionState,
    },
};

use crate::proto;

impl From<MnemonicState>
    for proto::get_account_state_response::account_state_summary::MnemonicState
{
    fn from(mnemonic: MnemonicState) -> Self {
        match mnemonic {
            MnemonicState::Stored { .. } => Self::Stored,
            MnemonicState::NotStored => Self::NotStored,
        }
    }
}

impl From<AccountRegistered>
    for proto::get_account_state_response::account_state_summary::AccountRegistered
{
    fn from(account_registered: AccountRegistered) -> Self {
        match account_registered {
            AccountRegistered::Registered => Self::AccountRegistered,
            AccountRegistered::NotRegistered => Self::AccountNotRegistered,
        }
    }
}

impl From<AccountState>
    for proto::get_account_state_response::account_state_summary::account_summary::AccountState
{
    fn from(account: AccountState) -> Self {
        match account {
            AccountState::Inactive => Self::Inactive,
            AccountState::Active => Self::Active,
            AccountState::DeleteMe => Self::DeleteMe,
        }
    }
}

impl From<SubscriptionState>
    for proto::get_account_state_response::account_state_summary::account_summary::SubscriptionState
{
    fn from(subscription: SubscriptionState) -> Self {
        match subscription {
            SubscriptionState::NotActive => Self::NotRegistered,
            SubscriptionState::Pending => Self::Pending,
            SubscriptionState::Active => Self::Active,
            SubscriptionState::Complete => Self::Complete,
        }
    }
}

impl From<DeviceSummary>
    for proto::get_account_state_response::account_state_summary::account_summary::DeviceSummary
{
    fn from(device_summary: DeviceSummary) -> Self {
        Self {
            active: device_summary.active,
            max: device_summary.max,
            remaining: device_summary.remaining,
        }
    }
}

impl From<FairUsage>
    for proto::get_account_state_response::account_state_summary::account_summary::FairUsageState
{
    fn from(fair_usage: FairUsage) -> Self {
        Self {
            used_gb: fair_usage.used_gb,
            limit_gb: fair_usage.limit_gb,
            resets_on_utc: fair_usage.resets_on_utc,
        }
    }
}

impl From<AccountSummary>
    for proto::get_account_state_response::account_state_summary::AccountSummary
{
    fn from(account_summary: AccountSummary) -> Self {
        use proto::get_account_state_response::account_state_summary::account_summary::{
            AccountState, DeviceSummary, FairUsageState,
        };
        Self {
            account: AccountState::from(account_summary.account) as i32,
            subscription: SubscriptionState::from(account_summary.subscription) as i32,
            device_summary: Some(DeviceSummary::from(account_summary.device_summary)),
            fair_usage: Some(FairUsageState::from(account_summary.fair_usage)),
        }
    }
}

impl From<DeviceState> for proto::get_account_state_response::account_state_summary::DeviceState {
    fn from(device: DeviceState) -> Self {
        match device {
            DeviceState::NotRegistered => Self::NotRegistered,
            DeviceState::Inactive => Self::Inactive,
            DeviceState::Active => Self::Active,
            DeviceState::DeleteMe => Self::DeleteMe,
        }
    }
}

impl From<RegisterDeviceResult> for proto::RegisterDeviceResult {
    fn from(device_registration: RegisterDeviceResult) -> Self {
        match device_registration {
            RegisterDeviceResult::InProgress => Self {
                kind: proto::register_device_result::RegisterDeviceResultType::InProgress as i32,
                error: None,
            },
            RegisterDeviceResult::Success => Self {
                kind: proto::register_device_result::RegisterDeviceResultType::Success as i32,
                error: None,
            },
            RegisterDeviceResult::Failed(err) => Self {
                kind: proto::register_device_result::RegisterDeviceResultType::Failed as i32,
                error: Some(proto::RegisterDeviceError::from(err)),
            },
        }
    }
}

impl From<RequestZkNymResult> for proto::RequestZkNymResult {
    fn from(zk_nym_request: RequestZkNymResult) -> Self {
        match zk_nym_request {
            RequestZkNymResult::InProgress => Self {
                kind: proto::request_zk_nym_result::RequestZkNymResultType::InProgress as i32,
                successes: Default::default(),
                failures: Default::default(),
            },
            RequestZkNymResult::Done {
                successes,
                failures,
            } => Self {
                kind: proto::request_zk_nym_result::RequestZkNymResultType::Done as i32,
                successes: successes
                    .into_iter()
                    .map(proto::RequestZkNymSuccess::from)
                    .collect(),
                failures: failures
                    .into_iter()
                    .map(proto::RequestZkNymError::from)
                    .collect(),
            },
            RequestZkNymResult::Error(e) => Self {
                kind: proto::request_zk_nym_result::RequestZkNymResultType::Error as i32,
                successes: Default::default(),
                failures: vec![proto::RequestZkNymError::from(e)],
            },
        }
    }
}

impl From<AccountStateSummary> for proto::get_account_state_response::AccountStateSummary {
    fn from(state: AccountStateSummary) -> Self {
        use proto::get_account_state_response::account_state_summary::{
            AccountRegistered, AccountSummary, DeviceState, MnemonicState,
        };

        Self {
            mnemonic: state.mnemonic.map(MnemonicState::from).map(|m| m as i32),
            account_registered: state
                .account_registered
                .map(AccountRegistered::from)
                .map(|m| m as i32),
            account_summary: state.account_summary.map(AccountSummary::from),
            device: state.device.map(DeviceState::from).map(|m| m as i32),
            register_device_result: state
                .register_device_result
                .map(proto::RegisterDeviceResult::from),
            request_zk_nym_result: state
                .request_zk_nym_result
                .map(proto::RequestZkNymResult::from),
        }
    }
}
