use std::path::PathBuf;

use nym_vpn_lib::credential_storage::error::StorageError;
use nym_vpn_lib::credentials::CredentialError;
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

    #[error("storage error: {0}")]
    StorageError(String),

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

impl From<CredentialError> for ImportCredentialError {
    fn from(err: CredentialError) -> Self {
        let mut error = ImportCredentialError::Generic(err.to_string());
        match err {
            CredentialError::FailedToSetupStoragePaths { .. } => {}
            CredentialError::FailedToCreateCredentialStoreDirectory { .. } => {}
            CredentialError::FailedToReadCredentialStoreMetadata { .. } => {}
            CredentialError::FailedToSetCredentialStorePermissions { .. } => {}
            CredentialError::FailedToInitializePersistentStorage { .. } => {}
            CredentialError::FreepassExpired { expiry_date } => {
                error = ImportCredentialError::FreepassExpired {
                    expiration: expiry_date,
                };
            }
            CredentialError::FailedToReadCredentialFile { .. } => {}
            CredentialError::FailedToImportCredential { location, source } => {
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
                                        error =
                                            ImportCredentialError::StorageError(error.to_string())
                                    }
                                }
                                StorageError::MigrationError(_) => (),
                                StorageError::InconsistentData => (),
                                StorageError::NoCredential => (),
                            }
                        } else {
                            error = ImportCredentialError::StorageError(source.to_string())
                        }
                    }
                }
            }
            CredentialError::FailedToDecodeBase58Credential { source } => {
                error = ImportCredentialError::Generic(source.to_string());
            }
            CredentialError::FailedToGetNextUsableCredential { .. } => {}
            CredentialError::MissingBandwidthTypeAttribute => {}
            CredentialError::FailedToVerifyCredential => {
                error = ImportCredentialError::VerificationFailed
            }
            CredentialError::FailedToCreateNyxdClientConfig(_) => {}
            CredentialError::FailedToConnectUsingNyxdClient(_) => {}
            CredentialError::NoNyxdEndpointsFound => {}
            CredentialError::FailedToQueryContract => {
                error = ImportCredentialError::FailedToQueryContract
            }
            CredentialError::FailedToFetchCoconutApiClients(_) => {}
            CredentialError::FailedToUnpackRawCredential { .. } => {}
            CredentialError::FailedToObtainAggregateVerificationKey(_) => {}
            CredentialError::FailedToPrepareCredentialForSpending(_) => {}

            // TODO: just like we have FailedToImportCredential, we should have a
            // FailedToCheckCredential, which in turn would group a number of the low level errors
            // above
        };
        error
    }
}
