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

    #[error("failed to setup credential storage")]
    SetupCredentialStorage(#[source] nym_sdk::Error),

    #[error("failed to send account controller command")]
    AccountCommandSend {
        #[from]
        source: SendError<AccountCommand>,
    },

    #[error("failed to receive account controller result")]
    AccountCommandRecv {
        #[from]
        source: tokio::sync::oneshot::error::RecvError,
    },

    #[error("failed to construct withdrawal request")]
    ConstructWithdrawalRequest(#[source] nym_compact_ecash::CompactEcashError),

    #[error("failed to send get zk-nyms request")]
    GetZkNyms(#[source] nym_vpn_api_client::VpnApiClientError),

    #[error("failed to send request zk-nym request")]
    RequestZkNym(#[source] nym_vpn_api_client::VpnApiClientError),

    #[error("failed to send confirm zk-nym download")]
    ConfirmZkNymDownload(#[source] nym_vpn_api_client::VpnApiClientError),

    #[error("failed to import zk-nym")]
    ImportZkNym(#[source] nym_compact_ecash::CompactEcashError),

    #[error("failed to create ecash key pair")]
    CreateEcashKeyPair(#[source] nym_vpn_api_client::VpnApiClientError),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("succesfull zknym response is missing blinded shares")]
    MissingBlindedShares,

    #[error("missing master verification key")]
    MissingMasterVerificationKey,

    #[error("invalid master verification key: {0}")]
    InvalidMasterVerificationKey(#[source] nym_compact_ecash::CompactEcashError),

    #[error("invalid verification key: {0}")]
    InvalidVerificationKey(#[source] nym_compact_ecash::CompactEcashError),

    #[error("failed to deserialize blinded signature")]
    DeserializeBlindedSignature(nym_compact_ecash::CompactEcashError),

    #[error("inconsistent epoch id")]
    InconsistentEpochId,

    #[error("decoded key missing index")]
    DecodedKeysMissingIndex,

    #[error("failed to aggregate wallets")]
    AggregateWallets(#[source] nym_compact_ecash::CompactEcashError),

    #[error("failed to parse ticket type: {0}")]
    ParseTicketType(String),

    #[error("credential storage not initialized")]
    CredentialStorageNotInitialized,
}

impl Error {
    pub fn internal(msg: impl ToString) -> Self {
        Error::Internal(msg.to_string())
    }
}
