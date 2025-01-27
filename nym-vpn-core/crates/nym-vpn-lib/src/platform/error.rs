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

    #[error("account update failed: {details}")]
    UpdateAccountEndpointFailure {
        details: String,
        message_id: Option<String>,
        code_reference_id: Option<String>,
    },

    #[error("device update failed: {details}")]
    UpdateDeviceEndpointFailure {
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

    #[error("failed to request zk nym")]
    RequestZkNym {
        successes: Vec<RequestZkNymSuccess>,
        failed: Vec<RequestZkNymError>,
    },

    #[error("invalid account storage path: {details}")]
    InvalidAccountStoragePath { details: String },

    #[error("invalid statistics recipient")]
    StatisticsRecipient,

    #[error("failed to remove device from nym vpn api: {details}")]
    UnregisterDeviceApiClientFailure { details: String },

    #[error("failed to parse mnemonic with error: {details}")]
    InvalidMnemonic { details: String },
}

#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq)]
pub struct RequestZkNymSuccess {
    pub id: String,
}

impl From<nym_vpn_account_controller::RequestZkNymSuccess> for RequestZkNymSuccess {
    fn from(value: nym_vpn_account_controller::RequestZkNymSuccess) -> Self {
        Self { id: value.id }
    }
}

#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq)]
pub struct RequestZkNymError {
    pub message: String,
    pub message_id: Option<String>,
    pub ticket_type: Option<String>,
}

impl From<nym_vpn_account_controller::RequestZkNymError> for RequestZkNymError {
    fn from(error: nym_vpn_account_controller::RequestZkNymError) -> Self {
        Self {
            message: error.message(),
            message_id: error.message_id(),
            ticket_type: error.ticket_type(),
        }
    }
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
        }
    }
}

impl From<nym_vpn_account_controller::AccountCommandError> for VpnError {
    fn from(value: nym_vpn_account_controller::AccountCommandError) -> Self {
        use nym_vpn_account_controller::AccountCommandError;
        match value {
            AccountCommandError::SyncAccountEndpointFailure(e) => {
                VpnError::UpdateAccountEndpointFailure {
                    details: e.message,
                    message_id: e.message_id,
                    code_reference_id: e.code_reference_id,
                }
            }
            AccountCommandError::SyncDeviceEndpointFailure(e) => {
                VpnError::UpdateDeviceEndpointFailure {
                    details: e.message,
                    message_id: e.message_id,
                    code_reference_id: e.code_reference_id,
                }
            }
            AccountCommandError::RegisterDeviceEndpointFailure(e) => {
                VpnError::DeviceRegistrationFailed {
                    details: e.message,
                    message_id: e.message_id,
                    code_reference_id: e.code_reference_id,
                }
            }
            AccountCommandError::RequestZkNym { successes, failed } => VpnError::RequestZkNym {
                successes: successes.into_iter().map(|e| e.into()).collect(),
                failed: failed.into_iter().map(|e| e.into()).collect(),
            },
            AccountCommandError::RequestZkNymGeneral(e) => VpnError::RequestZkNym {
                successes: vec![],
                failed: vec![e.into()],
            },
            AccountCommandError::NoAccountStored => VpnError::NoAccountStored,
            AccountCommandError::NoDeviceStored => VpnError::NoDeviceIdentity,
            AccountCommandError::RemoveAccount(e) => VpnError::InternalError { details: e },
            AccountCommandError::RemoveDeviceIdentity(e) => VpnError::InternalError { details: e },
            AccountCommandError::ResetCredentialStorage(e) => {
                VpnError::InternalError { details: e }
            }
            AccountCommandError::RemoveAccountFiles(e) => VpnError::InternalError { details: e },
            AccountCommandError::InitDeviceKeys(e) => VpnError::InternalError { details: e },
            AccountCommandError::General(err) => VpnError::InternalError { details: err },
            AccountCommandError::Internal(err) => VpnError::InternalError { details: err },
            AccountCommandError::UnregisterDeviceApiClientFailure(err) => {
                VpnError::InternalError { details: err }
            }
            AccountCommandError::RegistrationInProgress => VpnError::InternalError {
                details: value.to_string(),
            },
            AccountCommandError::GetAccountEndpointFailure(e) => {
                VpnError::UpdateAccountEndpointFailure {
                    details: e.message,
                    message_id: e.message_id,
                    code_reference_id: e.code_reference_id,
                }
            }
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
