#[derive(Debug, thiserror::Error)]
pub enum CredentialError {
    #[error("the free pass has already expired! The expiration was set to {expiry_date}")]
    FreepassExpired { expiry_date: String },

    #[error("failed to import credential to: {location}: {source}")]
    FailedToImportCredential {
        location: std::path::PathBuf,
        source: nym_id::NymIdError,
    },

    #[error("failed decode base58 credential: {source}")]
    FailedToDecodeBase58Credential { source: bs58::decode::Error },

    #[error("failed to get next usable credential from {location}: {reason}")]
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

    #[error("{0}")]
    CoconutApiError(#[from] nym_validator_client::coconut::CoconutApiError),

    #[error("{0}")]
    NymSdkError(#[from] nym_sdk::Error),

    #[error("{0}")]
    NymCredentialsError(#[from] nym_credentials::Error),

    #[error("{0}")]
    IoError(#[from] std::io::Error),
}
