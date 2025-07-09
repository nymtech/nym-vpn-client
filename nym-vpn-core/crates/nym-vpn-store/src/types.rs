// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RawWireguardKeys {
    pub gateway_id_bs58: String,
    pub entry_private_key_bs58: String,
    pub exit_private_key_bs58: String,
}
