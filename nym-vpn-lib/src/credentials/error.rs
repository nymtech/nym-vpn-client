#[derive(Debug, thiserror::Error)]
pub enum CredentialError {
    #[error(transparent)]
    NymSdkError(#[from] nym_sdk::Error),

    #[error(transparent)]
    NymCredentialsError(#[from] nym_credentials::Error),

    #[error(transparent)]
    NymCredentialStorageError(#[from] nym_credential_storage::error::StorageError),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error("the free pass has already expired! The expiration was set to {expiry_date}")]
    FreepassExpired { expiry_date: String },

    #[error("failed to import credential to: {location}: {source}")]
    FailedToImportCredential {
        location: std::path::PathBuf,
        source: nym_id::NymIdError,
    },

    #[error("failed decode base58 credential: {source}")]
    FailedToDecodeBase58Credential { source: bs58::decode::Error },

    #[error("failed to get next usable credential: {reason}")]
    FailedToGetNextUsableCredential {
        location: std::path::PathBuf,
        reason: String,
    },

    #[error("missing bandwidth type attribute")]
    MissingBandwidthTypeAttribute,

    #[error("failed to verify credential")]
    FailedToVerifyCredential,

    #[error("failed to get nyxd client: {0}")]
    NyxdError(#[from] nym_validator_client::nyxd::error::NyxdError),

    #[error("no nyxd endpoints found")]
    NoNyxdEndpointsFound,

    #[error("failed to query contract")]
    FailedToQueryContract,

    #[error("failed to fetch coconut api clients: {0}")]
    FailedToFetchCoconutApiClients(nym_validator_client::coconut::CoconutApiError),
}
