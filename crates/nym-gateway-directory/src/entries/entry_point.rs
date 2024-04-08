// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt::{Display, Formatter};

use crate::{error::Result, DescribedGatewayWithLocation};
use nym_sdk::mixnet::NodeIdentity;
use serde::{Deserialize, Serialize};

use super::described_gateway::{
    by_identity, by_location, by_random, by_random_low_latency, LookupGateway,
};

// The entry point is always a gateway identity, or some other entry that can be resolved to a
// gateway identity.
// #[derive(Clone, Debug, Deserialize, Serialize, uniffi::Enum)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum EntryPoint {
    Gateway { identity: NodeIdentity },
    // NOTE: Consider using a crate with strongly typed country codes instead of strings
    Location { location: String },
    RandomLowLatency,
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
    pub fn is_location(&self) -> bool {
        matches!(self, EntryPoint::Location { .. })
    }
}

#[async_trait::async_trait]
impl LookupGateway for EntryPoint {
    async fn lookup_gateway_identity(
        &self,
        gateways: &[DescribedGatewayWithLocation],
    ) -> Result<NodeIdentity> {
        match &self {
            EntryPoint::Gateway { identity } => by_identity(gateways, identity),
            EntryPoint::Location { location } => by_location(gateways, location),
            EntryPoint::RandomLowLatency => by_random_low_latency(gateways).await,
            EntryPoint::Random => by_random(gateways),
        }
    }
}
