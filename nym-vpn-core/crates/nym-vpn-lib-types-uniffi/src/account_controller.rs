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

#[derive(uniffi::Enum)]
pub enum AccountCommandErrorKind {
    Internal,
    Storage,
    VpnApi,
    UnexpectedVpnApiResponse,
    NoAccountStored,
    NoDeviceStored,
    ExistingAccount,
    Offline,
    InvalidMnemonic,
    NyxdConnectionFailure,
    NyxdQueryFailure,
    AccountDoesntExistOnChain,
    AccountNotDecentralised,
    AccountDecentralised,
    InsufficientFunds,
    ZkNymAcquisitionFailure,
}

#[derive(Debug, uniffi::Object)]
pub struct AccountCommandError {
    inner: nym_vpn_lib_types::AccountCommandError,
}

impl AccountCommandError {
    pub fn new(inner: nym_vpn_lib_types::AccountCommandError) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl AccountCommandError {
    pub fn kind(&self) -> AccountCommandErrorKind {
        match self.inner {
            nym_vpn_lib_types::AccountCommandError::Internal(_) => {
                AccountCommandErrorKind::Internal
            }
            nym_vpn_lib_types::AccountCommandError::Storage(_) => AccountCommandErrorKind::Storage,
            nym_vpn_lib_types::AccountCommandError::VpnApi(_) => AccountCommandErrorKind::VpnApi,
            nym_vpn_lib_types::AccountCommandError::UnexpectedVpnApiResponse(_) => {
                AccountCommandErrorKind::UnexpectedVpnApiResponse
            }
            nym_vpn_lib_types::AccountCommandError::NoAccountStored => {
                AccountCommandErrorKind::NoAccountStored
            }
            nym_vpn_lib_types::AccountCommandError::NoDeviceStored => {
                AccountCommandErrorKind::NoDeviceStored
            }
            nym_vpn_lib_types::AccountCommandError::ExistingAccount => {
                AccountCommandErrorKind::ExistingAccount
            }
            nym_vpn_lib_types::AccountCommandError::Offline => AccountCommandErrorKind::Offline,
            nym_vpn_lib_types::AccountCommandError::InvalidMnemonic(_) => {
                AccountCommandErrorKind::InvalidMnemonic
            }
            nym_vpn_lib_types::AccountCommandError::NyxdConnectionFailure(_) => {
                AccountCommandErrorKind::NyxdConnectionFailure
            }
            nym_vpn_lib_types::AccountCommandError::NyxdQueryFailure(_) => {
                AccountCommandErrorKind::NyxdQueryFailure
            }
            nym_vpn_lib_types::AccountCommandError::AccountDoesntExistOnChain => {
                AccountCommandErrorKind::AccountDoesntExistOnChain
            }
            nym_vpn_lib_types::AccountCommandError::AccountNotDecentralised => {
                AccountCommandErrorKind::AccountNotDecentralised
            }
            nym_vpn_lib_types::AccountCommandError::AccountDecentralised => {
                AccountCommandErrorKind::AccountDecentralised
            }
            nym_vpn_lib_types::AccountCommandError::InsufficientFunds => {
                AccountCommandErrorKind::InsufficientFunds
            }
            nym_vpn_lib_types::AccountCommandError::ZkNymAcquisitionFailure(_) => {
                AccountCommandErrorKind::ZkNymAcquisitionFailure
            }
        }
    }

    pub fn message(&self) -> String {
        self.inner.to_string()
    }
}

impl From<nym_vpn_lib_types::AccountCommandError> for AccountCommandError {
    fn from(value: nym_vpn_lib_types::AccountCommandError) -> Self {
        Self::new(value)
    }
}

#[derive(uniffi::Enum, Debug, Clone, PartialEq)]
pub enum AccountControllerState {
    Offline,
    Syncing,
    LoggedOut,
    Decentralised,
    ReadyToConnect,
    Error(AccountControllerErrorStateReason),
}

impl From<nym_vpn_lib_types::AccountControllerState> for AccountControllerState {
    fn from(value: nym_vpn_lib_types::AccountControllerState) -> Self {
        match value {
            nym_vpn_lib_types::AccountControllerState::Offline => AccountControllerState::Offline,
            nym_vpn_lib_types::AccountControllerState::Syncing => Self::Syncing,
            nym_vpn_lib_types::AccountControllerState::LoggedOut => Self::LoggedOut,
            nym_vpn_lib_types::AccountControllerState::Decentralised => Self::Decentralised,
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
