// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("timeout")]
    Cancelled,

    #[error(transparent)]
    Command(#[from] nym_vpn_lib_types::AccountCommandError),

    #[error(transparent)]
    ControllerState(#[from] nym_vpn_lib_types::AccountControllerError),
}
