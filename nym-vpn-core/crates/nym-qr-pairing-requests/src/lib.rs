// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

#[derive(Debug, Serialize, Deserialize)]
pub struct PairAccountRequestBody {
    pub account_id: String,
    pub pubkey: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PairingCheckRequestBody {
    pub account_id: String,
    pub pubkey: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PairingConfirmPinRequestBody {
    pub account_id: String,
    pub pubkey: String,
    pub enc_pin_code: Vec<u8>,
    pub nonce: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TryToPairRequestBody {
    pub own_pubkey: String,
    pub remote_pubkey: String,
    #[serde(with = "nym_serde_helpers::base64")]
    pub encrypted_pin_code: Vec<u8>,
    #[serde(with = "nym_serde_helpers::base64")]
    pub nonce: Vec<u8>,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TryToGetSecretRequestBody {
    pub remote_pubkey: String,
    pub pairing_token: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Zeroize)]
pub struct SharePairSecretBody {
    pub own_pubkey: String,
    pub account_id: String,
    #[serde(with = "nym_serde_helpers::base64")]
    pub enc_mnemonic: Vec<u8>,
    #[serde(with = "nym_serde_helpers::base64")]
    pub nonce: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NymVpnPairingResponse {
    pub ttl_seconds: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NymVpnPairingFinalisedResponse {
    #[serde(with = "nym_serde_helpers::base64")]
    pub encrypted_pin_code: Vec<u8>,
    #[serde(with = "nym_serde_helpers::base64")]
    pub nonce: Vec<u8>,
    pub remote_pub_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NymVpnPairingCheckResponse {
    pub inner: Option<NymVpnPairingFinalisedResponse>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum NymVpnGetPairingSecretResponse {
    Nothing,
    PinConfirmation(PairingConfirmPinRequestBody),
    Secret(SharePairSecretBody),
}
