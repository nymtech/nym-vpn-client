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
pub struct BackendError {
    /// Human readable error message for debugging/logs purposes
    pub message: String,
    /// Error key to be used in the UI to display localized error message
    pub key: ErrorKey,
    /// Extra data to be passed along to help specialize the problem
    pub data: Option<HashMap<String, String>>,
}

impl BackendError {
    pub fn new(message: &str, key: ErrorKey) -> Self {
        Self {
            message: message.to_string(),
            key,
            data: None,
        }
    }

    pub fn new_with_data(message: &str, key: ErrorKey, data: HashMap<&str, String>) -> Self {
        Self {
            message: message.to_string(),
            key,
            data: Some(data.into_iter().map(|(k, v)| (k.to_string(), v)).collect()),
        }
    }

    pub fn new_with_optional_data(
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
            data: data.map(|d| d.into_iter().map(|(k, v)| (k.to_string(), v)).collect()),
        }
    }
}

impl Display for BackendError {
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

impl From<VpndError> for BackendError {
    fn from(error: VpndError) -> Self {
        match error {
            VpndError::GrpcError(s) => {
                BackendError::new(&format!("grpc error: {}", s), ErrorKey::GrpcError)
            }
            VpndError::FailedToConnectIpc(_) | VpndError::FailedToConnectHttp(_) => {
                BackendError::new(
                    "not connected to the daemon",
                    ErrorKey::NotConnectedToDaemon,
                )
            }
        }
    }
}

impl From<nym_vpn_proto::Error> for BackendError {
    fn from(error: nym_vpn_proto::Error) -> Self {
        Self {
            message: error.message.clone(),
            key: error.kind().into(),
            data: error.details.into(),
        }
    }
}

/// Enum of the possible specialized errors emitted by the daemon
/// or the app backend side, to be passed to the UI layer
#[derive(Debug, Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub enum ErrorKey {
    /// Generic unhandled error
    UnknownError,
    /// Any error that is not explicitly handled, and not related
    /// to the application layer
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
    ConnectionSameEntryAndExitGw,
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
    /// Forwarded from proto
    OutOfBandwidth,
    /// HTTP request failure when fetching countries from the Gateway API
    GetEntryCountriesRequest,
    GetExitCountriesRequest,
}

impl From<DaemonError> for ErrorKey {
    fn from(value: DaemonError) -> Self {
        match value {
            DaemonError::NoValidCredentials => ErrorKey::ConnectionNoValidCredential,
            DaemonError::Timeout => ErrorKey::ConnectionTimeout,
            DaemonError::GatewayDirectory => ErrorKey::ConnectionGatewayLookup,
            DaemonError::GatewayDirectorySameEntryAndExitGw => {
                ErrorKey::ConnectionSameEntryAndExitGw
            }
            DaemonError::OutOfBandwidth => ErrorKey::OutOfBandwidth,
            _ => ErrorKey::UnknownError,
        }
    }
}

impl From<ImportError> for BackendError {
    fn from(error: ImportError) -> Self {
        let data = error.details.clone().into();
        match error.kind() {
            ImportErrorType::Unspecified => BackendError::new_internal("grpc unspecified", data),
            ImportErrorType::VpnRunning => BackendError::new_with_optional_data(
                "vpn running",
                ErrorKey::CredentialVpnRunning,
                data,
            ),
            ImportErrorType::CredentialAlreadyImported => BackendError::new_with_optional_data(
                "credential already imported",
                ErrorKey::CredentialAlreadyImported,
                data,
            ),
            ImportErrorType::StorageError => {
                // TODO remove this
                // backward compatibility check with the old error message from daemon
                if data.as_ref().is_some_and(|d| {
                    d.get("error")
                        .is_some_and(|e| e.contains("unique constraint violation"))
                }) {
                    return BackendError::new_with_optional_data(
                        "credential already imported",
                        ErrorKey::CredentialAlreadyImported,
                        data,
                    );
                }
                BackendError::new_with_optional_data(
                    "credential strorage error",
                    ErrorKey::CredentialStorageError,
                    data,
                )
            }
            ImportErrorType::DeserializationFailure => BackendError::new_with_optional_data(
                "credential deserialization failure",
                ErrorKey::CredentialDeserializationFailure,
                data,
            ),
            ImportErrorType::CredentialExpired => BackendError::new_with_optional_data(
                "credential expired",
                ErrorKey::CredentialExpired,
                data,
            ),
        }
    }
}
