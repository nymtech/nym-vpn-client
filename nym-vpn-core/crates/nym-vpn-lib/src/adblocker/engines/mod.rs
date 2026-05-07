// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod brave;
#[cfg(test)]
mod mock;
mod simple;

use std::path::Path;

pub use brave::BraveAdblockEngine;
#[cfg(test)]
pub use mock::MockEngine;
pub use simple::SimpleAdBlockEngine;

use crate::{
    adblocker::Result,
    dns_filter::{DnsFilterDecision, DnsFilterT},
};

/// Type describing adblock engine.
#[async_trait::async_trait]
pub trait AdBlockEngine: DnsFilterT + Send + Sync {
    /// Loads filters from the given directory and invokes the completion callback with the result.
    async fn load_filters(&self, dir: &Path) -> Result<()>;

    /// Unloads any loaded filters.
    async fn unload_filters(&self);
}

// Static dispatch alternative to Arc<dyn AdBlockEngine>
#[allow(dead_code)]
pub enum AdBlockEngineWrap {
    Brave(BraveAdblockEngine),
    Simple(SimpleAdBlockEngine),
    #[cfg(test)]
    Mock(MockEngine),
}

#[async_trait::async_trait]
impl AdBlockEngine for AdBlockEngineWrap {
    async fn load_filters(&self, dir: &Path) -> Result<()> {
        match self {
            Self::Brave(engine) => engine.load_filters(dir).await,
            Self::Simple(engine) => engine.load_filters(dir).await,
            #[cfg(test)]
            Self::Mock(engine) => engine.load_filters(dir).await,
        }
    }

    async fn unload_filters(&self) {
        match self {
            Self::Brave(engine) => engine.unload_filters().await,
            Self::Simple(engine) => engine.unload_filters().await,
            #[cfg(test)]
            Self::Mock(engine) => engine.unload_filters().await,
        }
    }
}

#[async_trait::async_trait]
impl DnsFilterT for AdBlockEngineWrap {
    async fn should_block(&self, domain: &str) -> DnsFilterDecision {
        match self {
            Self::Brave(engine) => engine.should_block(domain).await,
            Self::Simple(engine) => engine.should_block(domain).await,
            #[cfg(test)]
            Self::Mock(engine) => engine.should_block(domain).await,
        }
    }
}

fn qname_to_domain_name(domain: &str) -> &str {
    domain.trim().trim_end_matches(['/', '.'])
}
