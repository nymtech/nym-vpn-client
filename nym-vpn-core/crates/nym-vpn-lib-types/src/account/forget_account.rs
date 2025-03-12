// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::VpnApiErrorResponse;

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum ForgetAccountError {
    #[error("registration is in progress")]
    RegistrationInProgress,

    #[error("failed to remove device from nym vpn api: {0}")]
    UpdateDeviceErrorResponse(VpnApiErrorResponse),

    #[error("unexpected response: {0}")]
    UnexpectedResponse(String),

    #[error("failed to remove account: {0}")]
    RemoveAccount(String),

    #[error("failed to remove device keys: {0}")]
    RemoveDeviceKeys(String),

    #[error("failed to reset credential storage: {0}")]
    ResetCredentialStorage(String),

    #[error("failed to remove account files: {0}")]
    RemoveAccountFiles(String),

    #[error("failed to init device keys: {0}")]
    InitDeviceKeys(String),
}
