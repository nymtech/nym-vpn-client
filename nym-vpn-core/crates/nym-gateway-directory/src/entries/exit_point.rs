// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt::{Display, Formatter};

use crate::{
    entries::described_gateway::{by_location_described, by_random_described},
    error::Result,
    DescribedGatewayWithLocation, Error, MixAddresses,
};
use nym_sdk::mixnet::{NodeIdentity, Recipient};
use serde::{Deserialize, Serialize};

use super::described_gateway::{
    by_identity, by_location, by_random, verify_identity, LookupGateway,
};

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
    // Select an exit gateway at random.
    Random,
}

impl Display for ExitPoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ExitPoint::Address { address } => write!(f, "Address: {}", address),
            ExitPoint::Gateway { identity } => write!(f, "Gateway: {}", identity),
            ExitPoint::Location { location } => write!(f, "Location: {}", location),
            ExitPoint::Random => write!(f, "Random"),
        }
    }
}

impl ExitPoint {
    pub fn is_location(&self) -> bool {
        matches!(self, ExitPoint::Location { .. })
    }

    pub fn lookup_mix_addresses(
        &self,
        gateways: &[DescribedGatewayWithLocation],
        entry_gateway: Option<&NodeIdentity>,
    ) -> Result<(MixAddresses, Option<String>)> {
        match &self {
            ExitPoint::Address { address } => {
                // There is no validation done when a ip packet router is specified by address
                // since it might be private and not available in any directory.
                Ok((
                    MixAddresses {
                        ip_packet_router_address: *address,
                        authenticator_address: None,
                    },
                    None,
                ))
            }
            ExitPoint::Gateway { identity } => {
                let gateway = by_identity(gateways, identity)?;
                Ok((
                    MixAddresses::try_from_described_gateway(&gateway.gateway)?,
                    gateway.two_letter_iso_country_code(),
                ))
            }
            ExitPoint::Location { location } => {
                log::info!("Selecting a random exit gateway in location: {}", location);
                let exit_gateways = gateways
                    .iter()
                    .filter(|g| g.has_ip_packet_router())
                    .filter(|g| g.is_current_build())
                    .cloned()
                    .collect::<Vec<_>>();

                // If there is only one exit gateway available and it is the entry gateway, we
                // should not use it as the exit gateway.
                if exit_gateways.len() == 1
                    && exit_gateways[0].node_identity().as_ref() == entry_gateway
                {
                    return Err(Error::OnlyAvailableExitGatewayIsTheEntryGateway {
                        requested_location: location.clone(),
                        gateway: Box::new(exit_gateways[0].clone()),
                    });
                }

                let exit_gateways = exit_gateways
                    .into_iter()
                    .filter(|g| g.node_identity().as_ref() != entry_gateway)
                    .collect::<Vec<_>>();

                let gateway = by_location_described(&exit_gateways, location)?;
                Ok((
                    MixAddresses::try_from_described_gateway(&gateway.gateway)?,
                    gateway.two_letter_iso_country_code(),
                ))
            }
            ExitPoint::Random => {
                log::info!("Selecting a random exit gateway");
                let exit_gateways = gateways
                    .iter()
                    .filter(|g| g.has_ip_packet_router())
                    .filter(|g| g.is_current_build())
                    .filter(|g| g.node_identity().as_ref() != entry_gateway)
                    .cloned()
                    .collect::<Vec<_>>();
                let gateway = by_random_described(&exit_gateways)?;
                Ok((
                    MixAddresses::try_from_described_gateway(&gateway.gateway)?,
                    gateway.two_letter_iso_country_code(),
                ))
            }
        }
    }
}

#[async_trait::async_trait]
impl LookupGateway for ExitPoint {
    async fn lookup_gateway_identity(
        &self,
        gateways: &[DescribedGatewayWithLocation],
    ) -> Result<(NodeIdentity, Option<String>)> {
        match &self {
            ExitPoint::Address { .. } => Err(Error::InvalidExitPointDescription),
            ExitPoint::Gateway { identity } => verify_identity(gateways, identity),
            ExitPoint::Location { location } => {
                by_location(gateways, location).map_err(|e| match e {
                    Error::NoMatchingGatewayForLocation {
                        requested_location,
                        available_countries,
                    } => Error::NoMatchingExitGatewayForLocation {
                        requested_location,
                        available_countries,
                    },
                    e => e,
                })
            }
            ExitPoint::Random => {
                log::info!("Selecting a random exit gateway");
                by_random(gateways)
            }
        }
    }
}
