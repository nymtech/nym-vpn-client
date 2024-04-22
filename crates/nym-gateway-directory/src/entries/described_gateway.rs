// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    error::{Error, Result},
    helpers::*,
};
use chrono::{DateTime, Utc};
use nym_explorer_client::Location;
use nym_sdk::mixnet::NodeIdentity;
use nym_validator_client::{client::IdentityKey, models::DescribedGateway};

const BUILD_VERSION: &str = "1.1.34";
const BUILD_TIME: &str = "2024-03-25T10:47:53.981548588Z";

#[derive(Clone, Debug)]
pub struct DescribedGatewayWithLocation {
    pub gateway: DescribedGateway,
    pub location: Option<Location>,
}

impl DescribedGatewayWithLocation {
    pub fn identity_key(&self) -> &IdentityKey {
        &self.gateway.bond.gateway.identity_key
    }

    pub fn has_ip_packet_router(&self) -> bool {
        self.gateway
            .self_described
            .as_ref()
            .and_then(|d| d.ip_packet_router.as_ref())
            .is_some()
    }

    pub fn is_current_build(&self) -> bool {
        self.has_current_build_timestamp() && self.has_current_build_version()
    }

    fn has_current_build_timestamp(&self) -> bool {
        let expected_build_time: DateTime<Utc> = BUILD_TIME.parse().expect("Invalid timestamp");
        self.build_timestamp()
            .map_or(false, |d| d >= expected_build_time)
    }

    fn build_timestamp(&self) -> Option<DateTime<Utc>> {
        self.gateway.self_described.as_ref().map(|d| {
            d.build_information
                .build_timestamp
                .parse::<DateTime<Utc>>()
                .ok()
        })?
    }

    //can make this more flexible with backwards compatibility
    fn has_current_build_version(&self) -> bool {
        self.gateway
            .self_described
            .as_ref()
            .map(|d| d.build_information.build_version == BUILD_VERSION)
            .is_some()
    }

    pub fn has_location(&self) -> bool {
        self.location.is_some()
    }

    pub fn location(&self) -> Option<Location> {
        self.location.clone()
    }

    pub fn two_letter_iso_country_code(&self) -> Option<String> {
        self.location
            .as_ref()
            .map(|l| l.two_letter_iso_country_code.clone())
    }

    pub fn is_two_letter_iso_country_code(&self, code: &str) -> bool {
        self.two_letter_iso_country_code()
            .map_or(false, |gateway_iso_code| gateway_iso_code == code)
    }

    pub fn country_name(&self) -> Option<String> {
        self.location.as_ref().map(|l| l.country_name.clone())
    }
}

impl From<DescribedGateway> for DescribedGatewayWithLocation {
    fn from(gateway: DescribedGateway) -> Self {
        DescribedGatewayWithLocation {
            gateway,
            location: None,
        }
    }
}

#[async_trait::async_trait]
pub trait LookupGateway {
    async fn lookup_gateway_identity(
        &self,
        gateways: &[DescribedGatewayWithLocation],
    ) -> Result<(NodeIdentity, Option<String>)>;
}

pub fn by_identity(
    gateways: &[DescribedGatewayWithLocation],
    identity: &NodeIdentity,
) -> Result<(NodeIdentity, Option<String>)> {
    // Confirm up front that the gateway identity is in the list of gateways from the
    // directory.
    gateways
        .iter()
        .find(|gateway| gateway.identity_key() == &identity.to_string())
        .ok_or(Error::NoMatchingGateway)?;
    Ok((*identity, None))
}

pub fn by_location(
    gateways: &[DescribedGatewayWithLocation],
    location: &str,
) -> Result<(NodeIdentity, Option<String>)> {
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

pub async fn by_random_low_latency(
    gateways: &[DescribedGatewayWithLocation],
) -> Result<(NodeIdentity, Option<String>)> {
    log::info!("Selecting a random low latency entry gateway");
    select_random_low_latency_gateway_node(gateways).await
}

pub fn by_random(
    gateways: &[DescribedGatewayWithLocation],
) -> Result<(NodeIdentity, Option<String>)> {
    log::info!("Selecting a random entry gateway");
    select_random_gateway_node(gateways)
}
