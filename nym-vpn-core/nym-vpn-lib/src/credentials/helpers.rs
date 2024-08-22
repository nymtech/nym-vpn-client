use std::path::{Path, PathBuf};

use nym_credential_storage_pre_ecash::persistent_storage::PersistentStorage;

use nym_sdk::{mixnet::StoragePaths, NymNetworkDetails};
use nym_validator_client::{
    nyxd::{Config as NyxdClientConfig, NyxdClient},
    QueryHttpRpcNyxdClient,
};
use sqlx::{ConnectOptions as _, Row as _};
use tracing::debug;

const PRE_ECASH_DB_MIGRATION_VERSION: i64 = 20241104120000;

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
        source: nym_credential_storage_pre_ecash::error::StorageError,
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

    #[error("failed to connect to db for getting the version: {0}")]
    FailedToConnectoToDbForFetchingVersion(#[source] sqlx::Error),

    #[error("failed to fetch db version: {0}")]
    FailedToFetchDbVersion(#[source] sqlx::Error),

    #[error("failed to copy old db file: {0}")]
    FailedToCopyOldDbFile(std::io::Error),
}

fn forked_db_path(db_path: &Path) -> PathBuf {
    db_path.with_file_name(format!(
        "fork_{}",
        db_path.file_name().unwrap().to_str().unwrap()
    ))
}

async fn is_db_old(db_path: &Path) -> Result<bool, CredentialStoreError> {
    let mut opts = sqlx::sqlite::SqliteConnectOptions::new().filename(db_path);
    opts.disable_statement_logging();
    let pool = sqlx::SqlitePool::connect_with(opts)
        .await
        .map_err(CredentialStoreError::FailedToConnectoToDbForFetchingVersion)?;

    let row = sqlx::query("SELECT MAX(version) as version FROM _sqlx_migrations")
        .fetch_one(&pool)
        .await
        .map_err(CredentialStoreError::FailedToFetchDbVersion)?;

    let migration_version: i64 = row.get("version");
    Ok(migration_version == PRE_ECASH_DB_MIGRATION_VERSION)
}

async fn copy_old_db_file(db_path: &Path, new_db_path: &Path) -> Result<u64, CredentialStoreError> {
    debug!("Copying old db file to {}", new_db_path.display());
    std::fs::copy(db_path, new_db_path).map_err(CredentialStoreError::FailedToCopyOldDbFile)
}

async fn migrate_to_forked_credential_db(
    credential_db_path: &Path,
) -> Result<PathBuf, CredentialStoreError> {
    let fork_credential_db_path = forked_db_path(credential_db_path);
    if !fork_credential_db_path.exists()
        && credential_db_path.exists()
        && is_db_old(credential_db_path).await?
    {
        copy_old_db_file(credential_db_path, &fork_credential_db_path).await?;
    };
    Ok(fork_credential_db_path)
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

    // For the freepasses we need to work with a forked db copy as part of the transition to ecash.
    // The credential path will used again later in the connection phase by the mixnet client where
    // it will be migrated to a newer schema, and hence become incompatible with this client
    // credential check.
    let fork_credential_db_path = migrate_to_forked_credential_db(&credential_db_path).await?;
    debug!(
        "Forked credential store: {}",
        fork_credential_db_path.display()
    );

    let storage = nym_credential_storage_pre_ecash::persistent_storage::PersistentStorage::init(
        fork_credential_db_path.clone(),
    )
    .await
    .map_err(
        |err| CredentialStoreError::FailedToInitializePersistentStorage {
            path: fork_credential_db_path.clone(),
            source: err,
        },
    )?;

    #[cfg(target_family = "unix")]
    {
        use std::{fs, os::unix::fs::PermissionsExt};

        let metadata = fs::metadata(&fork_credential_db_path).map_err(|err| {
            CredentialStoreError::FailedToReadCredentialStoreMetadata {
                path: fork_credential_db_path.clone(),
                source: err,
            }
        })?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o600);
        fs::set_permissions(&fork_credential_db_path, permissions).map_err(|err| {
            CredentialStoreError::FailedToSetCredentialStorePermissions {
                path: fork_credential_db_path.clone(),
                source: err,
            }
        })?;
    }

    Ok((storage, fork_credential_db_path))
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
