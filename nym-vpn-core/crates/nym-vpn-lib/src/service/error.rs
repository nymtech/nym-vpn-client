// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    MixnetError, favorites::RecentsError, tunnel_state_machine::Error as TunnelStateMachineError,
};
use nym_vpn_api_client::error::VpnApiClientError;
use nym_vpn_lib_types::GatewayType;

use super::config::ConfigSetupError;

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
    #[error("failed to create account controller")]
    CreateAccountController(#[source] nym_vpn_account_controller::Error),

    #[error("failed to setup bandwidth controller storage paths")]
    BandwidthControllerStorage(#[source] Box<nym_sdk::Error>),

    #[error("failed to create gateway client")]
    CreateGatewayClient(#[source] crate::gateway_directory::Error),

    #[error("config setup error")]
    ConfigSetup(#[source] ConfigSetupError),

    #[error("failed to set up paths")]
    PathsSetup(#[source] crate::paths::PathsSetupError),

    #[error("failed to create file updater")]
    CreateFileUpdater(#[source] nym_file_updater::FileUpdaterError),

    #[error("state machine error")]
    StateMachine(#[source] TunnelStateMachineError),

    #[error("mixnet setup error")]
    MixnetSetup(#[from] MixnetError),

    #[error("failed to create api client")]
    CreateApiClient(#[source] VpnApiClientError),

    #[error("invalid environment: {0}")]
    InvalidEnvironment(&'static str),

    #[error("failed to convert API URLs")]
    ConvertApiUrls(#[source] VpnApiClientError),

    #[error("Network environment is not initialized")]
    NetworkEnvNotInitialized,
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum GlobalConfigError {
    #[error("failed to read config")]
    ReadConfig(String),
    #[error("failed to write config")]
    WriteConfig(String),
}

#[derive(Debug, thiserror::Error)]
pub enum GeoExclusionConfigError {
    #[error("invalid port")]
    InvalidPort,

    #[error("port '{0}' is reserved and cannot be used")]
    ReservedPort(u16),

    #[error("invalid country code '{0}': must be a 2-letter uppercase ISO code")]
    InvalidCountryCode(String),

    #[error("unsupported country code '{0}': only 'CN' is currently supported")]
    UnsupportedCountry(String),

    #[error("'CN' must be included in the excluded countries list")]
    CnRequired,
}

#[derive(Debug, thiserror::Error)]
pub enum ListGatewaysError {
    #[error("failed to get gateways ({gw_type:?})")]
    GetGateways {
        gw_type: GatewayType,
        source: crate::gateway_directory::Error,
    },

    #[error("failed to get filtered gateways ({gw_type:?})")]
    GetFilteredGateways {
        gw_type: GatewayType,
        source: crate::gateway_directory::Error,
    },

    #[error("failed to get recent gateways ({0})")]
    GetRecentGateways(RecentsError),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
