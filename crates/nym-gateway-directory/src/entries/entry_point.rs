// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    error::Result,
    helpers::{
        list_all_country_iso_codes, select_random_gateway_node,
        select_random_low_latency_gateway_node,
    },
    DescribedGatewayWithLocation, Error,
};
use nym_sdk::mixnet::NodeIdentity;
use serde::{Deserialize, Serialize};

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

impl EntryPoint {
    pub fn is_location(&self) -> bool {
        matches!(self, EntryPoint::Location { .. })
    }

    pub async fn lookup_gateway_identity(
        &self,
        gateways: &[DescribedGatewayWithLocation],
    ) -> Result<NodeIdentity> {
        match &self {
            EntryPoint::Gateway { identity } => {
                // Confirm up front that the gateway identity is in the list of gateways from the
                // directory.
                gateways
                    .iter()
                    .find(|gateway| gateway.identity_key() == &identity.to_string())
                    .ok_or(Error::NoMatchingGateway)?;
                Ok(*identity)
            }
            EntryPoint::Location { location } => {
                // Caution: if an explorer-api for a different network was specified, then
                // none of the gateways will have an associated location. There is a check
                // against this earlier in the call stack to guard against this scenario.
                let gateways_with_specified_location = gateways
                    .iter()
                    .filter(|g| g.is_two_letter_iso_country_code(location));
                if gateways_with_specified_location.clone().count() == 0 {
                    return Err(Error::NoMatchingEntryGatewayForLocation {
                        requested_location: location.to_string(),
                        available_countries: list_all_country_iso_codes(gateways),
                    });
                }
                select_random_gateway_node(gateways_with_specified_location)
            }
            EntryPoint::RandomLowLatency => {
                log::info!("Selecting a random low latency entry gateway");
                select_random_low_latency_gateway_node(gateways).await
            }
            EntryPoint::Random => {
                log::info!("Selecting a random entry gateway");
                select_random_gateway_node(gateways)
            }
        }
    }
}
