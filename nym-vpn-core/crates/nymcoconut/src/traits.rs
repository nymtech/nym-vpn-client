// Copyright 2021-2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

#![warn(clippy::expect_used)]
#![warn(clippy::unwrap_used)]

use crate::CoconutError;

pub(crate) trait Bytable
where
    Self: Sized,
{
    fn to_byte_vec(&self) -> Vec<u8>;

    fn try_from_byte_slice(slice: &[u8]) -> Result<Self, CoconutError>;
}

pub(crate) trait Base58
where
    Self: Bytable,
{
    fn try_from_bs58<S: AsRef<str>>(x: S) -> Result<Self, CoconutError> {
        let bs58_decoded = &bs58::decode(x.as_ref()).into_vec()?;
        Self::try_from_byte_slice(bs58_decoded)
    }
    fn to_bs58(&self) -> String {
        bs58::encode(self.to_byte_vec()).into_string()
    }
}
