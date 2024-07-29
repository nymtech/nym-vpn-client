// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{fmt::Display, str::FromStr};

use crate::{error::Result, DescribedGatewayWithLocation, Error};
use nym_sdk::mixnet::Recipient;

// optional, until we remove the wireguard feature flag
#[derive(Debug, Copy, Clone)]
pub struct AuthAddress(pub Option<Recipient>);

impl AuthAddress {
    pub(crate) fn try_from_base58_string(address: &str) -> Result<Self> {
        let recipient = Recipient::try_from_base58_string(address).unwrap();
        Ok(AuthAddress(Some(recipient)))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct AuthAddresses {
    entry_addr: AuthAddress,
    exit_addr: AuthAddress,
}

impl AuthAddresses {
    pub fn new(entry_addr: AuthAddress, exit_addr: AuthAddress) -> Self {
        AuthAddresses {
            entry_addr,
            exit_addr,
        }
    }

    pub fn entry(&self) -> AuthAddress {
        self.entry_addr
    }

    pub fn exit(&self) -> AuthAddress {
        self.exit_addr
    }
}

impl Display for AuthAddresses {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "entry: {:?} exit: {:?}",
            self.entry_addr.0, self.exit_addr.0
        )
    }
}

pub fn extract_authenticator(
    gateways: &[DescribedGatewayWithLocation],
    identity: String,
) -> Result<AuthAddress> {
    let auth_addr = gateways
        .iter()
        .find(|gw| *gw.gateway.bond.identity() == identity)
        .ok_or(Error::RequestedGatewayIdNotFound(identity.clone()))?
        .gateway
        .self_described
        .clone()
        .ok_or(Error::NoGatewayDescriptionAvailable(identity))?
        .authenticator
        .map(|auth| Recipient::from_str(&auth.address))
        .transpose()
        .map_err(|_| Error::RecipientFormattingError)?;
    Ok(AuthAddress(auth_addr))
}
