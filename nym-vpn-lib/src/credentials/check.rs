use std::{fs, path::PathBuf};

use nym_bandwidth_controller::{BandwidthController, PreparedCredential, RetrievedCredential};
use nym_credentials::{
    coconut::bandwidth::{
        bandwidth_credential_params, issued::BandwidthCredentialIssuedDataVariant,
    },
    obtain_aggregate_verification_key, IssuedBandwidthCredential,
};

use tracing::{debug, info};

use super::{
    helpers::{get_coconut_api_clients, get_credentials_store, get_nyxd_client, CoconutClients},
    CredentialError,
};

pub async fn check_raw_credential(raw_credential: Vec<u8>) -> Result<(), CredentialError> {
    let version = None;
    let credential = IssuedBandwidthCredential::try_unpack(&raw_credential, version)?;

    // Check expiry
    match credential.variant_data() {
        BandwidthCredentialIssuedDataVariant::Voucher(_) => {
            debug!("credential is a bandwidth voucher");
        }
        BandwidthCredentialIssuedDataVariant::FreePass(freepass_info) => {
            debug!("credential is a free pass");
            if freepass_info.expired() {
                return Err(CredentialError::FreepassExpired {
                    expiry_date: freepass_info.expiry_date().to_string(),
                });
            }
        }
    }

    // TODO: verify?

    Ok(())
}

pub async fn check_credential_base58(credential: &str) -> Result<(), CredentialError> {
    let raw_credential = bs58::decode(credential)
        .into_vec()
        .map_err(|err| CredentialError::FailedToDecodeBase58Credential { source: err })?;
    check_raw_credential(raw_credential).await
}

pub async fn check_credential_file(credential_file: PathBuf) -> Result<(), CredentialError> {
    let raw_credential = fs::read(credential_file)?;
    check_raw_credential(raw_credential).await
}

pub async fn check_imported_credential(
    data_path: PathBuf,
    gateway_id: &str,
) -> Result<(), CredentialError> {
    let client = get_nyxd_client()?;
    let (credentials_store, _location) = get_credentials_store(data_path.clone()).await?;
    let bandwidth_controller = BandwidthController::new(credentials_store, client);
    let usable_credential = bandwidth_controller
        .get_next_usable_credential(gateway_id)
        .await
        .map_err(|err| CredentialError::FailedToGetNextUsableCredential {
            location: data_path,
            reason: err.to_string(),
        })?;

    let epoch_id = usable_credential.credential.epoch_id();
    let client = get_nyxd_client()?;
    let coconut_api_clients = match get_coconut_api_clients(client, epoch_id).await? {
        CoconutClients::Clients(clients) => clients,
        CoconutClients::NoContactAvailable => {
            info!("No Coconut API clients on this network, we are ok");
            return Ok(());
        }
    };

    verify_credential(usable_credential, coconut_api_clients).await
}

async fn verify_credential(
    usable_credential: RetrievedCredential,
    coconut_api_clients: Vec<nym_validator_client::coconut::CoconutApiClient>,
) -> Result<(), CredentialError> {
    let verification_key = obtain_aggregate_verification_key(&coconut_api_clients)?;
    let spend_request = usable_credential
        .credential
        .prepare_for_spending(&verification_key)?;
    let prepared_credential = PreparedCredential {
        data: spend_request,
        epoch_id: usable_credential.credential.epoch_id(),
        credential_id: usable_credential.credential_id,
    };

    if !prepared_credential.data.validate_type_attribute() {
        return Err(CredentialError::MissingBandwidthTypeAttribute);
    }

    let params = bandwidth_credential_params();
    if prepared_credential.data.verify(params, &verification_key) {
        info!("Successfully verified credential");
        Ok(())
    } else {
        return Err(CredentialError::FailedToVerifyCredential);
    }
}
