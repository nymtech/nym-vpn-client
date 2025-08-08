// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_sdk::mixnet::Recipient;

use crate::{Error, error::Result};

// optional, until we remove the wireguard feature flag
#[derive(Debug, Copy, Clone)]
pub struct AuthAddress(pub Option<Recipient>);

impl AuthAddress {
    pub(crate) fn try_from_base58_string(address: &str) -> Result<Self> {
        let recipient = Recipient::try_from_base58_string(address).map_err(|source| {
            Error::RecipientFormattingError {
                address: address.to_string(),
                source,
            }
        })?;
        Ok(AuthAddress(Some(recipient)))
    }
}
