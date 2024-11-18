// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(thiserror::Error, uniffi::Error, Debug, Clone, PartialEq)]
pub enum VpnError {
    #[error("{details}")]
    InternalError { details: String },

    #[error("{details}")]
    NetworkConnectionError { details: String },

    #[error("{details}")]
    GatewayError { details: String },

    #[error("{details}")]
    InvalidCredential { details: String },

    #[error("Client is out of bandwidth")]
    OutOfBandwidth,

    #[error("{details}")]
    InvalidStateError { details: String },

    #[error("account state is ready to connect")]
    AccountReady,

    #[error("no account stored")]
    NoAccountStored,

    #[error("account not synced")]
    AccountNotSynced,

    #[error("account not registered")]
    AccountNotRegistered,

    #[error("account not active")]
    AccountNotActive,

    #[error("no active subscription")]
    NoActiveSubscription,

    #[error("device not registered")]
    AccountDeviceNotRegistered,

    #[error("device not active")]
    AccountDeviceNotActive,

    #[error("no device identity stored")]
    NoDeviceIdentity,

    #[error("timeout connecting to nym-vpn-api")]
    VpnApiTimeout,
    
    //#[error("max devices reached: {0}")]
    //MaxDevicesReached(u64),
    #[error("account update failed: {details}")]
    AccountUpdateFailed {
        details: String,
        message_id: Option<String>,
        code_reference_id: Option<String>,
    },

    #[error("device update failed: {details}")]
    DeviceUpdateFailed {
        details: String,
        message_id: Option<String>,
        code_reference_id: Option<String>,
    },

    #[error("device registration failed: {details}")]
    DeviceRegistrationFailed {
        details: String,
        message_id: Option<String>,
        code_reference_id: Option<String>,
    },

    #[error("invalid account storage path: {details}")]
    InvalidAccountStoragePath { details: String },
}

impl From<nym_vpn_account_controller::ReadyToConnect> for VpnError {
    fn from(value: nym_vpn_account_controller::ReadyToConnect) -> Self {
        match value {
            nym_vpn_account_controller::ReadyToConnect::Ready => Self::AccountReady,
            nym_vpn_account_controller::ReadyToConnect::NoMnemonicStored => Self::NoAccountStored,
            nym_vpn_account_controller::ReadyToConnect::AccountNotSynced => Self::AccountNotSynced,
            nym_vpn_account_controller::ReadyToConnect::AccountNotRegistered => {
                Self::AccountNotRegistered
            }
            nym_vpn_account_controller::ReadyToConnect::AccountNotActive => Self::AccountNotActive,
            nym_vpn_account_controller::ReadyToConnect::NoActiveSubscription => {
                Self::NoActiveSubscription
            }
            nym_vpn_account_controller::ReadyToConnect::DeviceNotRegistered => {
                Self::AccountDeviceNotRegistered
            }
            nym_vpn_account_controller::ReadyToConnect::DeviceNotActive => {
                Self::AccountDeviceNotActive
            }
            nym_vpn_account_controller::ReadyToConnect::DeviceRegistrationFailed {
                message,
                message_id,
                code_reference_id,
            } => Self::DeviceRegistrationFailed {
                details: message,
                message_id,
                code_reference_id,
            },
        }
    }
}

impl From<nym_vpn_account_controller::AccountCommandError> for VpnError {
    fn from(value: nym_vpn_account_controller::AccountCommandError) -> Self {
        use nym_vpn_account_controller::AccountCommandError;
        match value {
            AccountCommandError::UpdateAccountEndpointFailure {
                message,
                message_id,
                code_reference_id,
                base_url: _,
            } => VpnError::AccountUpdateFailed {
                details: message,
                message_id,
                code_reference_id,
            },
            AccountCommandError::UpdateDeviceEndpointFailure {
                message,
                message_id,
                code_reference_id,
            } => VpnError::DeviceUpdateFailed {
                details: message,
                message_id,
                code_reference_id,
            },
            AccountCommandError::RegisterDeviceEndpointFailure {
                message,
                message_id,
                code_reference_id,
            } => VpnError::DeviceRegistrationFailed {
                details: message,
                message_id,
                code_reference_id,
            },
            AccountCommandError::NoAccountStored => VpnError::NoAccountStored,
            AccountCommandError::NoDeviceStored => VpnError::NoDeviceIdentity,
            AccountCommandError::General(err) => VpnError::InternalError { details: err },
            AccountCommandError::Internal(err) => VpnError::InternalError { details: err },
        }
    }
}

impl From<crate::Error> for VpnError {
    fn from(value: crate::Error) -> Self {
        Self::InternalError {
            details: value.to_string(),
        }
    }
}

impl From<nym_gateway_directory::Error> for VpnError {
    fn from(value: nym_gateway_directory::Error) -> Self {
        Self::NetworkConnectionError {
            details: value.to_string(),
        }
    }
}

impl From<nym_vpn_api_client::VpnApiClientError> for VpnError {
    fn from(value: nym_vpn_api_client::VpnApiClientError) -> Self {
        Self::NetworkConnectionError {
            details: value.to_string(),
        }
    }
}
