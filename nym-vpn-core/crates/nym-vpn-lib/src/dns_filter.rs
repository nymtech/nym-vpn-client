// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! DNS filter types used by the ad-blocker on all platforms.

use std::sync::Arc;

/// How to handle a DNS-filtered blocking decision.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DnsFilterStrategy {
    EmptyRecord, // Return an empty record.
    Localhost,   // Return localhost
}

/// DNS filter decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DnsFilterDecision {
    Pass,
    Block(DnsFilterStrategy),
}

/// DNS filter trait.
#[async_trait::async_trait]
pub trait DnsFilterT {
    async fn should_block(&self, domain: &str) -> DnsFilterDecision;
}

/// DNS filter type — a shared, dynamically-dispatched filter.
pub type DnsFilter = Arc<dyn DnsFilterT + Send + Sync + 'static>;

/// Null DNS Filter (always passes).
#[cfg(not(target_os = "android"))]
pub struct NullDnsFilter;

#[cfg(not(target_os = "android"))]
#[async_trait::async_trait]
impl DnsFilterT for NullDnsFilter {
    async fn should_block(&self, _domain: &str) -> DnsFilterDecision {
        DnsFilterDecision::Pass
    }
}
