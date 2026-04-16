// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! DNS filter types used by the ad-blocker on all platforms.

use std::{any::Any, sync::Arc};
use tokio::sync::Mutex;

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
pub trait DnsFilterT: Send + Sync + 'static {
    fn should_block(&self, domain: &str) -> DnsFilterDecision;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// DNS filter type — a shared, dynamically-dispatched filter.
pub type DnsFilter = Arc<Mutex<Box<dyn DnsFilterT + Send + Sync>>>;

/// Null DNS Filter (always passes).
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub struct NullDnsFilter;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
impl DnsFilterT for NullDnsFilter {
    fn should_block(&self, _domain: &str) -> DnsFilterDecision {
        DnsFilterDecision::Pass
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
