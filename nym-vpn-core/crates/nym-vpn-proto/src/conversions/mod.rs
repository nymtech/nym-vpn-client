// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod account;
pub mod account_controller_state;
pub mod error;
pub mod network_config;
pub mod prost;
pub mod tunnel_event;
pub mod tunnel_state;
pub mod vpn_api_client;
pub mod vpnd;

pub use error::ConversionError;

use crate::proto;

impl From<nym_gateway_directory::GatewayType> for proto::GatewayType {
    fn from(value: nym_gateway_directory::GatewayType) -> Self {
        match value {
            nym_gateway_directory::GatewayType::MixnetEntry => proto::GatewayType::MixnetEntry,
            nym_gateway_directory::GatewayType::MixnetExit => proto::GatewayType::MixnetExit,
            nym_gateway_directory::GatewayType::MixnetNearestEntry => {
                proto::GatewayType::MixnetEntry
            }
            nym_gateway_directory::GatewayType::Wg => proto::GatewayType::Wg,
            nym_gateway_directory::GatewayType::WgNearestEntry => proto::GatewayType::Wg,
        }
    }
}

impl From<proto::GatewayType> for nym_gateway_directory::GatewayType {
    fn from(value: proto::GatewayType) -> Self {
        match value {
            proto::GatewayType::MixnetEntry => nym_gateway_directory::GatewayType::MixnetEntry,
            proto::GatewayType::MixnetExit => nym_gateway_directory::GatewayType::MixnetExit,
            proto::GatewayType::Wg => nym_gateway_directory::GatewayType::Wg,
        }
    }
}

impl From<proto::UserAgent> for nym_sdk::UserAgent {
    fn from(user_agent: proto::UserAgent) -> Self {
        Self {
            application: user_agent.application,
            version: user_agent.version,
            platform: user_agent.platform,
            git_commit: user_agent.git_commit,
        }
    }
}

impl From<nym_http_api_client::UserAgent> for proto::UserAgent {
    fn from(user_agent: nym_http_api_client::UserAgent) -> Self {
        Self {
            application: user_agent.application,
            version: user_agent.version,
            platform: user_agent.platform,
            git_commit: user_agent.git_commit,
        }
    }
}

impl From<String> for proto::Url {
    fn from(url: String) -> Self {
        proto::Url { url }
    }
}

impl From<url::Url> for proto::Url {
    fn from(url: url::Url) -> Self {
        proto::Url {
            url: url.to_string(),
        }
    }
}

impl From<&nym_sdk::mixnet::NodeIdentity> for proto::EntryNode {
    fn from(identity: &nym_sdk::mixnet::NodeIdentity) -> Self {
        Self {
            entry_node_enum: Some(proto::entry_node::EntryNodeEnum::Gateway(proto::Gateway {
                id: identity.to_base58_string(),
            })),
        }
    }
}

impl From<&nym_sdk::mixnet::NodeIdentity> for proto::ExitNode {
    fn from(identity: &nym_sdk::mixnet::NodeIdentity) -> Self {
        Self {
            exit_node_enum: Some(proto::exit_node::ExitNodeEnum::Gateway(proto::Gateway {
                id: identity.to_base58_string(),
            })),
        }
    }
}

impl From<&nym_sdk::mixnet::Recipient> for proto::ExitNode {
    fn from(address: &nym_sdk::mixnet::Recipient) -> Self {
        Self {
            exit_node_enum: Some(proto::exit_node::ExitNodeEnum::Address(proto::Address {
                nym_address: address.to_string(),
                gateway_id: address.gateway().to_base58_string(),
            })),
        }
    }
}

impl From<std::net::IpAddr> for proto::Dns {
    fn from(ip: std::net::IpAddr) -> Self {
        Self { ip: ip.to_string() }
    }
}

impl From<u8> for proto::Threshold {
    fn from(performance: u8) -> Self {
        Self {
            min_performance: performance.into(),
        }
    }
}
