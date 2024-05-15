use std::{fs, path::PathBuf};
use tracing::info;

use super::{
    error::{ImportCredentialBase58Error, ImportCredentialError, ImportCredentialFileError},
    helpers::get_credentials_store,
};

// Import binary credential data
pub async fn import_credential(
    raw_credential: Vec<u8>,
    data_path: PathBuf,
) -> Result<(), ImportCredentialError> {
    info!("Importing credential");
    let (credentials_store, location) =
        get_credentials_store(data_path.clone())
            .await
            .map_err(|err| ImportCredentialError::CredentialStoreError {
                path: data_path,
                source: err,
            })?;
    let version = None;
    nym_id::import_credential(credentials_store, raw_credential, version)
        .await
        .map_err(|err| ImportCredentialError::FailedToImportRawCredential {
            location,
            source: err,
        })
}

// Import credential data from a base58 string
pub async fn import_credential_base58(
    credential: &str,
    data_path: PathBuf,
) -> Result<(), ImportCredentialBase58Error> {
    let raw_credential = bs58::decode(credential).into_vec()?;
    import_credential(raw_credential, data_path)
        .await
        .map_err(|err| err.into())
}

// Import credential data from a binary file
pub async fn import_credential_file(
    credential_file: PathBuf,
    data_path: PathBuf,
) -> Result<(), ImportCredentialFileError> {
    let raw_credential = fs::read(credential_file.clone()).map_err(|err| {
        ImportCredentialFileError::ImportCredentialFile {
            path: credential_file,
            source: err,
        }
    })?;
    import_credential(raw_credential, data_path)
        .await
        .map_err(|err| ImportCredentialFileError::ImportCredential { source: err })
}
