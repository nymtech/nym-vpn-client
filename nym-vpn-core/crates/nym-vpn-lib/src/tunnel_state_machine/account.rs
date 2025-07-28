// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::{
    RegisterDeviceError, RequestZkNymError, SyncAccountError, SyncDeviceError,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("timeout")]
    Cancelled,

    #[error(transparent)]
    Command(#[from] nym_vpn_lib_types::AccountCommandError),

    #[error("device time not synced")]
    DeviceTimeOutOfSync,

    #[error(transparent)]
    SyncAccount(#[from] SyncAccountError),

    #[error(transparent)]
    SyncDevice(#[from] SyncDeviceError),

    #[error(transparent)]
    RegisterDevice(#[from] RegisterDeviceError),

    #[error(transparent)]
    RequestZkNym(#[from] RequestZkNymError),
}
