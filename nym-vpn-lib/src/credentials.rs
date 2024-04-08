use std::{fs, path::PathBuf};

use nym_sdk::mixnet::StoragePaths;

use crate::error::{Error, Result};

// Import binary credential data
pub async fn import_credential(credential: Vec<u8>, data_path: PathBuf) -> Result<()> {
    let storage_path = StoragePaths::new_from_dir(data_path)?;
    let credential_path = storage_path.credential_database_path;
    let credentials_store =
        nym_credential_storage::initialise_persistent_storage(credential_path).await;

    let version = None;

    nym_id::import_credential(credentials_store, credential, version)
        .await
        .map_err(|err| Error::FailedToImportCredential { source: err })
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
