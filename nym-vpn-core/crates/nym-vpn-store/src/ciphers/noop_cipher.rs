// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::{Result, StoreCipher};

#[derive(Default)]
pub struct NoopStoreCipher;

#[async_trait::async_trait]
impl StoreCipher for NoopStoreCipher {
    async fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(data.to_owned())
    }

    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(data.to_owned())
    }
}
