// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt::Debug;

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum GetMnemonicError {
    #[error("storage: {0}")]
    Storage(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl GetMnemonicError {
    pub fn internal(err: impl ToString) -> Self {
        GetMnemonicError::Internal(err.to_string())
    }

    pub fn storage(err: impl ToString) -> Self {
        GetMnemonicError::Storage(err.to_string())
    }
}
