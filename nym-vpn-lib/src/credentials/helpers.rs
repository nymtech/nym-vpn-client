use std::path::PathBuf;

use nym_credential_storage::persistent_storage::PersistentStorage;

use nym_sdk::{mixnet::StoragePaths, NymNetworkDetails};
use nym_validator_client::{
    coconut::{all_coconut_api_clients, CoconutApiError},
    nyxd::{error::NyxdError, Config as NyxdClientConfig, NyxdClient},
    QueryHttpRpcNyxdClient,
};
use tracing::debug;

use super::{
    error::{CredentialCoconutApiClientError, CredentialNyxdClientError, CredentialStoreError},
    CredentialError,
};

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

pub(super) fn get_nyxd_client() -> Result<QueryHttpRpcNyxdClient, CredentialNyxdClientError> {
    let network = NymNetworkDetails::new_from_env();
    let config = NyxdClientConfig::try_from_nym_network_details(&network)
        .map_err(|err| CredentialNyxdClientError::FailedToCreateNyxdClientConfig(err))?;

    // Safe to use pick the first one?
    let nyxd_url = network
        .endpoints
        .first()
        .ok_or(CredentialNyxdClientError::NoNyxdEndpointsFound)?
        .nyxd_url();

    debug!("Connecting to nyx validator at: {}", nyxd_url);
    NyxdClient::connect(config, nyxd_url.as_str())
        .map_err(|err| CredentialNyxdClientError::FailedToConnectUsingNyxdClient(err))
}

pub(super) enum CoconutClients {
    Clients(Vec<nym_validator_client::coconut::CoconutApiClient>),
    NoContractAvailable,
}

pub(super) async fn get_coconut_api_clients(
    nyxd_client: QueryHttpRpcNyxdClient,
    epoch_id: u64,
) -> Result<CoconutClients, CredentialCoconutApiClientError> {
    match all_coconut_api_clients(&nyxd_client, epoch_id).await {
        Ok(clients) => Ok(CoconutClients::Clients(clients)),
        Err(CoconutApiError::ContractQueryFailure { source }) => match source {
            NyxdError::NoContractAddressAvailable(_) => Ok(CoconutClients::NoContractAvailable),
            _ => Err(CredentialCoconutApiClientError::FailedToQueryContract),
        },
        Err(err) => Err(CredentialCoconutApiClientError::FailedToFetchCoconutApiClients(err)),
    }
}
