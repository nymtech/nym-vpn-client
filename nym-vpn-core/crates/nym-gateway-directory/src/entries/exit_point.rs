// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use crate::{
    entries::described_gateway::{by_location_described, by_random_described},
    error::Result,
    DescribedGatewayWithLocation, Error, IpPacketRouterAddress,
};
use nym_sdk::mixnet::{NodeIdentity, Recipient};
use serde::{Deserialize, Serialize};
use tracing::debug;

use super::{
    described_gateway::by_identity,
    gateway::{Gateway, GatewayList},
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

    // DEPRECATED: will be removed from nym-gateway-probe and then we can delete it
    pub fn lookup_router_address(
        &self,
        gateways: &[DescribedGatewayWithLocation],
        entry_gateway: Option<&NodeIdentity>,
    ) -> Result<(IpPacketRouterAddress, Option<String>)> {
        match &self {
            ExitPoint::Address { address } => {
                // There is no validation done when a ip packet router is specified by address
                // since it might be private and not available in any directory.
                Ok((IpPacketRouterAddress(*address), None))
            }
            ExitPoint::Gateway { identity } => {
                let gateway = by_identity(gateways, identity)?;
                Ok((
                    IpPacketRouterAddress::try_from_described_gateway(&gateway.gateway)?,
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
                    IpPacketRouterAddress::try_from_described_gateway(&gateway.gateway)?,
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
                    IpPacketRouterAddress::try_from_described_gateway(&gateway.gateway)?,
                    gateway.two_letter_iso_country_code(),
                ))
            }
        }
    }

    pub fn lookup_gateway(&self, gateways: &GatewayList) -> Result<Gateway> {
        match &self {
            ExitPoint::Address { address } => {
                debug!("Selecting gateway by address: {}", address);
                // There is no validation done when a ip packet router is specified by address
                // since it might be private and not available in any directory.
                let ipr_address = IpPacketRouterAddress(*address);
                let gateway_address = ipr_address.gateway();

                gateways
                    .gateway_with_identity(gateway_address)
                    .ok_or(Error::NoMatchingGateway)
                    .cloned()
            }
            ExitPoint::Gateway { identity } => {
                debug!("Selecting gateway by identity: {}", identity);
                gateways
                    .gateway_with_identity(identity)
                    .ok_or_else(|| Error::NoMatchingGateway)
                    .cloned()
            }
            ExitPoint::Location { location } => {
                debug!("Selecting gateway by location: {}", location);
                gateways
                    .random_gateway_located_at(location.to_string())
                    .ok_or_else(|| Error::NoMatchingExitGatewayForLocation {
                        requested_location: location.clone(),
                        available_countries: gateways.all_iso_codes(),
                    })
            }
            ExitPoint::Random => {
                log::info!("Selecting a random exit gateway");
                gateways
                    .random_gateway()
                    .ok_or_else(|| Error::FailedToSelectGatewayRandomly)
            }
        }
    }
}

pub fn extract_router_address(
    gateways: &[DescribedGatewayWithLocation],
    identity_key: String,
) -> Result<IpPacketRouterAddress> {
    Ok(IpPacketRouterAddress(
        Recipient::from_str(
            &gateways
                .iter()
                .find(|gw| *gw.gateway.bond.identity() == identity_key)
                .ok_or(Error::NoMatchingGateway)?
                .gateway
                .self_described
                .clone()
                .ok_or(Error::NoGatewayDescriptionAvailable(identity_key))?
                .ip_packet_router
                .ok_or(Error::MissingIpPacketRouterAddress)?
                .address,
        )
        .map_err(|_| Error::RecipientFormattingError)?,
    ))
}
