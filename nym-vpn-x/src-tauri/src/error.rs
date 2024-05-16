use std::fmt::{self, Display};

use nym_vpn_proto::import_error::ImportErrorType;
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

#[derive(Error, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CmdError {
    #[source]
    pub source: CmdErrorSource,
    pub message: String,
    pub i18n_key: Option<I18nKey>,
}

impl CmdError {
    pub fn new(error: CmdErrorSource, message: &str) -> Self {
        Self {
            message: message.to_string(),
            source: error,
            i18n_key: None,
        }
    }

    pub fn new_with_local(error: CmdErrorSource, message: &str, key: I18nKey) -> Self {
        Self {
            message: message.to_string(),
            source: error,
            i18n_key: Some(key),
        }
    }
}

impl Display for CmdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.source, self.message)
    }
}

impl From<VpndError> for CmdError {
    fn from(error: VpndError) -> Self {
        match error {
            VpndError::RpcError(s) => CmdError::new(
                CmdErrorSource::DaemonError,
                &format!("failed to call the daemon: {}", s),
            ),
            VpndError::FailedToConnectIpc(_) | VpndError::FailedToConnectHttp(_) => {
                CmdError::new(CmdErrorSource::DaemonError, "not connected to the daemon")
            }
        }
    }
}

/// Enum of the localization keys for error messages
/// displayed in the UI
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum I18nKey {
    UnknownError,
    CredentialInvalid,
    CredentialVpnRunning,
    CredentialAlreadyImported,
    CredentialStorageError,
    CredentialDeserializationFailure,
    CredentialExpired,
}

impl From<ImportErrorType> for CmdError {
    fn from(error: ImportErrorType) -> Self {
        match error {
            ImportErrorType::Unspecified => CmdError::new_with_local(
                CmdErrorSource::InternalError,
                "grpc unspecified",
                I18nKey::UnknownError,
            ),
            ImportErrorType::VpnRunning => CmdError::new_with_local(
                CmdErrorSource::CallerError,
                "vpn running",
                I18nKey::CredentialVpnRunning,
            ),
            ImportErrorType::CredentialAlreadyImported => CmdError::new_with_local(
                CmdErrorSource::CallerError,
                "credential already imported",
                I18nKey::CredentialAlreadyImported,
            ),
            ImportErrorType::StorageError => CmdError::new_with_local(
                CmdErrorSource::InternalError,
                "credential strorage error",
                I18nKey::CredentialStorageError,
            ),
            ImportErrorType::DeserializationFailure => CmdError::new_with_local(
                CmdErrorSource::InternalError,
                "credential deserialization failure",
                I18nKey::CredentialDeserializationFailure,
            ),
            ImportErrorType::CredentialExpired => CmdError::new_with_local(
                CmdErrorSource::CallerError,
                "credential expired",
                I18nKey::CredentialExpired,
            ),
        }
    }
}
