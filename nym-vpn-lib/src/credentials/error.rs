use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum CredentialError {
    #[error("failed to setup storage paths: {path}: {source}")]
    FailedToSetupStoragePaths {
        path: PathBuf,
        source: nym_sdk::Error,
    },

    #[error("failed to create credential store directory: {path}: {source}")]
    FailedToCreateCredentialStoreDirectory {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to read credential store metadata: {path}: {source}")]
    FailedToReadCredentialStoreMetadata {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to set credential store permissions: {path}: {source}")]
    FailedToSetCredentialStorePermissions {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to read credential file: {path}: {source}")]
    FailedToReadCredentialFile {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to initialize persistent storage: {path}: {source}")]
    FailedToInitializePersistentStorage {
        path: PathBuf,
        source: nym_credential_storage::error::StorageError,
    },

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

    #[error("failed to create nyxd client config: {0}")]
    FailedToCreateNyxdClientConfig(nym_validator_client::nyxd::error::NyxdError),

    #[error("failed to connect using nyxd client: {0}")]
    FailedToConnectUsingNyxdClient(nym_validator_client::nyxd::error::NyxdError),

    #[error("no nyxd endpoints found")]
    NoNyxdEndpointsFound,

    #[error("failed to query contract")]
    FailedToQueryContract,

    #[error("failed to unpack raw credential: {source}")]
    FailedToUnpackRawCredential { source: nym_credentials::Error },

    #[error("failed to fetch coconut api clients: {0}")]
    FailedToFetchCoconutApiClients(nym_validator_client::coconut::CoconutApiError),

    #[error("failed to obtain aggregate key")]
    FailedToObtainAggregateVerificationKey(nym_credentials::Error),

    #[error("failed to prepare credential for spending")]
    FailedToPrepareCredentialForSpending(nym_credentials::Error),
}
