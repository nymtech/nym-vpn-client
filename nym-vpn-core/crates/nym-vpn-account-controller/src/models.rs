// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_credentials::{AggregatedCoinIndicesSignatures, AggregatedExpirationDateSignatures};
use serde::{Deserialize, Serialize};

// These are temporarily copy pasted here from the nym-credential-proxy. They will eventually make
// their way into the crates we use through the nym repo.

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WalletShare {
    pub node_index: u64,
    pub bs58_encoded_share: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TicketbookWalletSharesResponse {
    epoch_id: u64,
    shares: Vec<WalletShare>,
    master_verification_key: Option<MasterVerificationKeyResponse>,
    aggregated_coin_index_signatures: Option<AggregatedCoinIndicesSignaturesResponse>,
    aggregated_expiration_date_signatures: Option<AggregatedExpirationDateSignaturesResponse>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MasterVerificationKeyResponse {
    epoch_id: u64,
    bs58_encoded_key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AggregatedCoinIndicesSignaturesResponse {
    signatures: AggregatedCoinIndicesSignatures,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AggregatedExpirationDateSignaturesResponse {
    signatures: AggregatedExpirationDateSignatures,
}
