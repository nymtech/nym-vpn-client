// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("identity not formatted correctly: {identity}")]
    NodeIdentityFormattingError {
        identity: String,
        source: nym_sdk::mixnet::ed25519::Ed25519RecoveryError,
    },

    #[error("recipient is not formatted correctly: {address}")]
    RecipientFormattingError {
        address: String,
        source: nym_sdk::mixnet::RecipientFormattingError,
    },

    #[error("{0}")]
    ValidatorClientError(#[from] nym_validator_client::ValidatorClientError),

    #[error(transparent)]
    ExplorerApiError(#[from] nym_explorer_client::ExplorerApiError),

    #[error(transparent)]
    HarbourMasterError(#[from] nym_harbour_master_client::HarbourMasterError),

    #[error(transparent)]
    HarbourMasterApiError(#[from] nym_harbour_master_client::HarbourMasterApiError),

    #[error(transparent)]
    NymVpnApiClientError(#[from] nym_vpn_api_client::VpnApiClientError),

    #[error("failed to fetch location data from explorer-api: {error}")]
    FailedFetchLocationData {
        error: nym_explorer_client::ExplorerApiError,
    },

    #[error("failed to resolve gateway hostname: {hostname}: {source}")]
    FailedToDnsResolveGateway {
        hostname: String,
        source: hickory_resolver::error::ResolveError,
    },

    #[error("resolved hostname {0} but no IP address found")]
    ResolvedHostnameButNoIp(String),

    #[error("failed to lookup described gateways: {source}")]
    FailedToLookupDescribedGateways {
        source: nym_validator_client::ValidatorClientError,
    },

    #[error("requested gateway not found in the remote list: {0}")]
    RequestedGatewayIdNotFound(String),

    #[error("missing ip packet router address for gateway")]
    MissingIpPacketRouterAddress,

    #[error("no matching gateway found")]
    NoMatchingGateway,

    #[error("no gateway available for location {requested_location}, available countries: {available_countries:?}")]
    NoMatchingGatewayForLocation {
        requested_location: String,
        available_countries: Vec<String>,
    },

    #[error("no entry gateway available for location {requested_location}, available countries: {available_countries:?}")]
    NoMatchingEntryGatewayForLocation {
        requested_location: String,
        available_countries: Vec<String>,
    },

    #[error("no exit gateway available for location {requested_location}, available countries: {available_countries:?}")]
    NoMatchingExitGatewayForLocation {
        requested_location: String,
        available_countries: Vec<String>,
    },

    #[error("failed to select gateway based on low latency: {source}")]
    FailedToSelectGatewayBasedOnLowLatency {
        source: nym_client_core::error::ClientCoreError,
    },

    #[error("failed to select gateway randomly")]
    FailedToSelectGatewayRandomly,

    #[error("gateway {0} doesn't have a description available")]
    NoGatewayDescriptionAvailable(String),
}

// Result type based on our error type
pub type Result<T> = std::result::Result<T, Error>;
