// Copyright 2021 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use crate::coconut::bandwidth::issued::CURRENT_SERIALIZATION_REVISION;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to (de)serialize provided credential using revision {revision}: {source}")]
    SerializationFailure {
        #[source]
        source: bincode::Error,
        revision: u8,
    },

    #[error("unknown credential serializatio revision {revision}. the current (and max supported) version is {CURRENT_SERIALIZATION_REVISION}")]
    UnknownSerializationRevision { revision: u8 },
}
