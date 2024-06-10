use std::path::PathBuf;

use nym_credential_storage::persistent_storage::PersistentStorage;

use nym_sdk::{mixnet::StoragePaths, NymNetworkDetails};
use nym_validator_client::{
    nyxd::{Config as NyxdClientConfig, NyxdClient},
    QueryHttpRpcNyxdClient,
};
use tracing::debug;

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

pub(super) async fn get_credentials_store(
    data_path: PathBuf,
) -> Result<(PersistentStorage, PathBuf), CredentialStoreError> {
    // Create data_path if it doesn't exist
    std::fs::create_dir_all(&data_path).map_err(|err| {
        CredentialStoreError::FailedToCreateCredentialStoreDirectory {
            path: data_path.clone(),
            source: err,
        }
    })?;

    let storage_path = StoragePaths::new_from_dir(data_path.clone()).map_err(|err| {
        CredentialStoreError::FailedToSetupStoragePaths {
            path: data_path.clone(),
            source: err,
        }
    })?;
    let credential_db_path = storage_path.credential_database_path;
    debug!("Credential store: {}", credential_db_path.display());
    let storage = nym_credential_storage::persistent_storage::PersistentStorage::init(
        credential_db_path.clone(),
    )
    .await
    .map_err(
        |err| CredentialStoreError::FailedToInitializePersistentStorage {
            path: credential_db_path.clone(),
            source: err,
        },
    )?;

    #[cfg(target_family = "unix")]
    {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        let metadata = fs::metadata(&credential_db_path).map_err(|err| {
            CredentialStoreError::FailedToReadCredentialStoreMetadata {
                path: credential_db_path.clone(),
                source: err,
            }
        })?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o600);
        fs::set_permissions(&credential_db_path, permissions).map_err(|err| {
            CredentialStoreError::FailedToSetCredentialStorePermissions {
                path: credential_db_path.clone(),
                source: err,
            }
        })?;
    }

    Ok((storage, credential_db_path))
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

pub(super) fn get_nyxd_client() -> Result<QueryHttpRpcNyxdClient, CredentialNyxdClientError> {
    let network = NymNetworkDetails::new_from_env();
    let config = NyxdClientConfig::try_from_nym_network_details(&network)
        .map_err(CredentialNyxdClientError::FailedToCreateNyxdClientConfig)?;

    // Safe to use pick the first one?
    let nyxd_url = network
        .endpoints
        .first()
        .ok_or(CredentialNyxdClientError::NoNyxdEndpointsFound)?
        .nyxd_url();

    debug!("Connecting to nyx validator at: {}", nyxd_url);
    NyxdClient::connect(config, nyxd_url.as_str())
        .map_err(CredentialNyxdClientError::FailedToConnectUsingNyxdClient)
}

#[derive(Debug, thiserror::Error)]
pub enum CredentialCoconutApiClientError {
    #[error("failed to query contract")]
    FailedToQueryContract,

    #[error("failed to fetch coconut api clients: {0}")]
    FailedToFetchCoconutApiClients(nym_validator_client::coconut::CoconutApiError),
}
