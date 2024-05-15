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

    #[error("failed to import base58 credential: {source}")]
    FailedToImportBase58Credential { source: bs58::decode::Error },

    #[error("failed to import credential file: {path}: {source}")]
    FailedToImportCredentialFile {
        path: PathBuf,
        source: std::io::Error,
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

#[derive(Debug, thiserror::Error)]
pub enum ImportCredentialError {
    #[error("failed to import credential to: {location}: {source}")]
    FailedToImportRawCredential {
        location: std::path::PathBuf,
        source: nym_id::NymIdError,
    },

    #[error("credential store error: {path}: {source}")]
    CredentialStoreError {
        path: std::path::PathBuf,
        source: CredentialStoreError,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum ImportCredentialBase58Error {
    #[error("failed to decode base58 credential: {source}")]
    FailedToDecodeBase58 {
        #[from]
        source: bs58::decode::Error,
    },

    #[error("failed to import credential to: {source}")]
    FailedToImportRawCredential {
        #[from]
        source: ImportCredentialError,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum ImportCredentialFileError {
    #[error("failed to read credential file: {path}: {source}")]
    FailedToReadCredentialFile {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to import credential file: {path}: {source}")]
    FailedToImportCredentialFile {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error(transparent)]
    FailedToImportCredential {
        source: ImportCredentialError,
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CredentialStoreError {
    #[error("failed to create credential store directory: {path}: {source}")]
    FailedToCreateCredentialStoreDirectory {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to setup storage paths: {path}: {source}")]
    FailedToSetupStoragePaths {
        path: PathBuf,
        source: nym_sdk::Error,
    },

    #[error("failed to initialize persistent storage: {path}: {source}")]
    FailedToInitializePersistentStorage {
        path: PathBuf,
        source: nym_credential_storage::error::StorageError,
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
}

#[derive(Debug, thiserror::Error)]
pub enum CheckRawCredentialError {
    #[error("failed to unpack raw credential: {source}")]
    FailedToUnpackRawCredential { source: nym_credentials::Error },

    #[error("the free pass has already expired! The expiration was set to {expiry_date}")]
    FreepassExpired { expiry_date: String },
}

#[derive(Debug, thiserror::Error)]
pub enum CheckBase58CredentialError {
    #[error("failed decode base58 credential: {source}")]
    FailedToDecodeBase58Credential {
        #[from]
        source: bs58::decode::Error,
    },

    #[error(transparent)]
    FailedToCheckRawCredential {
        #[from]
        source: CheckRawCredentialError,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum CheckFileCredentialError {
    #[error("failed to read credential file: {path}: {source}")]
    FailedToReadCredentialFile {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error(transparent)]
    FailedToCheckRawCredential {
        #[from]
        source: CheckRawCredentialError,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum CheckImportedCredentialError {
    #[error("failed to get next usable credential: {reason}")]
    FailedToGetNextUsableCredential {
        location: std::path::PathBuf,
        reason: String,
    },

    #[error(transparent)]
    CredentialStoreError {
        #[from]
        source: CredentialStoreError,
    },

    #[error(transparent)]
    VerifyCredentialError {
        #[from]
        source: VerifyCredentialError,
    },

    #[error(transparent)]
    NyxdClientError {
        #[from]
        source: CredentialNyxdClientError,
    },

    #[error(transparent)]
    CoconutApiClientError {
        #[from]
        source: CredentialCoconutApiClientError,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum VerifyCredentialError {
    #[error("failed to obtain aggregate key")]
    FailedToObtainAggregateVerificationKey(nym_credentials::Error),

    #[error("failed to prepare credential for spending")]
    FailedToPrepareCredentialForSpending(nym_credentials::Error),

    #[error("missing bandwidth type attribute")]
    MissingBandwidthTypeAttribute,

    #[error("failed to verify credential")]
    FailedToVerifyCredential,
}

#[derive(Debug, thiserror::Error)]
pub enum CredentialNyxdClientError {
    #[error("failed to create nyxd client config: {0}")]
    FailedToCreateNyxdClientConfig(nym_validator_client::nyxd::error::NyxdError),

    #[error("no nyxd endpoints found")]
    NoNyxdEndpointsFound,

    #[error("failed to connect using nyxd client: {0}")]
    FailedToConnectUsingNyxdClient(nym_validator_client::nyxd::error::NyxdError),
}

#[derive(Debug, thiserror::Error)]
pub enum CredentialCoconutApiClientError {
    #[error("failed to query contract")]
    FailedToQueryContract,

    #[error("failed to fetch coconut api clients: {0}")]
    FailedToFetchCoconutApiClients(nym_validator_client::coconut::CoconutApiError),
}
