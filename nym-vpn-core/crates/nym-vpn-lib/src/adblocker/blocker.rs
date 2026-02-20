// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::{AdBlockerError, Result};
use crate::resolver::{DnsFilterDecision, DnsFilterStrategy, DnsFilterT};
use adblock::{Engine, FilterSet, request::Request};
use std::any::Any;

#[derive(Default)]
pub struct AdBlocker {
    engine: Engine,
    initted: bool,
}

impl AdBlocker {
    const PASS: DnsFilterDecision = DnsFilterDecision::Pass;
    const BLOCK: DnsFilterDecision = DnsFilterDecision::Block(DnsFilterStrategy::Localhost);

    pub async fn use_filter_set(&mut self, filter_set: Box<FilterSet>) {
        tracing::trace!("AdBlocker using new filter-set");
        self.engine = Engine::from_filter_set(*filter_set, true);
        self.initted = true;
    }

    pub async fn clear_filter_set(&mut self) {
        tracing::trace!("AdBlocker clearing filter-set");
        self.engine = Engine::default();
        self.initted = false;
    }

    pub fn is_initted(&self) -> bool {
        self.initted
    }

    pub fn should_block_url(&self, url: url::Url) -> Result<bool> {
        // If we're not initialized, then we already know the answer
        if !self.initted {
            return Ok(false);
        }

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

        let matched = self.engine.check_network_request(&request).matched;

        Ok(matched)
    }
}

impl DnsFilterT for AdBlocker {
    fn should_block(&self, domain: &str) -> DnsFilterDecision {
        // Some DNS qname strings can crash `Request::new()`, so clean them up before parsing.
        let mut domain = domain.trim();

        // Treat empty / root as non-blockable.
        if domain.is_empty() || domain == "." || domain == "./" {
            return Self::PASS;
        }

        // Remove any trailing "/"
        domain = domain.trim_end_matches('/');

        // Remove any trailing "." (including cases like "example.com./")
        domain = domain.trim_end_matches('.');

        if domain.is_empty() {
            return Self::BLOCK;
        }

        let domain_url = format!("https://{domain}");
        let Ok(url) = url::Url::parse(&domain_url) else {
            tracing::error!("Ad-blocker failed to parse url {domain_url}");
            return Self::BLOCK;
        };

        match self.should_block_url(url) {
            Ok(block) => {
                if block {
                    Self::BLOCK
                } else {
                    Self::PASS
                }
            }
            Err(error) => {
                tracing::error!("Ad-blocker failed to check domain {domain}: {error}");
                Self::BLOCK
            }
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
