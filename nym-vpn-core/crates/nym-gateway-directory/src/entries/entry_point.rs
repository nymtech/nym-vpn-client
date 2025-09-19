// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt::{Display, Formatter};

use nym_sdk::mixnet::NodeIdentity;
use serde::{Deserialize, Serialize};
use tracing::debug;

use super::gateway::{Gateway, GatewayList};
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

    pub async fn lookup_gateway(&self, gateways: &GatewayList) -> Result<Gateway> {
        match &self {
            EntryPoint::Gateway { identity } => {
                debug!("Selecting gateway by identity: {identity}");
                gateways
                    .gateway_with_identity(identity)
                    .ok_or_else(|| Error::NoMatchingGateway {
                        requested_identity: identity.to_string(),
                    })
                    .cloned()
            }
            EntryPoint::Country {
                two_letter_iso_country_code,
            } => {
                debug!("Selecting gateway by country: {two_letter_iso_country_code}");
                gateways
                    .random_gateway_located_at_country(two_letter_iso_country_code.to_string())
                    .ok_or_else(|| Error::NoMatchingEntryGatewayForLocation {
                        requested_location: two_letter_iso_country_code.clone(),
                        available_countries: gateways.all_iso_codes(),
                    })
            }
            EntryPoint::Region { region } => {
                debug!("Selecting gateway by region/state: {region}");
                gateways
                    .random_gateway_located_at_region(region.to_string())
                    .ok_or_else(|| Error::NoMatchingEntryGatewayForLocation {
                        requested_location: region.clone(),
                        available_countries: gateways.all_iso_codes(),
                    })
            }
            EntryPoint::Random => {
                debug!("Selecting a random gateway");
                gateways
                    .random_gateway()
                    .ok_or_else(|| Error::FailedToSelectGatewayRandomly)
            }
        }
    }
}
