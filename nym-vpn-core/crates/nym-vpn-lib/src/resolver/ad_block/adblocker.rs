// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::{
    files::{init_files, load_filter_set, update_files}, AdBlockError,
    Result,
};
use adblock::{request::Request, Engine};
use std::path::PathBuf;

pub struct AdBlocker {
    engine: Engine,
}

impl AdBlocker {
    pub async fn new(data_dir: PathBuf, force_init: bool) -> Result<Self> {
        tracing::debug!("Initializing ad-blocker");

        init_files(&data_dir, force_init).await?;
        let filter_set = load_filter_set(&data_dir).await?;
        let engine = Engine::from_filter_set(filter_set, true);
        Ok(Self { engine })
    }

    pub async fn with_updated_files(data_dir: PathBuf, user_agent: &str) -> Result<Option<Self>> {
        tracing::debug!("Checking for Ad-blocker file updates");

        if update_files(&data_dir, user_agent).await? {
            let filter_set = load_filter_set(&data_dir).await?;
            let engine = Engine::from_filter_set(filter_set, true);
            Ok(Some(Self { engine }))
        } else {
            Ok(None)
        }
    }

    pub async fn should_block_domain(&self, domain: &str) -> Result<bool> {
        // Some DNS qname strings can crash `Request::new()`, so clean them up before parsing.
        let mut domain = domain.trim();

        // Treat empty / root as non\-blockable.
        if domain.is_empty() || domain == "." || domain == "./" {
            return Ok(false);
        }

        // Remove any trailing "/"
        domain = domain.trim_end_matches('/');

        // Remove any trailing "." (including cases like "example.com./")
        domain = domain.trim_end_matches('.');

        if domain.is_empty() {
            return Ok(false);
        }

        let domain_url = format!("https://{domain}");
        let url = url::Url::parse(&domain_url).map_err(|error| AdBlockError::ParseUrl {
            url: domain_url,
            error,
        })?;

        self.should_block_url(url).await
    }

    pub async fn should_block_url(&self, url: url::Url) -> Result<bool> {
        // Use empty string for source URL since it is unavailable.
        let source_url = "";

        // Use `other` as request type since it is unavailable.
        let request_type = "other";

        let request = Request::new(url.as_str(), source_url, request_type).map_err(|error| {
            AdBlockError::CreateRequest {
                url: url.to_string(),
                error,
            }
        })?;

        let matched = self.engine.check_network_request(&request).matched;

        Ok(matched)
    }
}
