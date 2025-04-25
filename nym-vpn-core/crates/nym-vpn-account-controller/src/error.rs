// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to setup nym-vpn-api client")]
    SetupVpnApiClient(nym_vpn_api_client::VpnApiClientError),

    #[error("Mnemonic store error")]
    MnemonicStore {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Key store error")]
    KeyStore {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Failed to setup account storage paths")]
    StoragePaths(#[source] nym_sdk::Error),

    #[error(transparent)]
    CredentialStorage(#[from] nym_credential_storage::error::StorageError),

    #[error(transparent)]
    PendingCredentialRequestsStorage(#[from] crate::storage::PendingCredentialRequestsStorageError),

    #[error("Failed to setup credential storage")]
    SetupCredentialStorage(#[source] nym_sdk::Error),

    #[error("Failed to setup pending credential requests storage")]
    SetupPendingCredentialRequestsStorage(
        #[source] crate::storage::PendingCredentialRequestsStorageError,
    ),

    #[error("Failed to remove credential storage")]
    RemoveCredentialStorage(std::io::Error),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Failed to parse ticket type: {0}")]
    ParseTicketType(String),
}

impl Error {
    pub fn internal(msg: impl ToString) -> Self {
        Error::Internal(msg.to_string())
    }
}
