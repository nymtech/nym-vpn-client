use std::{
    collections::HashMap,
    fmt::{self, Display},
};

use crate::db::DbError;
use crate::vpnd::{client::VpndError, gateway::GatewayType};
use serde::Serialize;
use thiserror::Error;
use ts_rs::TS;

#[derive(Error, Debug, Serialize, TS, Clone)]
#[ts(export, export_to = "tauri.ts", rename = "TBackendError")]
/// Generic error type made to be passed to the frontend and
/// displayed in the UI as localized error message
pub struct BackendError {
    /// Error message for debugging/logs purposes
    /// not intended to be displayed to the user
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

    pub fn with_detail(message: &str, key: ErrorKey, detail: String) -> Self {
        Self {
            message: message.to_string(),
            key,
            data: Some(HashMap::from([("details".to_string(), detail)])),
        }
    }

    pub fn _with_optional_data(
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
            key: ErrorKey::Internal,
            data: data.map(|d| d.into_iter().map(|(k, v)| (k.to_string(), v)).collect()),
        }
    }

    pub fn internal_with_detail(message: &str, detail: String) -> Self {
        Self {
            message: message.to_string(),
            key: ErrorKey::Internal,
            data: Some(HashMap::from([("details".to_string(), detail)])),
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
            VpndError::AuthenticationRequired => {
                BackendError::new("not authenticated with the daemon", ErrorKey::AuthDenied)
            }
            VpndError::RpcClient(e) => {
                BackendError::new(&format!("rpc client error: {e}"), ErrorKey::VpndClient)
            }
            VpndError::FailedToConnectIpc(_) => BackendError::new(
                "not connected to the daemon",
                ErrorKey::NotConnectedToDaemon,
            ),
            VpndError::Response(e) => e,
        }
    }
}

/// Enum of the possible specialized errors emitted by the daemon
/// or the app backend side, to be passed to the UI layer
#[derive(Debug, Serialize, TS, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[ts(export, export_to = "tauri.ts")]
pub enum ErrorKey {
    /// Any error that is not explicitly handled, and not related
    /// to the application layer
    /// Extra data should be passed along to help specialize the problem
    Internal,
    /// Rpc Client layer error
    /// the error does not originate from the application layer
    VpndClient,
    /// Happens when the app is not connected to a running daemon
    /// and attempts to make an RPC call
    NotConnectedToDaemon,
    /// Daemon requires authentication that was denied or cancelled
    AuthDenied,
    // Various mixnet events that should be mapped to errors
    EntryGwDown,
    ExitGwDownIpv4,
    ExitGwDownIpv6,
    ExitGwRoutingErrorIpv4,
    ExitGwRoutingErrorIpv6,
    MixnetNoBandwidth,
    // Some specific account management errors
    AccountInvalidMnemonic,
    AccountInvalidSecret,
    NoAccountStored,
    NoDeviceStored,
    ExistingAccount,
    BandwidthExceeded,
    AccountStatusNotActive,
    NoSubscription,
    MaxDeviceReached,
    DeviceTimeDesync,
    SplitTunnelAppInvalid,
    SplitTunnelAppDuplicate,
    InsufficientFunds,
    // Failure when querying countries from daemon
    GetMixnetEntryCountriesQuery,
    GetMixnetExitCountriesQuery,
    GetWgCountriesQuery,
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

impl From<tauri_plugin_updater::Error> for BackendError {
    fn from(_: tauri_plugin_updater::Error) -> Self {
        BackendError::internal("internal updater error", None)
    }
}

impl From<anyhow::Error> for BackendError {
    fn from(_: anyhow::Error) -> Self {
        BackendError::internal("internal error", None)
    }
}

impl From<DbError> for BackendError {
    fn from(_: DbError) -> Self {
        BackendError::internal("db error", None)
    }
}
