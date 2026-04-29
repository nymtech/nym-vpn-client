// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{collections::HashSet, path::Path, sync::Arc};

use adblock::lists::{ParseOptions, ParsedFilter, RuleTypes, parse_filter};
use futures::StreamExt;
use tokio::sync::RwLock;

use crate::{
    adblocker::{
        Result,
        engines::AdBlockEngine,
        file_manager::{SOURCES, Source},
    },
    dns_filter::{DnsFilterDecision, DnsFilterT},
    resolver::DnsFilterStrategy,
};

/// Ad-block engine that uses a simple domain list for blocking.
#[derive(Default, Clone)]
pub struct SimpleAdBlockEngine {
    blocked_domains: Arc<RwLock<HashSet<String>>>,
}

#[async_trait::async_trait]
impl AdBlockEngine for SimpleAdBlockEngine {
    async fn load_filters(&self, dir: &Path) -> Result<()> {
        let blocked_domains = load_blocked_domain_list(dir).await?;
        let mut blocked_domains_guard = self.blocked_domains.write().await;
        *blocked_domains_guard = blocked_domains;
        Ok(())
    }

    async fn unload_filters(&self) {
        self.blocked_domains.write().await.clear();
    }
}

#[async_trait::async_trait]
impl DnsFilterT for SimpleAdBlockEngine {
    async fn should_block(&self, domain: &str) -> DnsFilterDecision {
        const PASS: DnsFilterDecision = DnsFilterDecision::Pass;
        const BLOCK: DnsFilterDecision = DnsFilterDecision::Block(DnsFilterStrategy::Localhost);

        // Some DNS qname strings can crash `Request::new()`, so clean them up before parsing.
        let mut domain = domain.trim();

        // Treat empty / root as non-blockable.
        if domain.is_empty() || domain == "." || domain == "./" {
            return PASS;
        }

        // Remove any trailing "/"
        domain = domain.trim_end_matches('/');

        // Remove any trailing "." (including cases like "example.com./")
        domain = domain.trim_end_matches('.');

        if domain.is_empty() {
            return BLOCK;
        }

        let blocked_domains = self.blocked_domains.read().await;
        // Convert to lowercase for case-insensitive comparison
        if blocked_domains.contains(&domain.to_lowercase()) {
            BLOCK
        } else {
            PASS
        }
    }
}

async fn load_blocked_domain_list(cache_dir: &Path) -> Result<HashSet<String>> {
    let mut blocked_domains = HashSet::new();

    for source in SOURCES.iter() {
        let data_path = cache_dir.join(source.file_name);

        let opts = ParseOptions {
            format: source.filterset_format,
            rule_types: RuleTypes::NetworkOnly,
            ..Default::default()
        };

        let mut lines = Source::stream_lines(&data_path);
        while let Some(line) = lines.next().await {
            let line = line?;

            // Ignore errors since they aren't that useful
            if let Ok(ParsedFilter::Network(filter)) = parse_filter(&line, false, opts)
                && let Some(ref domain) = filter.hostname
            {
                // Convert to lowercase for case-insensitive comparison
                blocked_domains.insert(domain.to_lowercase());
            }
        }
    }

    Ok(blocked_domains)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adblocker::file_manager::tests::init_tests;

    const SHOULD_BE_BLOCKED_DOMAIN: &str = "www.0.beer";

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn load_rules() {
        let temp_dir = init_tests().await.unwrap();
        let blocked_domains = load_blocked_domain_list(temp_dir.path()).await.unwrap();
        assert!(!blocked_domains.is_empty());
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_blocks_domain() {
        let temp_dir = init_tests().await.unwrap();
        let engine = SimpleAdBlockEngine::default();

        engine.load_filters(temp_dir.path()).await.unwrap();
        assert!(matches!(
            engine.should_block(SHOULD_BE_BLOCKED_DOMAIN).await,
            DnsFilterDecision::Block(_)
        ));
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_should_not_block_without_rules() {
        let adblock = SimpleAdBlockEngine::default();
        let decision = adblock.should_block(SHOULD_BE_BLOCKED_DOMAIN).await;
        assert!(matches!(decision, DnsFilterDecision::Pass));
    }
}
