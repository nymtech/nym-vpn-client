// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;

use crate::error::Error;

pub struct CredentialStoreAccessLock;

impl CredentialStoreAccessLock {
    pub fn try_acquire(_data_dir: &Path) -> Result<Self, Error> {
        tracing::warn!("Credential store cross-process lock is not implemented on this platform");
        Ok(Self)
    }

    pub fn acquire_blocking(_data_dir: &Path) -> Result<Self, Error> {
        Self::try_acquire(_data_dir)
    }
}
