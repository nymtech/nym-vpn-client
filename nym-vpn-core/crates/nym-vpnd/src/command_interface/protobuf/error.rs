// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::service::{SetNetworkError, VpnServiceConnectError};

impl From<VpnServiceConnectError> for nym_vpn_proto::ConnectRequestError {
    fn from(err: VpnServiceConnectError) -> Self {
        match err {
            VpnServiceConnectError::Internal(ref _account_error) => {
                nym_vpn_proto::ConnectRequestError {
                    kind: nym_vpn_proto::connect_request_error::ConnectRequestErrorType::Internal
                        as i32,
                    message: err.to_string(),
                }
            }
            VpnServiceConnectError::Cancel => nym_vpn_proto::ConnectRequestError {
                kind: nym_vpn_proto::connect_request_error::ConnectRequestErrorType::Internal
                    as i32,
                message: err.to_string(),
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
