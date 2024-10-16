// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio::sync::mpsc::error::SendError;

use crate::AccountCommand;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to get account summary")]
    GetAccountSummary(#[source] nym_vpn_api_client::VpnApiClientError),

    #[error("missing API URL")]
    MissingApiUrl,

    #[error("invalid API URL")]
    InvalidApiUrl,

    #[error("mnemonic store error")]
    MnemonicStore {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("key store error")]
    KeyStore {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

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
}
