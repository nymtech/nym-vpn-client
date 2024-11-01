// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio::sync::mpsc::error::SendError;
use url::Url;

use crate::commands::AccountCommand;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to get account summary from nym-vpn-api at {base_url}")]
    GetAccountSummary {
        base_url: Url,
        source: Box<nym_vpn_api_client::VpnApiClientError>,
    },

    #[error("missing API URL")]
    MissingApiUrl,

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

    #[error("failed to register device")]
    RegisterDevice(#[source] nym_vpn_api_client::VpnApiClientError),

    #[error("failed to get devices")]
    GetDevices(#[source] nym_vpn_api_client::VpnApiClientError),

    #[error("failed to send account controller command")]
    AccountCommandSend {
        #[from]
        source: SendError<AccountCommand>,
    },

    #[error("failed to construct withdrawal request")]
    ConstructWithdrawalRequest(#[source] nym_compact_ecash::CompactEcashError),

    #[error("failed to send get zk-nyms request")]
    GetZkNyms(#[source] nym_vpn_api_client::VpnApiClientError),

    #[error("failed to send request zk-nym request")]
    RequestZkNym(#[source] nym_vpn_api_client::VpnApiClientError),

    #[error(transparent)]
    HttpClient(#[from] nym_http_api_client::HttpClientError),

    #[error("trying to use epoch before it's available")]
    NoEpoch,

    #[error("failed to import zk-nym")]
    ImportZkNym(#[source] nym_compact_ecash::CompactEcashError),

    #[error("failed to create ecash key pair")]
    CreateEcashKeyPair(#[source] nym_vpn_api_client::VpnApiClientError),

    #[error("internal error: {0}")]
    Internal(String),

    #[error(transparent)]
    NymSdk(#[from] nym_sdk::Error),

    #[error("succesfull zknym response is missing blinded shares")]
    MissingBlindedShares,
}

impl Error {
    pub fn internal(msg: impl ToString) -> Self {
        Error::Internal(msg.to_string())
    }
}
