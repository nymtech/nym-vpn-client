// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::RequestZkNymSuccess;
use nym_vpnd_types::account_state::{
    AccountRegistered, AccountState, AccountStateSummary, AccountSummary, DeviceState,
    DeviceSummary, FairUsage, MnemonicState, RegisterDeviceResult, RequestZkNymErrorReason,
    RequestZkNymResult, SubscriptionState,
};

use crate::{conversions::ConversionError, proto};

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

impl From<proto::get_account_state_response::account_state_summary::MnemonicState>
    for MnemonicState
{
    fn from(
        mnemonic: proto::get_account_state_response::account_state_summary::MnemonicState,
    ) -> Self {
        match mnemonic {
            proto::get_account_state_response::account_state_summary::MnemonicState::Stored => {
                Self::Stored
            }
            proto::get_account_state_response::account_state_summary::MnemonicState::NotStored => {
                Self::NotStored
            }
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

impl From<proto::get_account_state_response::account_state_summary::AccountRegistered>
    for AccountRegistered
{
    fn from(
        account_registered: proto::get_account_state_response::account_state_summary::AccountRegistered,
    ) -> Self {
        match account_registered {
            proto::get_account_state_response::account_state_summary::AccountRegistered::AccountRegistered => Self::Registered,
            proto::get_account_state_response::account_state_summary::AccountRegistered::AccountNotRegistered => Self::NotRegistered,
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

impl From<proto::get_account_state_response::account_state_summary::account_summary::AccountState>
    for AccountState
{
    fn from(
        account:  proto::get_account_state_response::account_state_summary::account_summary::AccountState,
    ) -> Self {
        match account {
             proto::get_account_state_response::account_state_summary::account_summary::AccountState::Inactive => Self::Inactive,
             proto::get_account_state_response::account_state_summary::account_summary::AccountState::Active => Self::Active,
             proto::get_account_state_response::account_state_summary::account_summary::AccountState::DeleteMe => Self::DeleteMe,
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

impl From<proto::get_account_state_response::account_state_summary::account_summary::SubscriptionState>
    for  SubscriptionState
{
    fn from(subscription: proto::get_account_state_response::account_state_summary::account_summary::SubscriptionState) -> Self {
        match subscription {
            proto::get_account_state_response::account_state_summary::account_summary::SubscriptionState::NotRegistered => Self::NotActive,
            proto::get_account_state_response::account_state_summary::account_summary::SubscriptionState::Pending => Self::Pending,
            proto::get_account_state_response::account_state_summary::account_summary::SubscriptionState::Active => Self::Active,
            proto::get_account_state_response::account_state_summary::account_summary::SubscriptionState::Complete => Self::Complete,
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

impl From<proto::get_account_state_response::account_state_summary::account_summary::DeviceSummary>
    for DeviceSummary
{
    fn from(
        device_summary: proto::get_account_state_response::account_state_summary::account_summary::DeviceSummary,
    ) -> Self {
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

impl From<proto::get_account_state_response::account_state_summary::account_summary::FairUsageState>
    for FairUsage
{
    fn from(
        fair_usage: proto::get_account_state_response::account_state_summary::account_summary::FairUsageState,
    ) -> Self {
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

impl TryFrom<proto::get_account_state_response::account_state_summary::AccountSummary>
    for AccountSummary
{
    type Error = ConversionError;

    fn try_from(
        value: proto::get_account_state_response::account_state_summary::AccountSummary,
    ) -> Result<Self, Self::Error> {
        let account = AccountState::from(value.account());
        let subscription = SubscriptionState::from(value.subscription());
        let device_summary = value
            .device_summary
            .ok_or_else(|| ConversionError::NoValueSet("AccountSummary.device_summary"))
            .map(DeviceSummary::from)?;
        let fair_usage = value
            .fair_usage
            .ok_or_else(|| ConversionError::NoValueSet("AccountSummary.fair_usage"))
            .map(FairUsage::from)?;

        Ok(Self {
            account,
            subscription,
            device_summary,
            fair_usage,
        })
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
impl From<proto::get_account_state_response::account_state_summary::DeviceState> for DeviceState {
    fn from(device: proto::get_account_state_response::account_state_summary::DeviceState) -> Self {
        match device {
            proto::get_account_state_response::account_state_summary::DeviceState::NotRegistered => Self::NotRegistered,
            proto::get_account_state_response::account_state_summary::DeviceState::Inactive => Self::Inactive,
            proto::get_account_state_response::account_state_summary::DeviceState::Active => Self::Active,
            proto::get_account_state_response::account_state_summary::DeviceState::DeleteMe => Self::DeleteMe,
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

impl TryFrom<proto::RegisterDeviceResult> for RegisterDeviceResult {
    type Error = ConversionError;

    fn try_from(value: proto::RegisterDeviceResult) -> Result<Self, Self::Error> {
        match value.kind() {
            proto::register_device_result::RegisterDeviceResultType::InProgress => {
                Ok(RegisterDeviceResult::InProgress)
            }
            proto::register_device_result::RegisterDeviceResultType::Success => {
                Ok(RegisterDeviceResult::Success)
            }
            proto::register_device_result::RegisterDeviceResultType::Failed => {
                let error = value
                    .error
                    .ok_or_else(|| ConversionError::NoValueSet("RegisterDeviceResult.error"))?;
                Ok(RegisterDeviceResult::Failed(error.try_into()?))
            }
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

impl TryFrom<proto::RequestZkNymResult> for RequestZkNymResult {
    type Error = ConversionError;

    fn try_from(value: proto::RequestZkNymResult) -> Result<Self, ConversionError> {
        match value.kind() {
            proto::request_zk_nym_result::RequestZkNymResultType::InProgress => {
                Ok(Self::InProgress)
            }
            proto::request_zk_nym_result::RequestZkNymResultType::Done => {
                let successes = value
                    .successes
                    .into_iter()
                    .map(RequestZkNymSuccess::from)
                    .collect();
                let failures = value
                    .failures
                    .into_iter()
                    .map(RequestZkNymErrorReason::try_from)
                    .collect::<Result<Vec<_>, ConversionError>>()?;

                Ok(Self::Done {
                    successes,
                    failures,
                })
            }
            proto::request_zk_nym_result::RequestZkNymResultType::Error => {
                let error = value
                    .failures
                    .into_iter()
                    .next()
                    .ok_or_else(|| ConversionError::NoValueSet("RequestZkNymResult.failures"))
                    .and_then(RequestZkNymErrorReason::try_from)?;
                Ok(Self::Error(error))
            }
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

impl TryFrom<proto::get_account_state_response::AccountStateSummary> for AccountStateSummary {
    type Error = ConversionError;

    fn try_from(
        state: proto::get_account_state_response::AccountStateSummary,
    ) -> Result<Self, Self::Error> {
        let mnemonic = state
            .mnemonic
            .map(|value| {
                let proto_mnemonic =
                proto::get_account_state_response::account_state_summary::MnemonicState::try_from(
                    value,
                )
                .map_err(|e| ConversionError::Decode("AccountStateSummary.mnemonic", e))?;

                Ok(MnemonicState::from(proto_mnemonic))
            })
            .transpose()?;

        let account_registered = state
            .account_registered
            .map(|value| {
                let proto_account_registered =
                proto::get_account_state_response::account_state_summary::AccountRegistered::try_from(
                    value,
                )
                .map_err(|e| ConversionError::Decode("AccountStateSummary.account_registered", e))?;

                Ok(AccountRegistered::from(proto_account_registered))
            })
            .transpose()?;

        let account_summary = state
            .account_summary
            .map(AccountSummary::try_from)
            .transpose()?;

        let device = state
            .device
            .map(|value| {
                let proto_device_state =
                proto::get_account_state_response::account_state_summary::DeviceState::try_from(
                    value,
                )
                .map_err(|e| ConversionError::Decode("AccountStateSummary.device", e))?;

                Ok(DeviceState::from(proto_device_state))
            })
            .transpose()?;

        let register_device_result = state
            .register_device_result
            .map(RegisterDeviceResult::try_from)
            .transpose()?;
        let request_zk_nym_result = state
            .request_zk_nym_result
            .map(RequestZkNymResult::try_from)
            .transpose()?;

        Ok(Self {
            mnemonic,
            account_registered,
            account_summary,
            device,
            register_device_result,
            request_zk_nym_result,
        })
    }
}
