// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use sha2::Digest;

mod api_client;
pub mod config;
pub mod controller;
mod error;
pub mod events;
pub mod handler;
pub mod report;
mod storage;

const CLIENT_ID_PREFIX: &str = "vpnclient_stats_id";

pub fn generate_vpn_client_stats_id<M: AsRef<[u8]>>(seed: M) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(CLIENT_ID_PREFIX);
    hasher.update(&seed);
    let output = hasher.finalize();
    format!("{:x}", output)
}
