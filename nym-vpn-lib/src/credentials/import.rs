use std::{fs, path::PathBuf};
use tracing::info;

use super::{helpers::get_credentials_store, CredentialError};

// Import binary credential data
pub async fn import_credential(
    raw_credential: Vec<u8>,
    data_path: PathBuf,
) -> Result<(), CredentialError> {
    info!("Importing credential");
    let (credentials_store, location) = get_credentials_store(data_path).await?;
    let version = None;
    nym_id::import_credential(credentials_store, raw_credential, version)
        .await
        .map_err(|err| CredentialError::FailedToImportCredential {
            location,
            source: err,
        })
}

// Import credential data from a base58 string
pub async fn import_credential_base58(
    credential: &str,
    data_path: PathBuf,
) -> Result<(), CredentialError> {
    let raw_credential = bs58::decode(credential)
        .into_vec()
        .map_err(|err| CredentialError::FailedToDecodeBase58Credential { source: err })?;
    import_credential(raw_credential, data_path).await
}

// Import credential data from a binary file
pub async fn import_credential_file(
    credential_file: PathBuf,
    data_path: PathBuf,
) -> Result<(), CredentialError> {
    let raw_credential = fs::read(credential_file.clone()).map_err(|err| {
        CredentialError::FailedToReadCredentialFile {
            path: credential_file,
            source: err,
        }
    })?;
    import_credential(raw_credential, data_path).await
}
