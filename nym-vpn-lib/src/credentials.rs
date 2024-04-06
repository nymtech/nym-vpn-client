use std::{fs, path::PathBuf};

use nym_sdk::mixnet::StoragePaths;

use crate::error::Result;

pub async fn import_credential(credential_file: PathBuf, config_path: PathBuf) -> Result<()> {
    let storage_path = StoragePaths::new_from_dir(config_path)?;
    let credential_path = storage_path.credential_database_path;
    let credentials_store =
        nym_credential_storage::initialise_persistent_storage(credential_path).await;

    let raw_credential = fs::read(credential_file)?;
    let version = None;

    nym_id::import_credential(credentials_store, raw_credential, version)
        .await
        .unwrap();

    Ok(())
}
