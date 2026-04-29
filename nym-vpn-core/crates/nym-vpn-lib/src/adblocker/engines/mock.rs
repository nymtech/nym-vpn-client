// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;

use crate::{
    adblocker::{Result, engines::AdBlockEngine},
    resolver::{DnsFilterDecision, DnsFilterT},
};

pub struct MockEngine;

#[async_trait::async_trait]
impl AdBlockEngine for MockEngine {
    async fn load_filters(&self, _dir: &Path) -> Result<()> {
        Ok(())
    }

    async fn unload_filters(&self) {
        // todo
    }
}

#[async_trait::async_trait]
impl DnsFilterT for MockEngine {
    async fn should_block(&self, _domain: &str) -> DnsFilterDecision {
        DnsFilterDecision::Pass
    }
}
