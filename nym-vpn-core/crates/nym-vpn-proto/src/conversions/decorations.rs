// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

impl crate::EntryNode {
    pub fn new_from_location(country_code: &str) -> Self {
        Self {
            entry_node_enum: Some(crate::entry_node::EntryNodeEnum::Location(
                crate::Location {
                    two_letter_iso_country_code: country_code.to_string(),
                    latitude: None,
                    longitude: None,
                },
            )),
        }
    }

    pub fn new_random() -> Self {
        Self {
            entry_node_enum: Some(crate::entry_node::EntryNodeEnum::Random(())),
        }
    }

    pub fn new_random_low_latency() -> Self {
        Self {
            entry_node_enum: Some(crate::entry_node::EntryNodeEnum::RandomLowLatency(())),
        }
    }

    pub fn new_from_gateway(identity: &nym_sdk::mixnet::NodeIdentity) -> Self {
        identity.into()
    }
}

impl crate::ExitNode {
    pub fn new_from_location(country_code: &str) -> Self {
        Self {
            exit_node_enum: Some(crate::exit_node::ExitNodeEnum::Location(crate::Location {
                two_letter_iso_country_code: country_code.to_string(),
                latitude: None,
                longitude: None,
            })),
        }
    }

    pub fn new_random() -> Self {
        Self {
            exit_node_enum: Some(crate::exit_node::ExitNodeEnum::Random(())),
        }
    }

    pub fn new_from_gateway(identity: &nym_sdk::mixnet::NodeIdentity) -> Self {
        identity.into()
    }

    pub fn new_from_address(address: &nym_sdk::mixnet::Recipient) -> Self {
        address.into()
    }
}
