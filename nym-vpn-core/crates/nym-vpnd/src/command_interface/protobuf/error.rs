// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::service::{AccountNotReady, SetNetworkError, VpnServiceConnectError};

impl From<VpnServiceConnectError> for nym_vpn_proto::ConnectRequestError {
    fn from(err: VpnServiceConnectError) -> Self {
        match err {
            VpnServiceConnectError::Internal(ref _account_error) => {
                nym_vpn_proto::ConnectRequestError {
                    kind: nym_vpn_proto::connect_request_error::ConnectRequestErrorType::Internal
                        as i32,
                    message: err.to_string(),
                    message_id: None,
                    zk_nym_error: Vec::new(),
                }
            }
            VpnServiceConnectError::Account(not_ready_to_connect) => {
                nym_vpn_proto::ConnectRequestError::from(not_ready_to_connect)
            }
            VpnServiceConnectError::Cancel => nym_vpn_proto::ConnectRequestError {
                kind: nym_vpn_proto::connect_request_error::ConnectRequestErrorType::Internal
                    as i32,
                message: err.to_string(),
                message_id: None,
                zk_nym_error: Vec::new(),
            },
        }
    }
}

impl From<AccountNotReady> for nym_vpn_proto::ConnectRequestError {
    fn from(error: AccountNotReady) -> Self {
        match error {
            AccountNotReady::UpdateAccount {
                message,
                message_id,
                code_reference_id: _,
            } => Self {
                kind: nym_vpn_proto::connect_request_error::ConnectRequestErrorType::UpdateAccount
                    as i32,
                message: message.clone(),
                message_id: message_id.clone(),
                zk_nym_error: Vec::new(),
            },
            AccountNotReady::UpdateDevice {
                message,
                message_id,
                code_reference_id: _,
            } => Self {
                kind: nym_vpn_proto::connect_request_error::ConnectRequestErrorType::UpdateDevice
                    as i32,
                message: message.clone(),
                message_id: message_id.clone(),
                zk_nym_error: Vec::new(),
            },
            AccountNotReady::RegisterDevice {
                message,
                message_id,
                code_reference_id: _,
            } => Self {
                kind: nym_vpn_proto::connect_request_error::ConnectRequestErrorType::RegisterDevice
                    as i32,
                message: message.clone(),
                message_id: message_id.clone(),
                zk_nym_error: Vec::new(),
            },
            AccountNotReady::RequestZkNym { ref failed } => {
                let zk_nym_error = failed
                    .clone()
                    .into_iter()
                    .map(nym_vpn_proto::RequestZkNymError::from)
                    .collect();
                Self {
                    kind:
                        nym_vpn_proto::connect_request_error::ConnectRequestErrorType::RequestZkNym
                            as i32,
                    message: error.to_string(),
                    message_id: None,
                    zk_nym_error,
                }
            }
            AccountNotReady::NoAccountStored => Self {
                kind: nym_vpn_proto::connect_request_error::ConnectRequestErrorType::NoAccountStored
                    as i32,
                message: error.to_string(),
                message_id: None,
                zk_nym_error: Vec::new(),
            },
            AccountNotReady::NoDeviceStored => Self {
                kind: nym_vpn_proto::connect_request_error::ConnectRequestErrorType::NoDeviceStored
                    as i32,
                message: error.to_string(),
                message_id: None,
                zk_nym_error: Vec::new(),
            },
            AccountNotReady::General(_) => Self {
                kind: nym_vpn_proto::connect_request_error::ConnectRequestErrorType::Internal
                    as i32,
                message: error.to_string(),
                message_id: None,
                zk_nym_error: Vec::new(),
            },
            AccountNotReady::Internal(_) => Self {
                kind: nym_vpn_proto::connect_request_error::ConnectRequestErrorType::Internal
                    as i32,
                message: error.to_string(),
                message_id: None,
                zk_nym_error: Vec::new(),
            },
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum VpnCommandSendError {
    #[error("failed to send command to VPN service task")]
    Send,

    #[error("failed to receive response from VPN service task")]
    Receive,
}

impl From<VpnCommandSendError> for tonic::Status {
    fn from(err: VpnCommandSendError) -> Self {
        match err {
            VpnCommandSendError::Send | VpnCommandSendError::Receive => {
                tonic::Status::internal(err.to_string())
            }
        }
    }
}

impl From<SetNetworkError> for nym_vpn_proto::SetNetworkRequestError {
    fn from(err: SetNetworkError) -> Self {
        match err {
            SetNetworkError::NetworkNotFound(ref err) => nym_vpn_proto::SetNetworkRequestError {
                kind: nym_vpn_proto::set_network_request_error::SetNetworkRequestErrorType::InvalidNetworkName as i32,
                message: err.to_string(),
            },
            SetNetworkError::ReadConfig { .. } => nym_vpn_proto::SetNetworkRequestError {
                kind: nym_vpn_proto::set_network_request_error::SetNetworkRequestErrorType::Internal
                    as i32,
                message: err.to_string(),
            },
            SetNetworkError::WriteConfig { .. } => nym_vpn_proto::SetNetworkRequestError {
                kind: nym_vpn_proto::set_network_request_error::SetNetworkRequestErrorType::Internal
                    as i32,
                message: err.to_string(),
            },
        }
    }
}
