// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_account_controller::{
    shared_state::{
        AccountRegistered, AccountState, AccountSummary, DeviceState, DeviceSummary, FairUsage,
        MnemonicState, RegisterDeviceResult, RequestZkNymResult, SubscriptionState,
    },
    AccountStateSummary,
};

use crate::{
    get_account_state_response::{
        account_state_summary::{
            account_summary::{
                AccountState as ProtoAccountState, DeviceSummary as ProtoDeviceSummary,
                FairUsageState as ProtoFairUsageState, SubscriptionState as ProtoSubscriptionState,
            },
            AccountRegistered as ProtoAccountRegistered, AccountSummary as ProtoAccountSummary,
            DeviceState as ProtoDeviceState, MnemonicState as ProtoMnemonicState,
        },
        AccountStateSummary as ProtoAccountStateSummary,
    },
    RegisterDeviceError as ProtoRegisterDeviceError,
    RegisterDeviceResult as ProtoRegisterDeviceResult, RequestZkNymError as ProtoRequestZkNymError,
    RequestZkNymResult as ProtoRequestZkNymResult, RequestZkNymSuccess as ProtoRequestZkNymSuccess,
};

impl From<MnemonicState> for ProtoMnemonicState {
    fn from(mnemonic: MnemonicState) -> Self {
        match mnemonic {
            MnemonicState::Stored { .. } => Self::Stored,
            MnemonicState::NotStored => Self::NotStored,
        }
    }
}

impl From<AccountRegistered> for ProtoAccountRegistered {
    fn from(account_registered: AccountRegistered) -> Self {
        match account_registered {
            AccountRegistered::Registered => Self::AccountRegistered,
            AccountRegistered::NotRegistered => Self::AccountNotRegistered,
        }
    }
}

impl From<AccountState> for ProtoAccountState {
    fn from(account: AccountState) -> Self {
        match account {
            AccountState::Inactive => Self::Inactive,
            AccountState::Active => Self::Active,
            AccountState::DeleteMe => Self::DeleteMe,
        }
    }
}

impl From<SubscriptionState> for ProtoSubscriptionState {
    fn from(subscription: SubscriptionState) -> Self {
        match subscription {
            SubscriptionState::NotActive => Self::NotRegistered,
            SubscriptionState::Pending => Self::Pending,
            SubscriptionState::Active => Self::Active,
            SubscriptionState::Complete => Self::Complete,
        }
    }
}

impl From<DeviceSummary> for ProtoDeviceSummary {
    fn from(device_summary: DeviceSummary) -> Self {
        Self {
            active: device_summary.active,
            max: device_summary.max,
            remaining: device_summary.remaining,
        }
    }
}

impl From<FairUsage> for ProtoFairUsageState {
    fn from(fair_usage: FairUsage) -> Self {
        Self {
            used_gb: fair_usage.used_gb,
            limit_gb: fair_usage.limit_gb,
            resets_on_utc: fair_usage.resets_on_utc,
        }
    }
}

impl From<AccountSummary> for ProtoAccountSummary {
    fn from(account_summary: AccountSummary) -> Self {
        Self {
            account: ProtoAccountState::from(account_summary.account) as i32,
            subscription: ProtoSubscriptionState::from(account_summary.subscription) as i32,
            device_summary: Some(ProtoDeviceSummary::from(account_summary.device_summary)),
            fair_usage: Some(ProtoFairUsageState::from(account_summary.fair_usage)),
        }
    }
}

impl From<DeviceState> for ProtoDeviceState {
    fn from(device: DeviceState) -> Self {
        match device {
            DeviceState::NotRegistered => Self::NotRegistered,
            DeviceState::Inactive => Self::Inactive,
            DeviceState::Active => Self::Active,
            DeviceState::DeleteMe => Self::DeleteMe,
        }
    }
}

impl From<RegisterDeviceResult> for ProtoRegisterDeviceResult {
    fn from(device_registration: RegisterDeviceResult) -> Self {
        match device_registration {
            RegisterDeviceResult::InProgress => Self {
                kind: crate::register_device_result::RegisterDeviceResultType::InProgress as i32,
                error: None,
            },
            RegisterDeviceResult::Success => Self {
                kind: crate::register_device_result::RegisterDeviceResultType::Success as i32,
                error: None,
            },
            RegisterDeviceResult::Failed(err) => Self {
                kind: crate::register_device_result::RegisterDeviceResultType::Failed as i32,
                error: Some(ProtoRegisterDeviceError::from(err)),
            },
        }
    }
}

impl From<RequestZkNymResult> for ProtoRequestZkNymResult {
    fn from(zk_nym_request: RequestZkNymResult) -> Self {
        match zk_nym_request {
            RequestZkNymResult::InProgress => Self {
                kind: crate::request_zk_nym_result::RequestZkNymResultType::InProgress as i32,
                successes: Default::default(),
                failures: Default::default(),
            },
            RequestZkNymResult::Done {
                successes,
                failures,
            } => Self {
                kind: crate::request_zk_nym_result::RequestZkNymResultType::Done as i32,
                successes: successes
                    .into_iter()
                    .map(ProtoRequestZkNymSuccess::from)
                    .collect(),
                failures: failures
                    .into_iter()
                    .map(ProtoRequestZkNymError::from)
                    .collect(),
            },
            RequestZkNymResult::Error(e) => Self {
                kind: crate::request_zk_nym_result::RequestZkNymResultType::Error as i32,
                successes: Default::default(),
                failures: vec![ProtoRequestZkNymError::from(e)],
            },
        }
    }
}

impl From<AccountStateSummary> for ProtoAccountStateSummary {
    fn from(state: AccountStateSummary) -> Self {
        Self {
            mnemonic: state
                .mnemonic
                .map(ProtoMnemonicState::from)
                .map(|m| m as i32),
            account_registered: state
                .account_registered
                .map(ProtoAccountRegistered::from)
                .map(|m| m as i32),
            account_summary: state.account_summary.map(ProtoAccountSummary::from),
            device: state.device.map(ProtoDeviceState::from).map(|m| m as i32),
            register_device_result: state
                .register_device_result
                .map(ProtoRegisterDeviceResult::from),
            request_zk_nym_result: state
                .request_zk_nym_result
                .map(ProtoRequestZkNymResult::from),
        }
    }
}
