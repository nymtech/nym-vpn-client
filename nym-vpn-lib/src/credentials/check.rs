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
    error::{
        CheckBase58CredentialError, CheckFileCredentialError, CheckImportedCredentialError,
        CheckRawCredentialError, VerifyCredentialError,
    },
    helpers::{get_coconut_api_clients, get_credentials_store, get_nyxd_client, CoconutClients},
};

pub async fn check_raw_credential(raw_credential: Vec<u8>) -> Result<(), CheckRawCredentialError> {
    let version = None;
    let credential = IssuedBandwidthCredential::try_unpack(&raw_credential, version)
        .map_err(|err| CheckRawCredentialError::FailedToUnpackRawCredential { source: err })?;

    // Check expiry
    match credential.variant_data() {
        BandwidthCredentialIssuedDataVariant::Voucher(_) => {
            debug!("credential is a bandwidth voucher");
        }
        BandwidthCredentialIssuedDataVariant::FreePass(freepass_info) => {
            debug!("credential is a free pass");
            if freepass_info.expired() {
                return Err(CheckRawCredentialError::FreepassExpired {
                    expiry_date: freepass_info.expiry_date().to_string(),
                });
            }
        }
    }

    // TODO: verify?

    Ok(())
}

pub async fn check_credential_base58(credential: &str) -> Result<(), CheckBase58CredentialError> {
    let raw_credential = bs58::decode(credential).into_vec()?;
    // .map_err(|err| CredentialError::FailedToDecodeBase58Credential { source: err })?;
    check_raw_credential(raw_credential)
        .await
        .map_err(|err| err.into())
}

pub async fn check_credential_file(
    credential_file: PathBuf,
) -> Result<(), CheckFileCredentialError> {
    let raw_credential = fs::read(credential_file.clone()).map_err(|err| {
        CheckFileCredentialError::FailedToReadCredentialFile {
            path: credential_file,
            source: err,
        }
    })?;
    check_raw_credential(raw_credential)
        .await
        .map_err(|err| err.into())
}

pub async fn check_imported_credential(
    data_path: PathBuf,
    gateway_id: &str,
) -> Result<(), CheckImportedCredentialError> {
    let client = get_nyxd_client()?;
    let (credentials_store, _location) = get_credentials_store(data_path.clone()).await?;
    let bandwidth_controller = BandwidthController::new(credentials_store, client);
    let usable_credential = bandwidth_controller
        .get_next_usable_credential(gateway_id)
        .await
        .map_err(
            |err| CheckImportedCredentialError::FailedToGetNextUsableCredential {
                location: data_path,
                reason: err.to_string(),
            },
        )?;

    let epoch_id = usable_credential.credential.epoch_id();
    let client = get_nyxd_client()?;
    let coconut_api_clients = match get_coconut_api_clients(client, epoch_id).await? {
        CoconutClients::Clients(clients) => clients,
        CoconutClients::NoContractAvailable => {
            info!("No Coconut API clients on this network, we are ok");
            return Ok(());
        }
    };

    verify_credential(usable_credential, coconut_api_clients)
        .await
        .map_err(|err| CheckImportedCredentialError::VerifyCredentialError { source: err })
}

async fn verify_credential(
    usable_credential: RetrievedCredential,
    coconut_api_clients: Vec<nym_validator_client::coconut::CoconutApiClient>,
) -> Result<(), VerifyCredentialError> {
    let verification_key = obtain_aggregate_verification_key(&coconut_api_clients)
        .map_err(VerifyCredentialError::FailedToObtainAggregateVerificationKey)?;
    let spend_request = usable_credential
        .credential
        .prepare_for_spending(&verification_key)
        .map_err(VerifyCredentialError::FailedToPrepareCredentialForSpending)?;
    let prepared_credential = PreparedCredential {
        data: spend_request,
        epoch_id: usable_credential.credential.epoch_id(),
        credential_id: usable_credential.credential_id,
    };

    if !prepared_credential.data.validate_type_attribute() {
        return Err(VerifyCredentialError::MissingBandwidthTypeAttribute);
    }

    let params = bandwidth_credential_params();
    if prepared_credential.data.verify(params, &verification_key) {
        info!("Successfully verified credential");
        Ok(())
    } else {
        Err(VerifyCredentialError::FailedToVerifyCredential)
    }
}
