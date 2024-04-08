// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt::{Display, Formatter};

use crate::{
    error::Result, helpers::list_all_country_iso_codes, DescribedGatewayWithLocation, Error,
    IpPacketRouterAddress,
};
use nym_sdk::mixnet::{NodeIdentity, Recipient};
use rand::seq::IteratorRandom;
use serde::{Deserialize, Serialize};

use super::described_gateway::{by_identity, by_location, LookupGateway};

// The exit point is a nym-address, but if the exit ip-packet-router is running embedded on a
// gateway, we can refer to it by the gateway identity.
// #[derive(Clone, Debug, Deserialize, Serialize, uniffi::Enum)]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(clippy::large_enum_variant)]
pub enum ExitPoint {
    // An explicit exit address. This is useful when the exit ip-packet-router is running as a
    // standalone entity (private).
    Address { address: Recipient },
    // An explicit exit gateway identity. This is useful when the exit ip-packet-router is running
    // embedded on a gateway.
    Gateway { identity: NodeIdentity },
    // NOTE: Consider using a crate with strongly typed country codes instead of strings
    Location { location: String },
}

impl Display for ExitPoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ExitPoint::Address { address } => write!(f, "Address: {}", address),
            ExitPoint::Gateway { identity } => write!(f, "Gateway: {}", identity),
            ExitPoint::Location { location } => write!(f, "Location: {}", location),
        }
    }
}

impl ExitPoint {
    pub fn is_location(&self) -> bool {
        matches!(self, ExitPoint::Location { .. })
    }

    pub fn lookup_router_address(
        &self,
        gateways: &[DescribedGatewayWithLocation],
    ) -> Result<IpPacketRouterAddress> {
        match &self {
            ExitPoint::Address { address } => {
                // There is no validation done when a ip packet router is specified by address
                // since it might be private and not available in any directory.
                Ok(IpPacketRouterAddress(*address))
            }
            ExitPoint::Gateway { identity } => {
                let gateway = gateways
                    .iter()
                    .find(|gateway| gateway.identity_key() == &identity.to_string())
                    .ok_or(Error::NoMatchingGateway)?;
                IpPacketRouterAddress::try_from_described_gateway(&gateway.gateway)
            }
            ExitPoint::Location { location } => {
                let exit_gateways = gateways.iter().filter(|g| g.has_ip_packet_router());
                let gateways_with_specified_location = exit_gateways
                    .clone()
                    .filter(|gateway| gateway.is_two_letter_iso_country_code(location));
                let random_gateway =
                    gateways_with_specified_location.choose(&mut rand::thread_rng());

                match random_gateway {
                    Some(random_gateway) => {
                        IpPacketRouterAddress::try_from_described_gateway(&random_gateway.gateway)
                    }
                    None => Err(Error::NoMatchingExitGatewayForLocation {
                        requested_location: location.to_string(),
                        available_countries: list_all_country_iso_codes(exit_gateways),
                    }),
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl LookupGateway for ExitPoint {
    async fn lookup_gateway_identity(
        &self,
        gateways: &[DescribedGatewayWithLocation],
    ) -> Result<NodeIdentity> {
        match &self {
            ExitPoint::Address { .. } => Err(Error::InvalidExitPointDescription),
            ExitPoint::Gateway { identity } => by_identity(gateways, identity),
            ExitPoint::Location { location } => by_location(gateways, location),
        }
    }
}
