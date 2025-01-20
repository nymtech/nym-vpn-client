// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio::sync::mpsc::error::SendError;

use crate::commands::AccountCommand;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to setup nym-vpn-api client")]
    SetupVpnApiClient(nym_vpn_api_client::VpnApiClientError),

    #[error("mnemonic store error")]
    MnemonicStore {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("key store error")]
    KeyStore {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("failed to setup account storage paths")]
    StoragePaths(#[source] nym_sdk::Error),

    #[error(transparent)]
    CredentialStorage(#[from] nym_credential_storage::error::StorageError),

    #[error(transparent)]
    PendingCredentialRequestsStorage(#[from] crate::storage::PendingCredentialRequestsStorageError),

    #[error("failed to setup credential storage")]
    SetupCredentialStorage(#[source] nym_sdk::Error),

    #[error("failed to setup pending credential requests storage")]
    SetupPendingCredentialRequestsStorage(
        #[source] crate::storage::PendingCredentialRequestsStorageError,
    ),

    #[error("failed to remove credential storage: {0}")]
    RemoveCredentialStorage(std::io::Error),

    #[error("failed to send account controller command")]
    AccountCommandSend {
        #[from]
        source: SendError<AccountCommand>,
    },

    #[error("failed to send get zk-nyms request")]
    GetZkNyms(#[source] nym_vpn_api_client::VpnApiClientError),

    #[error("failed to send confirm zk-nym download")]
    ConfirmZkNymDownload(#[source] nym_vpn_api_client::VpnApiClientError),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("failed to parse ticket type: {0}")]
    ParseTicketType(String),
}

impl Error {
    pub fn internal(msg: impl ToString) -> Self {
        Error::Internal(msg.to_string())
    }
}
