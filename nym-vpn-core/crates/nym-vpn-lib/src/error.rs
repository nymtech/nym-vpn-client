// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(thiserror::Error, Debug)]
pub enum GatewayDirectoryError {
    #[error("failed to setup gateway directory client")]
    SetupGatewayDirectoryClient {
        config: Box<nym_gateway_directory::Config>,
        source: nym_gateway_directory::Error,
    },

    #[error("failed to lookup gateways")]
    LookupGateways(#[source] nym_gateway_directory::Error),

    #[error("failed to lookup gateway identity")]
    LookupGatewayIdentity(#[source] nym_gateway_directory::Error),

    #[error("failed to select entry gateway")]
    SelectEntryGateway(#[source] nym_gateway_directory::Error),

    #[error("no performant entry gateway available")]
    PerformantEntryGatewayUnavailable(#[source] nym_gateway_directory::Error),

    #[error("no performant exit gateway available")]
    PerformantExitGatewayUnavailable(#[source] nym_gateway_directory::Error),

    #[error("failed to select exit gateway")]
    SelectExitGateway(#[source] nym_gateway_directory::Error),

    #[error("failed to lookup router address")]
    LookupRouterAddress(#[source] nym_gateway_directory::Error),

    #[error("unable to use same entry and exit gateway: {identity}")]
    SameEntryAndExitGateway { identity: String },

    #[error("failed to load wireguard keypair from database for {identity} gateway")]
    LoadKeypair {
        identity: String,
        #[source]
        source: nym_vpn_store::keys::wireguard::KeysDbError,
    },
}
