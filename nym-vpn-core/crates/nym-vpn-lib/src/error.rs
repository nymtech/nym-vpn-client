// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(thiserror::Error, Debug)]
pub enum GatewayDirectoryError {
    #[error("failed to setup gateway directory client: {source}")]
    FailedtoSetupGatewayDirectoryClient {
        config: Box<nym_gateway_directory::Config>,
        source: nym_gateway_directory::Error,
    },

    #[error("failed to lookup gateways: {source}")]
    FailedToLookupGateways {
        source: nym_gateway_directory::Error,
    },

    #[error("failed to lookup gateway identity: {source}")]
    FailedToLookupGatewayIdentity {
        source: nym_gateway_directory::Error,
    },

    #[error("failed to select entry gateway: {source}")]
    FailedToSelectEntryGateway {
        source: nym_gateway_directory::Error,
    },

    #[error("failed to select exit gateway: {source}")]
    FailedToSelectExitGateway {
        source: nym_gateway_directory::Error,
    },

    #[error("failed to lookup router address: {source}")]
    FailedToLookupRouterAddress {
        source: nym_gateway_directory::Error,
    },

    #[error("unable to use same entry and exit gateway: {identity}")]
    SameEntryAndExitGateway { identity: String },
}

pub use super::tunnel_state_machine::Error;
