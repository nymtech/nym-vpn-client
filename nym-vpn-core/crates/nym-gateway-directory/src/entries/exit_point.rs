// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt::{Display, Formatter};

use nym_sdk::mixnet::{NodeIdentity, Recipient};
use serde::{Deserialize, Serialize};

use super::gateway::{Gateway, GatewayList};
use crate::{Error, IpPacketRouterAddress, error::Result};

// The exit point is a nym-address, but if the exit ip-packet-router is running embedded on a
// gateway, we can refer to it by the gateway identity.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum ExitPoint {
    // An explicit exit address. This is useful when the exit ip-packet-router is running as a
    // standalone entity (private).
    Address { address: Box<Recipient> },

    // An explicit exit gateway identity. This is useful when the exit ip-packet-router is running
    // embedded on a gateway.
    Gateway { identity: NodeIdentity },

    // Select a random entry gateway in a specific country.
    Country { two_letter_iso_country_code: String },

    // Select a random entry gateway in a specific region/state.
    Region { region: String },

    // Select an exit gateway at random.
    Random,
}

impl Display for ExitPoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ExitPoint::Address { address } => write!(f, "Address: {address}"),
            ExitPoint::Gateway { identity } => write!(f, "Gateway: {identity}"),
            ExitPoint::Country {
                two_letter_iso_country_code,
            } => write!(f, "Country: {two_letter_iso_country_code}"),
            ExitPoint::Region { region } => write!(f, "Region/state: {region}"),
            ExitPoint::Random => write!(f, "Random"),
        }
    }
}

impl ExitPoint {
    pub fn lookup_gateway(&self, gateways: &GatewayList) -> Result<Gateway> {
        match &self {
            ExitPoint::Address { address } => {
                tracing::debug!("Selecting gateway by address: {address}");
                // There is no validation done when a ip packet router is specified by address
                // since it might be private and not available in any directory.
                let ipr_address = IpPacketRouterAddress::from(**address);
                let gateway_address = ipr_address.gateway();

                // Now fetch the gateway that the IPR is connected to, and override its IPR address
                let mut gateway = gateways
                    .gateway_with_identity(&gateway_address)
                    .ok_or_else(|| Error::NoMatchingGateway {
                        requested_identity: gateway_address.to_string(),
                    })
                    .cloned()?;
                gateway.ipr_address = Some(ipr_address);
                Ok(gateway)
            }
            ExitPoint::Gateway { identity } => {
                tracing::debug!("Selecting gateway by identity: {identity}");
                gateways
                    .gateway_with_identity(identity)
                    .ok_or_else(|| Error::NoMatchingGateway {
                        requested_identity: identity.to_string(),
                    })
                    .cloned()
            }
            ExitPoint::Country {
                two_letter_iso_country_code,
            } => {
                tracing::debug!("Selecting gateway by country: {two_letter_iso_country_code}");
                gateways
                    .random_gateway_located_at_country(two_letter_iso_country_code.to_string())
                    .ok_or_else(|| Error::NoMatchingExitGatewayForLocation {
                        requested_location: two_letter_iso_country_code.clone(),
                        available_countries: gateways.all_iso_codes(),
                    })
            }
            ExitPoint::Region { region } => {
                tracing::debug!("Selecting gateway by region/state: {region}");
                gateways
                    .random_gateway_located_at_region(region.to_string())
                    .ok_or_else(|| Error::NoMatchingExitGatewayForLocation {
                        requested_location: region.clone(),
                        available_countries: gateways.all_iso_codes(),
                    })
            }
            ExitPoint::Random => {
                tracing::debug!("Selecting a random exit gateway");
                gateways
                    .random_gateway()
                    .ok_or_else(|| Error::FailedToSelectGatewayRandomly)
            }
        }
    }
}
