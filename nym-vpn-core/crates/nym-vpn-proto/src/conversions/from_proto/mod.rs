// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod account;
pub mod network_config;
pub mod tunnel_event;
pub mod tunnel_state;
pub mod vpnd;

use crate::GatewayType as ProtoGatewayType;

impl From<nym_gateway_directory::GatewayType> for ProtoGatewayType {
    fn from(value: nym_gateway_directory::GatewayType) -> Self {
        match value {
            nym_gateway_directory::GatewayType::MixnetEntry => ProtoGatewayType::MixnetEntry,
            nym_gateway_directory::GatewayType::MixnetExit => ProtoGatewayType::MixnetExit,
            nym_gateway_directory::GatewayType::Wg => ProtoGatewayType::Wg,
        }
    }
}

impl From<crate::UserAgent> for nym_sdk::UserAgent {
    fn from(user_agent: crate::UserAgent) -> Self {
        Self {
            application: user_agent.application,
            version: user_agent.version,
            platform: user_agent.platform,
            git_commit: user_agent.git_commit,
        }
    }
}
