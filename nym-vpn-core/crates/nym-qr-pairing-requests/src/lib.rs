// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

pub(crate) const PAIRING_TOKEN: &str = "pairing_token";

#[derive(Debug, Serialize, Deserialize)]
pub struct PairAccountRequestBody {
    pub account_id: String,
    pub pubkey: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TryToPairRequestBody {
    pub pubkey: String,
    pub pin_code: String,
    pub token: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Zeroize)]
pub struct SharePairSecretBody {
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
    pub pin_code: String,
    pub remote_pub_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NymVpnPairingCheckResponse {
    pub inner: Option<NymVpnPairingFinalisedResponse>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NymVpnGetPairingSecretResponse {
    pub inner: Option<SharePairSecretBody>,
}

#[derive(Clone, Default, Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct PairingParams {
    pub pairing_token: String,
}

impl PairingParams {
    pub fn to_param(&self) -> Vec<(String, String)> {
        vec![(PAIRING_TOKEN.to_string(), self.pairing_token.clone())]
    }
}
