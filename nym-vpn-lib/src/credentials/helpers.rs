use std::path::PathBuf;

use nym_credential_storage::persistent_storage::PersistentStorage;

use nym_sdk::{mixnet::StoragePaths, NymNetworkDetails};
use nym_validator_client::{
    coconut::{all_coconut_api_clients, CoconutApiError},
    nyxd::{error::NyxdError, Config as NyxdClientConfig, NyxdClient},
    QueryHttpRpcNyxdClient,
};
use tracing::debug;

use super::CredentialError;

pub(super) async fn get_credentials_store(
    data_path: PathBuf,
) -> Result<(PersistentStorage, PathBuf), CredentialError> {
    let storage_path = StoragePaths::new_from_dir(data_path)?;
    let credential_db_path = storage_path.credential_database_path;
    debug!("Credential store: {}", credential_db_path.display());
    Ok((
        nym_credential_storage::initialise_persistent_storage(credential_db_path.clone()).await,
        credential_db_path,
    ))
}

pub(super) fn get_nyxd_client() -> Result<QueryHttpRpcNyxdClient, CredentialError> {
    let network = NymNetworkDetails::new_from_env();
    let config = NyxdClientConfig::try_from_nym_network_details(&network)?;

    // Safe to use pick the first one?
    let nyxd_url = network
        .endpoints
        .first()
        .ok_or(CredentialError::NoNyxdEndpointsFound)?
        .nyxd_url();

    debug!("Connecting to nyx validator at: {}", nyxd_url);
    Ok(NyxdClient::connect(config, nyxd_url.as_str())?)
}

pub(super) enum CoconutClients {
    Clients(Vec<nym_validator_client::coconut::CoconutApiClient>),
    NoContactAvailable,
}

pub(super) async fn get_coconut_api_clients(
    nyxd_client: QueryHttpRpcNyxdClient,
    epoch_id: u64,
) -> Result<CoconutClients, CredentialError> {
    match all_coconut_api_clients(&nyxd_client, epoch_id).await {
        Ok(clients) => Ok(CoconutClients::Clients(clients)),
        Err(CoconutApiError::ContractQueryFailure { source }) => match source {
            NyxdError::NoContractAddressAvailable(_) => Ok(CoconutClients::NoContactAvailable),
            _ => Err(CredentialError::FailedToQueryContract),
        },
        Err(err) => Err(err.into()),
    }
}
