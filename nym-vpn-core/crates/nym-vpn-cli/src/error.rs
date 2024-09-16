// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(thiserror::Error, Debug)]
pub(crate) enum Error {
    #[error(transparent)]
    VpnLib(#[from] nym_vpn_lib::Error),

    #[error(transparent)]
    ImportCredential(#[from] nym_vpn_lib::credentials::ImportCredentialError),

    #[error("failed to read credential path: {0}")]
    FailedToReadCredentialPath(#[source] std::io::Error),

    #[error("failed to create credential data path: {0}")]
    FailedToCreateCredentialDataPath(#[source] std::io::Error),

    #[error("identity not formatted correctly")]
    NodeIdentityFormatting,

    #[error("recipient is not formatted correctly")]
    RecipientFormatting,

    #[error("config path not set")]
    ConfigPathNotSet,

    #[error("failed to parse encoded credential data")]
    FailedToParseEncodedCredentialData(#[source] bs58::decode::Error),

    #[cfg(unix)]
    #[error("sudo/root privileges required, try rerunning with sudo: `sudo -E {binary_name} run`")]
    RootPrivilegesRequired { binary_name: String },

    #[cfg(windows)]
    #[error("administrator privileges required, try rerunning with administrator privileges: `runas /user:Administrator {binary_name} run`")]
    AdminPrivilegesRequired { binary_name: String },

    #[error("failed to setup gateway minimum performance threshold: {0}")]
    FailedToSetupGatewayPerformanceThresholds(#[source] nym_vpn_api_client::VpnApiClientError),
}

// Result type based on our error type
pub(crate) type Result<T> = std::result::Result<T, Error>;
