// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt::{Display, Formatter};

use nym_sdk::mixnet::NodeIdentity;
use serde::{Deserialize, Serialize};

use crate::{Error, error::Result};

// The entry point is always a gateway identity, or some other entry that can be resolved to a
// gateway identity.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub enum EntryPoint {
    // An explicit entry gateway identity.
    Gateway { identity: NodeIdentity },
    // Select a random entry gateway in a specific country.
    Country { two_letter_iso_country_code: String },
    // Select a random entry gateway in a specific region/state.
    Region { region: String },
    // Select an entry gateway at random.
    Random,
}

impl Display for EntryPoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EntryPoint::Gateway { identity } => write!(f, "Gateway: {identity}"),
            EntryPoint::Country {
                two_letter_iso_country_code,
            } => write!(f, "Country: {two_letter_iso_country_code}"),
            EntryPoint::Region { region } => write!(f, "Region/state: {region}"),
            EntryPoint::Random => write!(f, "Random"),
        }
    }
}

impl EntryPoint {
    pub fn from_base58_string(base58: &str) -> Result<Self> {
        let identity = NodeIdentity::from_base58_string(base58).map_err(|source| {
            Error::NodeIdentityFormattingError {
                identity: base58.to_string(),
                source,
            }
        })?;
        Ok(EntryPoint::Gateway { identity })
    }
}
