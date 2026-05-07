// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{path::Path, sync::Arc};

use adblock::{
    Engine, FilterSet,
    lists::{ParseOptions, RuleTypes},
    request::Request,
};
use tokio::sync::RwLock;

use crate::{
    adblocker::{
        AdBlockerError, Result,
        engines::AdBlockEngine,
        file_manager::{SOURCES, Source, SourceMetaData},
    },
    dns_filter::{DnsFilterDecision, DnsFilterStrategy, DnsFilterT},
};

/// Ad-block engine based on Brave ad-engine.
#[derive(Default, Clone)]
pub struct BraveAdblockEngine {
    engine: Arc<RwLock<Option<Engine>>>,
}

#[async_trait::async_trait]
impl AdBlockEngine for BraveAdblockEngine {
    async fn load_filters(&self, dir: &Path) -> Result<()> {
        let filter_set = load_filter_set(dir).await?;
        tracing::info!("AdBlocker using new filter-set");
        let new_engine = Engine::from_filter_set(filter_set, true);
        self.engine.write().await.replace(new_engine);
        Ok(())
    }

    async fn unload_filters(&self) {
        tracing::info!("AdBlocker using no filter-set (not blocking Ads)");
        self.engine.write().await.take();
    }
}

impl BraveAdblockEngine {
    async fn should_block_url(&self, url: url::Url) -> Result<bool> {
        let engine = self.engine.read().await;

        let Some(engine) = engine.as_ref() else {
            // If we're not initialized, then we already know the answer
            return Ok(false);
        };

        // Use empty string for source URL since it is unavailable.
        let source_url = "";

        // Use `other` as request type since it is unavailable.
        let request_type = "other";

        let request = Request::new(url.as_str(), source_url, request_type).map_err(|error| {
            AdBlockerError::CreateRequest {
                url: url.to_string(),
                error,
            }
        })?;

        let matched = engine.check_network_request(&request).matched;

        Ok(matched)
    }
}

#[async_trait::async_trait]
impl DnsFilterT for BraveAdblockEngine {
    async fn should_block(&self, domain: &str) -> DnsFilterDecision {
        const PASS: DnsFilterDecision = DnsFilterDecision::Pass;
        const BLOCK: DnsFilterDecision = DnsFilterDecision::Block(DnsFilterStrategy::Localhost);

        // Some DNS qname strings can crash `Request::new()`, so clean them up before parsing.
        let domain = super::qname_to_domain_name(domain);

        // Treat empty / root as non-blockable.
        if domain.is_empty() {
            return PASS;
        }

        let domain_url = format!("https://{domain}");
        let Ok(url) = url::Url::parse(&domain_url) else {
            tracing::error!("Ad-blocker failed to parse url {domain_url}");
            return BLOCK;
        };

        match self.should_block_url(url).await {
            Ok(block) => {
                if block {
                    BLOCK
                } else {
                    PASS
                }
            }
            Err(error) => {
                tracing::error!("Ad-blocker failed to check domain {domain}: {error}");
                BLOCK
            }
        }
    }
}

/// Create an `adblock::FilterSet` from the data files on disk.
async fn load_filter_set(cache_dir: &Path) -> Result<FilterSet> {
    let mut filter_set = FilterSet::new(cfg!(debug_assertions));

    for source in SOURCES.iter() {
        let meta_path = cache_dir.join(source.meta_file_name);
        let meta_data = SourceMetaData::from_file(&meta_path).await?;
        let data_path = cache_dir.join(source.file_name);
        let domain_list = Source::load_data_file(&data_path, &meta_data).await?;
        filter_set.add_filter_list(
            &domain_list,
            ParseOptions {
                format: source.filterset_format,
                rule_types: RuleTypes::NetworkOnly,
                ..Default::default()
            },
        );
    }

    Ok(filter_set)
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
        let _filter_set = load_filter_set(temp_dir.path()).await.unwrap();
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_blocks_domain() {
        let temp_dir = init_tests().await.unwrap();
        let engine = BraveAdblockEngine::default();

        engine.load_filters(temp_dir.path()).await.unwrap();
        assert!(matches!(
            engine.should_block(SHOULD_BE_BLOCKED_DOMAIN).await,
            DnsFilterDecision::Block(_)
        ));
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_should_not_block_without_rules() {
        let adblock = BraveAdblockEngine::default();
        let decision = adblock.should_block(SHOULD_BE_BLOCKED_DOMAIN).await;
        assert!(matches!(decision, DnsFilterDecision::Pass));
    }
}
