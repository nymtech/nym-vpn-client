use std::path::PathBuf;

use nym_vpn_lib::credential_storage::error::StorageError;
use nym_vpn_lib::id::NymIdError;
use time::OffsetDateTime;
use tracing::error;

#[derive(Clone, Debug, thiserror::Error)]
pub enum ImportCredentialError {
    #[error("vpn is connected")]
    VpnRunning,

    #[error("credential already imported")]
    CredentialAlreadyImported,

    #[error("generic error: {0}")]
    Generic(String),

    #[error("storage error: {path}: {source}")]
    StorageError { path: PathBuf, source: String },

    #[error("failed to deserialize credential: {reason}")]
    DeserializationFailure { reason: String, location: PathBuf },

    #[error("credential expired: {expiration}")]
    CredentialExpired {
        expiration: OffsetDateTime,
        location: PathBuf,
    },

    // TODO: where is this coming from and is it really specific to freepass?
    #[error("freepass expired: {expiration}")]
    FreepassExpired { expiration: String },

    #[error("failed to verify credential")]
    VerificationFailed,

    #[error("failed to query contract")]
    FailedToQueryContract,
}

use nym_vpn_lib::credentials::ImportCredentialError as VpnLibImportCredentialError;

impl From<VpnLibImportCredentialError> for ImportCredentialError {
    fn from(err: VpnLibImportCredentialError) -> Self {
        let mut error = ImportCredentialError::Generic(err.to_string());
        match err {
            VpnLibImportCredentialError::CredentialStoreError { path, source } => {
                ImportCredentialError::StorageError {
                    path,
                    source: source.to_string(),
                }
            }
            VpnLibImportCredentialError::FailedToImportRawCredential { location, source } => {
                match source {
                    NymIdError::CredentialDeserializationFailure { source } => {
                        error = ImportCredentialError::DeserializationFailure {
                            reason: source.to_string(),
                            location,
                        };
                    }
                    NymIdError::ExpiredCredentialImport { expiration } => {
                        error = ImportCredentialError::CredentialExpired {
                            expiration,
                            location,
                        };
                    }
                    NymIdError::StorageError { source } => {
                        if let Some(storage_error) = source.downcast_ref::<StorageError>() {
                            match storage_error {
                                StorageError::InternalDatabaseError(db_error) => {
                                    // There was a recent change for the upstream crate
                                    // that adds a new variant to StorageError to capture
                                    // duplicate entries. Until that change makes its way
                                    // to the vpn-lib, we just match on the string as a
                                    // temporary solution.
                                    if db_error.to_string().contains("code: 2067") {
                                        error = ImportCredentialError::CredentialAlreadyImported
                                    } else {
                                        error = ImportCredentialError::StorageError {
                                            path: location,
                                            source: error.to_string(),
                                        }
                                    }
                                }
                                StorageError::MigrationError(_) => (),
                                StorageError::InconsistentData => (),
                                StorageError::NoCredential => (),
                            }
                        } else {
                            error = ImportCredentialError::StorageError {
                                path: location,
                                source: source.to_string(),
                            }
                        }
                    }
                }
            }
        };
        error
    }
}
