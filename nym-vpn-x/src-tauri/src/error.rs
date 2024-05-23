use std::{
    collections::HashMap,
    fmt::{self, Display},
};

use nym_vpn_proto::import_error::ImportErrorType;
use nym_vpn_proto::{error::ErrorType as DaemonError, ImportError};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

use crate::grpc::client::VpndError;

#[derive(Error, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum CmdErrorSource {
    #[error("daemon error")]
    DaemonError,
    #[error("internal error")]
    InternalError,
    #[error("caller error")]
    CallerError,
    #[error("unknown error")]
    Unknown,
}

#[derive(Error, Debug, Serialize, Deserialize, TS, Clone)]
#[ts(export)]
/// Generic error type made to be passed to the frontend and
/// displayed in the UI as localized error message
/// `Bkd` stands for 'Backend', source side from where the error is emitted
pub struct BkdError {
    /// Human readable error message for debugging/logs purposes
    pub message: String,
    /// Error key to be used in the UI to display localized error message
    pub key: ErrorKey,
    /// Extra data to be passed along to help specialize the problem
    pub data: Option<HashMap<String, String>>,
}

impl BkdError {
    pub fn new(message: &str, key: ErrorKey) -> Self {
        Self {
            message: message.to_string(),
            key,
            data: None,
        }
    }

    pub fn new_with_data(
        message: &str,
        key: ErrorKey,
        data: Option<HashMap<String, String>>,
    ) -> Self {
        Self {
            message: message.to_string(),
            key,
            data,
        }
    }

    pub fn new_internal(message: &str, data: Option<HashMap<String, String>>) -> Self {
        Self {
            message: message.to_string(),
            key: ErrorKey::InternalError,
            data,
        }
    }
}

impl Display for BkdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "message '{}' key [{:?}] data [{:?}]",
            self.message,
            self.key,
            self.data.as_ref()
        )
    }
}

impl From<VpndError> for BkdError {
    fn from(error: VpndError) -> Self {
        match error {
            VpndError::GrpcError(s) => {
                BkdError::new(&format!("grpc error: {}", s), ErrorKey::GrpcError)
            }
            VpndError::FailedToConnectIpc(_) | VpndError::FailedToConnectHttp(_) => BkdError::new(
                "not connected to the daemon",
                ErrorKey::NotConnectedToDaemon,
            ),
        }
    }
}

impl From<nym_vpn_proto::Error> for BkdError {
    fn from(error: nym_vpn_proto::Error) -> Self {
        Self {
            message: error.message.clone(),
            key: error.kind().into(),
            data: error.details.into(),
        }
    }
}

/// Enum of the possible typed errors emitted by the daemon and
/// passed to the UI layer
#[derive(Debug, Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub enum ErrorKey {
    /// Any unhandled error
    UnknownError,
    /// Any error that is not explicitly handled
    /// Extra data should be passed along to help specialize the problem
    InternalError,
    /// gRPC bare layer error, when a RPC call fails (aka `Tonic::Status`)
    /// That is, the error does not come from the application layer
    GrpcError,
    /// Happens when the app is not connected to a running daemon
    /// and attempts to make a gRPC call
    NotConnectedToDaemon,
    /// Forwarded from proto
    ConnectionTimeout,
    /// Forwarded from proto
    ConnectionGatewayLookup,
    /// Forwarded from proto
    ConnectionNoValidCredential,
    /// Forwarded from proto
    CredentialInvalid,
    /// Forwarded from proto
    CredentialVpnRunning,
    /// Forwarded from proto
    CredentialAlreadyImported,
    /// Forwarded from proto
    CredentialStorageError,
    /// Forwarded from proto
    CredentialDeserializationFailure,
    /// Forwarded from proto
    CredentialExpired,
}

impl From<DaemonError> for ErrorKey {
    fn from(value: DaemonError) -> Self {
        match value {
            DaemonError::NoValidCredentials => ErrorKey::ConnectionNoValidCredential,
            DaemonError::Timeout => ErrorKey::ConnectionTimeout,
            DaemonError::GatewayDirectory => ErrorKey::ConnectionGatewayLookup,
            _ => ErrorKey::UnknownError,
        }
    }
}

impl From<ImportError> for BkdError {
    fn from(error: ImportError) -> Self {
        let data = error.details.clone().into();
        match error.kind() {
            ImportErrorType::Unspecified => BkdError::new_internal("grpc unspecified", data),
            ImportErrorType::VpnRunning => {
                BkdError::new_with_data("vpn running", ErrorKey::CredentialVpnRunning, data)
            }
            ImportErrorType::CredentialAlreadyImported => BkdError::new_with_data(
                "credential already imported",
                ErrorKey::CredentialAlreadyImported,
                data,
            ),
            ImportErrorType::StorageError => BkdError::new_with_data(
                "credential strorage error",
                ErrorKey::CredentialStorageError,
                data,
            ),
            ImportErrorType::DeserializationFailure => BkdError::new_with_data(
                "credential deserialization failure",
                ErrorKey::CredentialDeserializationFailure,
                data,
            ),
            ImportErrorType::CredentialExpired => {
                BkdError::new_with_data("credential expired", ErrorKey::CredentialExpired, data)
            }
        }
    }
}
