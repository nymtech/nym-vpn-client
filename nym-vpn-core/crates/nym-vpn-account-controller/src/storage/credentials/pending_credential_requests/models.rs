// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use bincode::Options;
use nym_compact_ecash::VerificationKeyAuth;
use nym_credential_storage::persistent_storage::PersistentStorage as PersistentCredentialStorage;
use nym_credentials::{
    AggregatedCoinIndicesSignatures, AggregatedExpirationDateSignatures, EpochVerificationKey,
    IssuedTicketBook,
};
use nym_credentials_interface::{
    AnnotatedCoinIndexSignature, AnnotatedExpirationDateSignature, RequestInfo, TicketType,
};
use nym_sdk::mixnet::{CredentialStorage, StoragePaths};
use nym_vpn_api_client::types::{Device, VpnApiAccount};
use nym_vpn_store::{mnemonic::Mnemonic, VpnStorage};
use serde::{Deserialize, Serialize};
use sqlx::{ConnectOptions, FromRow};
use time::Date;
use tracing::log::LevelFilter;

use crate::{error::Error, AvailableTicketbooks};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PendingCredentialRequestStored {
    // WIP: remove pub
    pub(crate) id: String,
    // WIP: remove pub
    pub(crate) expiration_date: Date,
    // WIP: remove pub
    pub(crate) request_info: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingCredentialRequest {
    pub(crate) id: String,
    pub(crate) expiration_date: Date,
    pub(crate) request_info: RequestInfo,
}

impl TryFrom<PendingCredentialRequestStored> for PendingCredentialRequest {
    type Error = bincode::Error;

    fn try_from(value: PendingCredentialRequestStored) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            expiration_date: value.expiration_date,
            request_info: binary_serialiser().deserialize(&value.request_info)?,
        })
    }
}

impl TryFrom<PendingCredentialRequest> for PendingCredentialRequestStored {
    type Error = bincode::Error;

    fn try_from(value: PendingCredentialRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            expiration_date: value.expiration_date,
            request_info: binary_serialiser().serialize(&value.request_info)?,
        })
    }
}

fn binary_serialiser() -> impl bincode::Options {
    use bincode::Options;
    bincode::DefaultOptions::new()
        .with_big_endian()
        .with_varint_encoding()
}
