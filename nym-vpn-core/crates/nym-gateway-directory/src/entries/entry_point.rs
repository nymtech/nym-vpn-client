// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt::{Display, Formatter};

use crate::{error::Result, DescribedGatewayWithLocation, Error};
use nym_sdk::mixnet::NodeIdentity;
use serde::{Deserialize, Serialize};
use tracing::debug;

use super::{
    described_gateway::{
        by_location, by_random, by_random_low_latency, verify_identity, LookupGateway,
    },
    gateway::{Gateway, GatewayList},
};

// The entry point is always a gateway identity, or some other entry that can be resolved to a
// gateway identity.
// #[derive(Clone, Debug, Deserialize, Serialize, uniffi::Enum)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum EntryPoint {
    // An explicit entry gateway identity.
    Gateway { identity: NodeIdentity },
    // Select a random entry gateway in a specific location.
    Location { location: String },
    // Select a random entry gateway but increasey probability of selecting a low latency gateway
    // as determined by ping times.
    RandomLowLatency,
    // Select an entry gateway at random.
    Random,
}

impl Display for EntryPoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EntryPoint::Gateway { identity } => write!(f, "Gateway: {}", identity),
            EntryPoint::Location { location } => write!(f, "Location: {}", location),
            EntryPoint::RandomLowLatency => write!(f, "Random low latency"),
            EntryPoint::Random => write!(f, "Random"),
        }
    }
}

impl EntryPoint {
    pub fn from_base58_string(base58: &str) -> Result<Self> {
        let identity = NodeIdentity::from_base58_string(base58)
            .map_err(|_| Error::NodeIdentityFormattingError)?;
        Ok(EntryPoint::Gateway { identity })
    }

    pub fn is_location(&self) -> bool {
        matches!(self, EntryPoint::Location { .. })
    }

    pub fn lookup_gateway_identity2(&self, gateways: &GatewayList) -> Result<Gateway> {
        match &self {
            EntryPoint::Gateway { identity } => {
                debug!("Selecting gateway by identity: {}", identity);
                gateways
                    .gateway_with_identity(identity)
                    .ok_or_else(|| Error::NoMatchingGateway)
                    .cloned()
            }
            EntryPoint::Location { location } => {
                debug!("Selecting gateway by location: {}", location);
                gateways
                    .random_gateway_located_at(location.to_string())
                    .ok_or_else(|| Error::NoMatchingGatewayForLocation {
                        requested_location: location.clone(),
                        available_countries: gateways.all_iso_codes(),
                    })
            }
            EntryPoint::RandomLowLatency => {
                debug!("Selecting a random low latency gateway");
                todo!("Need to add client address to Gateway type");
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

#[async_trait::async_trait]
impl LookupGateway for EntryPoint {
    async fn lookup_gateway_identity(
        &self,
        gateways: &[DescribedGatewayWithLocation],
    ) -> Result<(NodeIdentity, Option<String>)> {
        match &self {
            EntryPoint::Gateway { identity } => verify_identity(gateways, identity),
            EntryPoint::Location { location } => {
                by_location(gateways, location).map_err(|err| match err {
                    Error::NoMatchingGatewayForLocation {
                        requested_location,
                        available_countries,
                    } => Error::NoMatchingEntryGatewayForLocation {
                        requested_location,
                        available_countries,
                    },
                    err => err,
                })
            }
            EntryPoint::RandomLowLatency => by_random_low_latency(gateways).await,
            EntryPoint::Random => {
                log::info!("Selecting a random entry gateway");
                by_random(gateways)
            }
        }
    }
}
