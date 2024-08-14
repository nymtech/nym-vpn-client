// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use crate::coconut::utils::scalar_serde_helper;
use nym_credentials_interface::{
    hash_to_scalar, Attribute, 
    PublicAttribute,
};
use nym_crypto::asymmetric::{encryption, identity};
use nym_validator_client::nyxd::{Coin, Hash};
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Debug, Zeroize, ZeroizeOnDrop, Serialize, Deserialize)]
pub struct BandwidthVoucherIssuedData {
    /// the plain value (e.g., bandwidth) encoded in this voucher
    // note: for legacy reasons we're only using the value of the coin and ignoring the denom
    #[zeroize(skip)]
    value: Coin,
}

impl BandwidthVoucherIssuedData {
    pub fn new(value: Coin) -> Self {
        BandwidthVoucherIssuedData { value }
    }

    pub fn value(&self) -> &Coin {
        &self.value
    }

    pub fn value_plain(&self) -> String {
        self.value.amount.to_string()
    }
}

#[derive(Zeroize, ZeroizeOnDrop, Serialize, Deserialize)]
pub struct BandwidthVoucherIssuanceData {
    /// the plain value (e.g., bandwidth) encoded in this voucher
    // note: for legacy reasons we're only using the value of the coin and ignoring the denom
    #[zeroize(skip)]
    value: Coin,

    // note: as mentioned above, we're only hashing the value of the coin!
    #[serde(with = "scalar_serde_helper")]
    value_prehashed: PublicAttribute,

    /// the hash of the deposit transaction
    #[zeroize(skip)]
    deposit_tx_hash: Hash,

    /// base58 encoded private key ensuring the depositer requested these attributes
    signing_key: identity::PrivateKey,

    /// base58 encoded private key ensuring only this client receives the signature share
    unused_ed25519: encryption::PrivateKey,
}

impl BandwidthVoucherIssuanceData {
    pub fn new(
        value: impl Into<Coin>,
        deposit_tx_hash: Hash,
        signing_key: identity::PrivateKey,
        unused_ed25519: encryption::PrivateKey,
    ) -> Self {
        let value = value.into();
        let value_prehashed = hash_to_scalar(value.amount.to_string());

        BandwidthVoucherIssuanceData {
            value,
            value_prehashed,
            deposit_tx_hash,
            signing_key,
            unused_ed25519,
        }
    }

    pub fn value_plain(&self) -> String {
        self.value.amount.to_string()
    }

    pub fn value_attribute(&self) -> &Attribute {
        &self.value_prehashed
    }
}
