use std::{fs, path::PathBuf};

use anyhow::{anyhow, bail};
use nym_bandwidth_controller::PreparedCredential;
use nym_credential_storage::{persistent_storage::PersistentStorage, storage::Storage};
use nym_credentials::{
    coconut::bandwidth::{
        bandwidth_credential_params, issued::BandwidthCredentialIssuedDataVariant,
    },
    obtain_aggregate_verification_key, IssuedBandwidthCredential,
};
use nym_sdk::{mixnet::StoragePaths, NymNetworkDetails};
use nym_validator_client::{
    coconut::{all_coconut_api_clients, CoconutApiError},
    nyxd::{error::NyxdError, Config as NyxdClientConfig, NyxdClient},
};
use tracing::{debug, info, warn};

use crate::error::{Error, Result};

// Import binary credential data
pub async fn import_credential(credential: Vec<u8>, data_path: PathBuf) -> Result<()> {
    info!("Importing credential");
    let (credentials_store, location) = get_credentials_store(data_path).await?;
    let version = None;
    nym_id::import_credential(credentials_store, credential, version)
        .await
        .map_err(|err| Error::FailedToImportCredential {
            location,
            source: err,
        })
}

// Import credential data from a base58 string
pub async fn import_credential_base58(credential: &str, data_path: PathBuf) -> Result<()> {
    let raw_credential = bs58::decode(credential)
        .into_vec()
        .map_err(|err| Error::FailedToDecodeBase58Credential { source: err })?;
    import_credential(raw_credential, data_path).await
}

// Import credential data from a binary file
pub async fn import_credential_file(credential_file: PathBuf, data_path: PathBuf) -> Result<()> {
    let raw_credential = fs::read(credential_file)?;
    import_credential(raw_credential, data_path).await
}

pub async fn check_imported_credential(data_path: PathBuf, gateway_id: &str) -> anyhow::Result<()> {
    debug!("Checking imported credential data");
    let (credentials_store, _location) = get_credentials_store(data_path).await?;

    let (valid_credential, credential_id) =
        fetch_valid_credential(&credentials_store, gateway_id).await?;

    let epoch_id = valid_credential.epoch_id();
    let coconut_api_clients = match get_coconut_api_clients(epoch_id).await? {
        CoconutClients::Clients(clients) => clients,
        CoconutClients::NoContactAvailable => {
            info!("No Coconut API clients on this network, we are ok");
            return Ok(());
        }
    };

    verify_credential(&valid_credential, credential_id, coconut_api_clients).await
}

async fn get_credentials_store(data_path: PathBuf) -> Result<(PersistentStorage, PathBuf)> {
    let storage_path = StoragePaths::new_from_dir(data_path)?;
    let credential_db_path = storage_path.credential_database_path;
    info!("Credential store: {}", credential_db_path.display());
    Ok((
        nym_credential_storage::initialise_persistent_storage(credential_db_path.clone()).await,
        credential_db_path,
    ))
}

async fn fetch_valid_credential(
    credentials_store: &PersistentStorage,
    gateway_id: &str,
) -> anyhow::Result<(IssuedBandwidthCredential, i64)> {
    debug!(
        "Checking if there is an unspent credential for gateway: {}",
        gateway_id
    );

    let stored_issued_credential = credentials_store
        .get_next_unspent_credential(gateway_id)
        .await?
        .ok_or(anyhow!("No unspent credentials found"))?;

    debug!("Found unspent credential: {}", stored_issued_credential.id);

    let credential_id = stored_issued_credential.id;
    let issued_credential =
        IssuedBandwidthCredential::unpack_v1(&stored_issued_credential.credential_data)?;

    let valid_credential = match issued_credential.variant_data() {
        BandwidthCredentialIssuedDataVariant::Voucher(_) => {
            debug!("Credential {credential_id} is a voucher");
            issued_credential
        }
        BandwidthCredentialIssuedDataVariant::FreePass(freepass_info) => {
            debug!("Credential {credential_id} is a free pass");
            if freepass_info.expired() {
                warn!(
                    "the free pass (id: {credential_id}) has already expired! The expiration was set to {}",
                    freepass_info.expiry_date()
                );
                credentials_store.mark_expired(credential_id).await?;
                bail!("credential {credential_id} has already expired");
            }
            issued_credential
        }
    };
    Ok((valid_credential, credential_id))
}

enum CoconutClients {
    Clients(Vec<nym_validator_client::coconut::CoconutApiClient>),
    NoContactAvailable,
}

async fn get_coconut_api_clients(epoch_id: u64) -> anyhow::Result<CoconutClients> {
    let network = NymNetworkDetails::new_from_env();
    let config = NyxdClientConfig::try_from_nym_network_details(&network)?;

    // Safe to use pick the first one?
    let nyxd_url = network
        .endpoints
        .first()
        .ok_or(anyhow!("No nyxd endpoints found"))?
        .nyxd_url();

    info!("Connecting to nyx validator at: {}", nyxd_url);
    let nyxd_client = NyxdClient::connect(config, nyxd_url.as_str())?;

    match all_coconut_api_clients(&nyxd_client, epoch_id).await {
        Ok(clients) => Ok(CoconutClients::Clients(clients)),
        Err(CoconutApiError::ContractQueryFailure { source }) => match source {
            NyxdError::NoContractAddressAvailable(_) => Ok(CoconutClients::NoContactAvailable),
            _ => bail!("failed to query contract"),
        },
        Err(err) => Err(err.into()),
    }
}

async fn verify_credential(
    valid_credential: &IssuedBandwidthCredential,
    credential_id: i64,
    coconut_api_clients: Vec<nym_validator_client::coconut::CoconutApiClient>,
) -> anyhow::Result<()> {
    let verification_key = obtain_aggregate_verification_key(&coconut_api_clients)?;
    let spend_request = valid_credential.prepare_for_spending(&verification_key)?;
    let prepared_credential = PreparedCredential {
        data: spend_request,
        epoch_id: valid_credential.epoch_id(),
        credential_id,
    };

    if !prepared_credential.data.validate_type_attribute() {
        bail!("missing bandwidth type attribute");
    }

    let params = bandwidth_credential_params();
    if prepared_credential.data.verify(params, &verification_key) {
        info!("Successfully validated credential");
        Ok(())
    } else {
        bail!("failed to validate credential");
    }
}
