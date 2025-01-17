// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use bincode::Options;
use nym_credentials_interface::RequestInfo;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::Date;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Zeroize, ZeroizeOnDrop)]
pub struct PendingCredentialRequestStored {
    pub id: String,
    #[zeroize(skip)]
    pub expiration_date: Date,
    pub request_info: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct PendingCredentialRequest {
    pub id: String,
    #[zeroize(skip)]
    pub expiration_date: Date,
    pub request_info: RequestInfo,
}

impl TryFrom<PendingCredentialRequestStored> for PendingCredentialRequest {
    type Error = bincode::Error;

    fn try_from(value: PendingCredentialRequestStored) -> Result<Self, Self::Error> {
        let request_info = binary_serialiser().deserialize(&value.request_info)?;
        Ok(Self {
            id: value.id.clone(),
            expiration_date: value.expiration_date,
            request_info,
        })
    }
}

impl TryFrom<PendingCredentialRequest> for PendingCredentialRequestStored {
    type Error = bincode::Error;

    fn try_from(value: PendingCredentialRequest) -> Result<Self, Self::Error> {
        let request_info = binary_serialiser().serialize(&value.request_info)?;
        Ok(Self {
            id: value.id.clone(),
            expiration_date: value.expiration_date,
            request_info,
        })
    }
}

fn binary_serialiser() -> impl bincode::Options {
    use bincode::Options;
    bincode::DefaultOptions::new()
        .with_big_endian()
        .with_varint_encoding()
}
