// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Identity not formatted correctly: {identity}")]
    NodeIdentityFormattingError {
        identity: String,
        source: nym_sdk::mixnet::ed25519::Ed25519RecoveryError,
    },

    #[error("Recipient is not formatted correctly: {address}")]
    RecipientFormattingError {
        address: String,
        source: nym_sdk::mixnet::RecipientFormattingError,
    },

    #[error(transparent)]
    ValidatorClientError(#[from] nym_validator_client::ValidatorClientError),

    #[error(transparent)]
    VpnApiClientError(#[from] nym_vpn_api_client::VpnApiClientError),

    #[error("Failed to resolve gateway hostname: {hostname}")]
    FailedToDnsResolveGateway {
        hostname: String,
        source: nym_http_api_client::HickoryDnsError,
    },

    #[error("Resolved hostname {0} but no IP address found")]
    ResolvedHostnameButNoIp(String),

    #[error("Failed to lookup described gateways")]
    FailedToLookupDescribedGateways(#[source] nym_validator_client::ValidatorClientError),

    #[error("Failed to lookup skimmed gateways")]
    FailedToLookupSkimmedGateways(#[source] nym_validator_client::ValidatorClientError),

    #[error("Failed to lookup skimmed nodes")]
    FailedToLookupSkimmedNodes(#[source] nym_validator_client::ValidatorClientError),

    #[error("Requested gateway not found in the remote list: {0}")]
    RequestedGatewayIdNotFound(String),

    #[error("Missing ip packet router address for gateway")]
    MissingIpPacketRouterAddress,

    #[error("Missing hostname or ip address for gateway")]
    MissingHostnameOrIpAddress { gateway_identity: String },

    #[error("No matching gateway found: {requested_identity}")]
    NoMatchingGateway { requested_identity: String },

    #[error("No entry gateway available for location {requested_location}, available countries: {available_countries:?}")]
    NoMatchingEntryGatewayForLocation {
        requested_location: String,
        available_countries: Vec<String>,
    },

    #[error("No exit gateway available for location {requested_location}, available countries: {available_countries:?}")]
    NoMatchingExitGatewayForLocation {
        requested_location: String,
        available_countries: Vec<String>,
    },

    #[error("Failed to select gateway based on low latency")]
    FailedToSelectGatewayBasedOnLowLatency {
        source: nym_client_core::error::ClientCoreError,
    },

    #[error("No matching gateway found after selecting low latency: {requested_identity}")]
    NoMatchingGatewayAfterSelectingLowLatency { requested_identity: String },

    #[error("Failed to select gateway randomly")]
    FailedToSelectGatewayRandomly,

    #[error("Gateway {0} doesn't have a description available")]
    NoGatewayDescriptionAvailable(String),

    #[error("Failed to lookup gateway ip for gateway {0}")]
    FailedToLookupIp(String),

    #[error("The url {url} doesn't parse to a host and/or a port: {reason}")]
    UrlError { url: url::Url, reason: String },

    #[error("The provided gateway information is malformed")]
    MalformedGateway,
}

// Result type based on our error type
pub type Result<T> = std::result::Result<T, Error>;
