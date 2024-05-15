use std::path::PathBuf;

use nym_vpn_lib::{
    credential_storage::error::StorageError,
    credentials::ImportCredentialError as VpnLibImportCredentialError, id::NymIdError,
};
use time::OffsetDateTime;
use tracing::error;

#[derive(Clone, Debug, thiserror::Error)]
pub enum ImportCredentialError {
    #[error("vpn is connected")]
    VpnRunning,

    #[error("credential already imported")]
    CredentialAlreadyImported,

    #[error("storage error: {path}: {error}")]
    StorageError { path: PathBuf, error: String },

    #[error("failed to deserialize credential: {reason}")]
    DeserializationFailure { reason: String, location: PathBuf },

    #[error("credential expired: {expiration}")]
    CredentialExpired {
        expiration: OffsetDateTime,
        location: PathBuf,
    },
    // #[error("failed to verify credential")]
    // VerificationFailed,

    // #[error("failed to query contract")]
    // FailedToQueryContract,
}

impl From<VpnLibImportCredentialError> for ImportCredentialError {
    fn from(err: VpnLibImportCredentialError) -> Self {
        match err {
            VpnLibImportCredentialError::CredentialStoreError { path, source } => {
                ImportCredentialError::StorageError {
                    path,
                    error: source.to_string(),
                }
            }
            VpnLibImportCredentialError::FailedToImportRawCredential { location, source } => {
                match source {
                    NymIdError::CredentialDeserializationFailure { source } => {
                        ImportCredentialError::DeserializationFailure {
                            reason: source.to_string(),
                            location,
                        }
                    }
                    NymIdError::ExpiredCredentialImport { expiration } => {
                        ImportCredentialError::CredentialExpired {
                            expiration,
                            location,
                        }
                    }
                    NymIdError::StorageError { source } => {
                        // There was a recent change for the upstream crate that adds a new variant
                        // to StorageError to capture duplicate entries. Until that change makes
                        // its way to the vpn-lib, we just match on the string as a temporary
                        // solution.
                        if let Some(StorageError::InternalDatabaseError(db_error)) =
                            source.downcast_ref::<StorageError>()
                        {
                            if db_error.to_string().contains("code: 2067") {
                                return ImportCredentialError::CredentialAlreadyImported;
                            }
                        }
                        ImportCredentialError::StorageError {
                            path: location,
                            error: source.to_string(),
                        }
                    }
                }
            }
        }
    }
}
