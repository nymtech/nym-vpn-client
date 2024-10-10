// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to get account summary")]
    GetAccountSummary,

    #[error("missing API URL")]
    MissingApiUrl,

    #[error("invalid API URL")]
    InvalidApiUrl,
}
