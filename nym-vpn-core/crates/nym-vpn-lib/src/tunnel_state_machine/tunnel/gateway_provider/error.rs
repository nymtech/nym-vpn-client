// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(thiserror::Error, Debug)]
pub enum GatewayProviderError {
    #[error("tunnel state machine error")]
    TunnelStateMachine(#[source] crate::tunnel_state_machine::Error),

    #[error("unable to use same entry and exit gateway: {identity}")]
    SameEntryAndExitGateway { identity: String },

    #[error("failed to lookup gateways")]
    LookupGateways(#[source] nym_gateway_directory::Error),

    #[error("failed to load wireguard keypair from database for {identity} gateway")]
    LoadKeypair {
        identity: String,
        #[source]
        source: nym_vpn_store::keys::wireguard::KeysDbError,
    },

    #[error("failed to select any entry gateway after trying all performance tiers")]
    EntryGatewayUnavailable(#[source] nym_gateway_directory::Error),

    #[error("failed to select any exit gateway after trying all performance tiers")]
    ExitGatewayUnavailable(#[source] nym_gateway_directory::Error),

    #[error("gateway information are malformed")]
    MalformedGateway(#[source] nym_gateway_directory::Error),

    #[error("gateway pair can be found if user agrees to relax the gateway independence criteria")]
    NeedsRelaxedIndependenceCriteria,

    #[error("auto is not possible when device geo location information is missing")]
    NeedsDeviceLocation,
}
