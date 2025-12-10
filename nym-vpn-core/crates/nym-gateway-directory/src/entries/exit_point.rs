// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt::{Display, Formatter};

use nym_sdk::mixnet::{NodeIdentity, Recipient};
use serde::{Deserialize, Serialize};

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
