// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{error::Result, Error};
use nym_sdk::mixnet::{NodeIdentity, Recipient};
use nym_validator_client::models::DescribedGateway;

#[derive(Debug, Copy, Clone)]
pub struct IpPacketRouterAddress(pub Recipient);

impl IpPacketRouterAddress {
    pub fn try_from_base58_string(ip_packet_router_nym_address: &str) -> Result<Self> {
        Ok(Self(
            Recipient::try_from_base58_string(ip_packet_router_nym_address).map_err(|_source| {
                Error::RecipientFormattingError {
                    address: ip_packet_router_nym_address.to_string(),
                    //source,
                }
            })?,
        ))
    }

    pub fn try_from_described_gateway(gateway: &DescribedGateway) -> Result<Self> {
        let address = gateway
            .self_described
            .clone()
            .and_then(|described_gateway| described_gateway.ip_packet_router)
            .map(|ipr| ipr.address)
            .ok_or(Error::MissingIpPacketRouterAddress)?;
        Ok(Self(Recipient::try_from_base58_string(&address).map_err(
            |_source| Error::RecipientFormattingError { address },
        )?))
    }

    pub fn gateway(&self) -> &NodeIdentity {
        self.0.gateway()
    }
}

impl std::fmt::Display for IpPacketRouterAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
