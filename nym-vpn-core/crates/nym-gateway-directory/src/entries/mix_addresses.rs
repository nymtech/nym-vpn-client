// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{error::Result, Error};
use nym_sdk::mixnet::{NodeIdentity, Recipient};
use nym_validator_client::models::DescribedGateway;

#[derive(Debug, Copy, Clone)]
pub struct MixAddresses {
    pub ip_packet_router_address: Recipient,
    // optional, until we remove the wireguard feature flag
    pub authenticator_address: Option<Recipient>,
}

impl MixAddresses {
    pub fn try_from_described_gateway(gateway: &DescribedGateway) -> Result<Self> {
        let ip_packet_router_address = Recipient::try_from_base58_string(
            gateway
                .self_described
                .clone()
                .and_then(|described_gateway| described_gateway.ip_packet_router)
                .map(|ipr| ipr.address)
                .ok_or(Error::MissingIpPacketRouterAddress)?,
        )
        .map_err(|_| Error::RecipientFormattingError)?;
        let authenticator_address = gateway
            .self_described
            .clone()
            .and_then(|described_gateway| described_gateway.authenticator)
            .map(|ipr| ipr.address)
            .map(Recipient::try_from_base58_string)
            .transpose()
            .map_err(|_| Error::RecipientFormattingError)?;
        Ok(Self {
            ip_packet_router_address,
            authenticator_address,
        })
    }

    pub fn gateway(&self) -> &NodeIdentity {
        self.ip_packet_router_address.gateway()
    }
}

impl std::fmt::Display for MixAddresses {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ip_packet_router_address: {} authenticator_address: {:?}",
            self.ip_packet_router_address, self.authenticator_address
        )
    }
}
