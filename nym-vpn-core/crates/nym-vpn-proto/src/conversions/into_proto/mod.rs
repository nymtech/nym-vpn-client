// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod account;
pub mod network_config;
pub mod vpnd;

impl From<String> for crate::Url {
    fn from(url: String) -> Self {
        crate::Url { url }
    }
}

impl From<url::Url> for crate::Url {
    fn from(url: url::Url) -> Self {
        crate::Url {
            url: url.to_string(),
        }
    }
}

impl From<&nym_sdk::mixnet::NodeIdentity> for crate::EntryNode {
    fn from(identity: &nym_sdk::mixnet::NodeIdentity) -> Self {
        Self {
            entry_node_enum: Some(crate::entry_node::EntryNodeEnum::Gateway(crate::Gateway {
                id: identity.to_base58_string(),
            })),
        }
    }
}

impl From<&nym_sdk::mixnet::NodeIdentity> for crate::ExitNode {
    fn from(identity: &nym_sdk::mixnet::NodeIdentity) -> Self {
        Self {
            exit_node_enum: Some(crate::exit_node::ExitNodeEnum::Gateway(crate::Gateway {
                id: identity.to_base58_string(),
            })),
        }
    }
}

impl From<&nym_sdk::mixnet::Recipient> for crate::ExitNode {
    fn from(address: &nym_sdk::mixnet::Recipient) -> Self {
        Self {
            exit_node_enum: Some(crate::exit_node::ExitNodeEnum::Address(crate::Address {
                nym_address: address.to_string(),
            })),
        }
    }
}

impl From<std::net::IpAddr> for crate::Dns {
    fn from(ip: std::net::IpAddr) -> Self {
        Self { ip: ip.to_string() }
    }
}

impl From<u8> for crate::Threshold {
    fn from(performance: u8) -> Self {
        Self {
            min_performance: performance.into(),
        }
    }
}
