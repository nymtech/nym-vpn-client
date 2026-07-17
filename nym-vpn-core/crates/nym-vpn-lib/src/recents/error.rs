// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(thiserror::Error, Debug)]
pub enum RecentsError {
    #[error("failed to lookup gateway cache")]
    GetGateways {
        source: crate::gateway_directory::Error,
    },
}
