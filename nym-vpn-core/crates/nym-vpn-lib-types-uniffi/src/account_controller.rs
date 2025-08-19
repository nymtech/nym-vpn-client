// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(uniffi::Record, Clone, Default, PartialEq)]
pub struct RegisterAccountResponse {
    pub account_token: String,
}

impl From<nym_vpn_lib_types::RegisterAccountResponse> for RegisterAccountResponse {
    fn from(value: nym_vpn_lib_types::RegisterAccountResponse) -> Self {
        RegisterAccountResponse {
            account_token: value.account_token,
        }
    }
}

#[derive(uniffi::Enum, Debug, Clone, PartialEq)]
pub enum AccountControllerState {
    Offline,
    Syncing,
    LoggedOut,
    ReadyToConnect,
    Error(AccountControllerErrorStateReason),
}

impl From<nym_vpn_lib_types::AccountControllerState> for AccountControllerState {
    fn from(value: nym_vpn_lib_types::AccountControllerState) -> Self {
        match value {
            nym_vpn_lib_types::AccountControllerState::Offline => AccountControllerState::Offline,
            nym_vpn_lib_types::AccountControllerState::Syncing => Self::Syncing,
            nym_vpn_lib_types::AccountControllerState::LoggedOut => Self::LoggedOut,
            nym_vpn_lib_types::AccountControllerState::ReadyToConnect => Self::ReadyToConnect,
            nym_vpn_lib_types::AccountControllerState::Error(error_state_reason) => {
                Self::Error(error_state_reason.into())
            }
        }
    }
}

#[derive(uniffi::Enum, Debug, Clone, PartialEq)]
pub enum AccountControllerErrorStateReason {
    Storage { context: String },
    ApiFailure { context: String, details: String },
    Internal { context: String, details: String },
    BandwidthExceeded { context: String },
    AccountStatusNotActive { status: String },
    InactiveSubscription,
    MaxDeviceReached,
    DeviceTimeDesynced,
}

impl From<nym_vpn_lib_types::AccountControllerErrorStateReason>
    for AccountControllerErrorStateReason
{
    fn from(value: nym_vpn_lib_types::AccountControllerErrorStateReason) -> Self {
        match value {
            nym_vpn_lib_types::AccountControllerErrorStateReason::Storage { context } => {
                Self::Storage { context }
            }
            nym_vpn_lib_types::AccountControllerErrorStateReason::ApiFailure {
                context,
                details,
            } => Self::ApiFailure { context, details },
            nym_vpn_lib_types::AccountControllerErrorStateReason::Internal { context, details } => {
                Self::Internal { context, details }
            }
            nym_vpn_lib_types::AccountControllerErrorStateReason::BandwidthExceeded { context } => {
                Self::BandwidthExceeded { context }
            }
            nym_vpn_lib_types::AccountControllerErrorStateReason::AccountStatusNotActive {
                status,
            } => Self::AccountStatusNotActive { status },
            nym_vpn_lib_types::AccountControllerErrorStateReason::InactiveSubscription => {
                Self::InactiveSubscription
            }
            nym_vpn_lib_types::AccountControllerErrorStateReason::MaxDeviceReached => {
                Self::MaxDeviceReached
            }
            nym_vpn_lib_types::AccountControllerErrorStateReason::DeviceTimeDesynced => {
                Self::DeviceTimeDesynced
            }
        }
    }
}
