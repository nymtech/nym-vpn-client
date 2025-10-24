// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_http_api_client::HttpClientError;
use nym_vpn_lib::{MixnetError, tunnel_state_machine::Error as TunnelStateMachineError};
use nym_vpn_lib_types::GatewayType;

use super::config::ConfigSetupError;

#[derive(Debug, thiserror::Error)]
pub enum AccountControllerError {
    #[error("failed to init account controller: {reason}")]
    Initialization { reason: String },
}

#[derive(Debug, thiserror::Error)]
pub enum SetNetworkError {
    #[error("failed to read config")]
    ReadConfig {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("failed to write config")]
    WriteConfig {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("failed to set network: {0}")]
    NetworkNotFound(String),
}

#[derive(Debug, thiserror::Error)]
pub enum AccountLinksError {
    #[error("account management not configured")]
    AccountManagementNotConfigured,

    #[error("failed to parse account management paths")]
    FailedToParseAccountLinks,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("account controller error")]
    AccountController(#[from] AccountControllerError),

    #[error("config setup error")]
    ConfigSetup(#[source] ConfigSetupError),

    #[error("state machine error")]
    StateMachine(#[source] TunnelStateMachineError),

    #[error("mixnet setup error")]
    MixnetSetup(#[from] MixnetError),

    #[error("HTTP Client Error: {0}")]
    HttpClient(#[from] Box<HttpClientError>),
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum GlobalConfigError {
    #[error("failed to read config")]
    ReadConfig(String),
    #[error("failed to write config")]
    WriteConfig(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ListGatewaysError {
    #[error("failed to get gateways ({gw_type:?})")]
    GetGateways {
        gw_type: GatewayType,
        source: nym_vpn_lib::gateway_directory::Error,
    },

    #[error("failed to get filtered gateways ({gw_type:?})")]
    GetFilteredGateways {
        gw_type: GatewayType,
        source: nym_vpn_lib::gateway_directory::Error,
    },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
