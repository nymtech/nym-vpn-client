// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use bincode::Options;
use nym_credentials_interface::RequestInfo;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::Date;
use zeroize::Zeroize;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PendingCredentialRequestStored {
    pub id: String,
    pub expiration_date: Date,
    pub request_info: Vec<u8>,
}

impl Drop for PendingCredentialRequestStored {
    fn drop(&mut self) {
        self.id.zeroize();
        self.request_info.zeroize();
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingCredentialRequest {
    pub id: String,
    pub expiration_date: Date,
    pub request_info: RequestInfo,
}

impl Drop for PendingCredentialRequest {
    fn drop(&mut self) {
        self.id.zeroize();
        self.request_info.zeroize();
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

fn binary_serialiser() -> impl Options {
    bincode::DefaultOptions::new()
        .with_big_endian()
        .with_varint_encoding()
}
