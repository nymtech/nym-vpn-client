// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use nym_validator_client::nyxd::Coin;
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
    pub fn value(&self) -> &Coin {
        &self.value
    }
}
