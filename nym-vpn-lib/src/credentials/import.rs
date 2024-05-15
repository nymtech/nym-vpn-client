use std::{fs, path::PathBuf};
use tracing::info;

use super::{
    helpers::get_credentials_store, CredentialStoreError,
};

#[derive(Debug, thiserror::Error)]
pub enum ImportCredentialError {
    #[error("failed to import credential to: {location}: {source}")]
    FailedToImportRawCredential {
        location: std::path::PathBuf,
        source: nym_id::NymIdError,
    },

    #[error("credential store error: {path}: {source}")]
    CredentialStoreError {
        path: std::path::PathBuf,
        source: CredentialStoreError,
    },
}

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

#[derive(Debug, thiserror::Error)]
pub enum ImportCredentialBase58Error {
    #[error("failed to decode base58 credential: {source}")]
    FailedToDecodeBase58 {
        #[from]
        source: bs58::decode::Error,
    },

    #[error("failed to import credential to: {source}")]
    FailedToImportRawCredential {
        #[from]
        source: ImportCredentialError,
    },
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

#[derive(Debug, thiserror::Error)]
pub enum ImportCredentialFileError {
    #[error("failed to read credential file: {path}: {source}")]
    ReadCredentialFile {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to import credential file: {path}: {source}")]
    ImportCredentialFile {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error(transparent)]
    ImportCredential { source: ImportCredentialError },
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
