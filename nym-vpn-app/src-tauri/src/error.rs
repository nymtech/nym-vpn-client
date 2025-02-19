use std::{
    collections::HashMap,
    fmt::{self, Display},
};

use nym_vpn_proto::account_error::AccountErrorType;
use nym_vpn_proto::connect_request_error::ConnectRequestErrorType;
use nym_vpn_proto::set_network_request_error::SetNetworkRequestErrorType;
use nym_vpn_proto::{AccountError, ConnectRequestError, SetNetworkRequestError};
use serde::Serialize;
use thiserror::Error;
use ts_rs::TS;

use crate::grpc::client::VpndError;
use crate::grpc::gateway::GatewayType;

#[derive(Error, Debug, Serialize, TS, Clone)]
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

    pub fn _with_data(message: &str, key: ErrorKey, data: HashMap<&str, String>) -> Self {
        Self {
            message: message.to_string(),
            key,
            data: Some(data.into_iter().map(|(k, v)| (k.to_string(), v)).collect()),
        }
    }

    pub fn with_details(message: &str, key: ErrorKey, details: String) -> Self {
        Self {
            message: message.to_string(),
            key,
            data: Some(HashMap::from([("details".to_string(), details)])),
        }
    }

    pub fn with_optional_data(
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

    pub fn internal(message: &str, data: Option<HashMap<String, String>>) -> Self {
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
            VpndError::Response(e) => e,
        }
    }
}

/// Enum of the possible specialized errors emitted by the daemon
/// or the app backend side, to be passed to the UI layer
#[derive(Debug, Serialize, TS, Clone)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum ErrorKey {
    /// Generic unhandled error
    UnknownError,
    /// Any error that is not explicitly handled, and not related
    /// to the application layer
    /// Extra data should be passed along to help specialize the problem
    InternalError,
    /// gRPC bare layer error, when an RPC call fails (aka `tonic::Status`)
    /// That is, the error does not come from the application layer
    GrpcError,
    /// Happens when the app is not connected to a running daemon
    /// and attempts to make a gRPC call
    NotConnectedToDaemon,
    // Various mixnet events that should be mapped to errors
    EntryGwDown,
    ExitGwDownIpv4,
    ExitGwDownIpv6,
    ExitGwRoutingErrorIpv4,
    ExitGwRoutingErrorIpv6,
    NoBandwidth,
    // Forwarded from proto `account_error::AccountErrorType`
    AccountInvalidMnemonic,
    AccountStorage,
    AccountIsConnected,
    // Failure when querying countries from gRPC
    GetMixnetEntryCountriesQuery,
    GetMixnetExitCountriesQuery,
    GetWgCountriesQuery,
    // Forwarded from proto `set_network_request_error::SetNetworkRequestErrorType`
    InvalidNetworkName,
}

impl From<AccountError> for BackendError {
    fn from(error: AccountError) -> Self {
        let data = error.details.clone().into();
        match error.kind() {
            AccountErrorType::StoreAccountErrorUnspecified => {
                BackendError::internal("grpc UNSPECIFIED", data)
            }
            AccountErrorType::InvalidMnemonic => BackendError::with_optional_data(
                "The provided mnemonic was not able to be parsed",
                ErrorKey::AccountInvalidMnemonic,
                data,
            ),
            AccountErrorType::Storage => BackendError::with_optional_data(
                "General error from the storage backend",
                ErrorKey::AccountStorage,
                data,
            ),
            AccountErrorType::IsConnected => BackendError::with_optional_data(
                "Unable to proceed while connected",
                ErrorKey::AccountIsConnected,
                data,
            ),
        }
    }
}

impl From<ConnectRequestErrorType> for ErrorKey {
    fn from(error: ConnectRequestErrorType) -> Self {
        match error {
            // let's keep this 0brain pattern matching for the sake
            // of reference and safety in case of future changes
            ConnectRequestErrorType::Internal | ConnectRequestErrorType::Unspecified => {
                ErrorKey::InternalError
            }
        }
    }
}

impl From<ConnectRequestError> for BackendError {
    fn from(error: ConnectRequestError) -> Self {
        BackendError::new(&error.message, ErrorKey::from(error.kind()))
    }
}

impl From<GatewayType> for ErrorKey {
    fn from(value: GatewayType) -> Self {
        match value {
            GatewayType::MxEntry => ErrorKey::GetMixnetEntryCountriesQuery,
            GatewayType::MxExit => ErrorKey::GetMixnetExitCountriesQuery,
            GatewayType::Wg => ErrorKey::GetWgCountriesQuery,
        }
    }
}

impl From<SetNetworkRequestErrorType> for ErrorKey {
    fn from(error: SetNetworkRequestErrorType) -> Self {
        match error {
            SetNetworkRequestErrorType::Internal => ErrorKey::InternalError,
            SetNetworkRequestErrorType::InvalidNetworkName => ErrorKey::InvalidNetworkName,
            SetNetworkRequestErrorType::Unspecified => ErrorKey::UnknownError,
        }
    }
}

impl From<SetNetworkRequestError> for BackendError {
    fn from(error: SetNetworkRequestError) -> Self {
        let message = error.message.clone();
        BackendError::new(&message, ErrorKey::from(error.kind()))
    }
}
