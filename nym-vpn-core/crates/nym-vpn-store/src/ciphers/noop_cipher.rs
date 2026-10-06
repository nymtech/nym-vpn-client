// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::{Result, StoreCipher};

pub struct NoopStoreCipher {}

impl StoreCipher for NoopStoreCipher {
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(data.to_vec())
    }

    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(data.to_vec())
    }
}
