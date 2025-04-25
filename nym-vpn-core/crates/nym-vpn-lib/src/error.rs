// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(thiserror::Error, Debug)]
pub enum GatewayDirectoryError {
    #[error("Failed to setup gateway directory client")]
    SetupGatewayDirectoryClient {
        config: Box<nym_gateway_directory::Config>,
        source: nym_gateway_directory::Error,
    },

    #[error("Failed to lookup gateways")]
    LookupGateways(#[source] nym_gateway_directory::Error),

    #[error("Failed to lookup gateway identity")]
    LookupGatewayIdentity(#[source] nym_gateway_directory::Error),

    #[error("Failed to select entry gateway")]
    SelectEntryGateway(#[source] nym_gateway_directory::Error),

    #[error("Failed to select exit gateway")]
    SelectExitGateway(#[source] nym_gateway_directory::Error),

    #[error("Failed to lookup router address")]
    LookupRouterAddress(#[source] nym_gateway_directory::Error),

    #[error("Unable to use same entry and exit gateway: {identity}")]
    SameEntryAndExitGateway { identity: String },
}

pub use super::tunnel_state_machine::Error;
