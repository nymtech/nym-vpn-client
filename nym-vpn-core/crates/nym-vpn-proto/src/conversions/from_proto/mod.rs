// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::ConversionError;

pub mod account;
pub mod network_config;
pub mod tunnel_event;
pub mod tunnel_state;
pub mod vpnd;

impl TryFrom<crate::GatewayType> for nym_gateway_directory::GatewayType {
    type Error = ConversionError;

    fn try_from(gateway_type: crate::GatewayType) -> Result<Self, Self::Error> {
        use nym_gateway_directory::GatewayType;
        let gw_type = match gateway_type {
            crate::GatewayType::Unspecified => {
                return Err(ConversionError::Generic(
                    "gateway type unspecified".to_string(),
                ))
            }
            crate::GatewayType::MixnetEntry => GatewayType::MixnetEntry,
            crate::GatewayType::MixnetExit => GatewayType::MixnetExit,
            crate::GatewayType::Wg => GatewayType::Wg,
        };
        Ok(gw_type)
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
