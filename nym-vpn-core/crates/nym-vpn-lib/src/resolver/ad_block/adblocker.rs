use super::{
    AdBlockError, Result,
    files::{init_files, load_filter_set, update_files},
};
use adblock::{Engine, request::Request};
use std::path::PathBuf;

pub struct AdBlocker {
    engine: Engine,
}

impl AdBlocker {
    pub async fn new(data_dir: PathBuf) -> Result<Option<Self>> {
        tracing::debug!("Initializing ad-blocker");

        init_files(&data_dir, false).await?;
        let filter_set = load_filter_set(&data_dir).await?;
        let engine = Engine::from_filter_set(filter_set, true);
        Ok(Some(Self { engine }))
    }

    pub async fn with_updated_files(data_dir: PathBuf) -> Result<Option<Self>> {
        tracing::debug!("Checking for Ad-blocker file updates");

        // Attempt to update the data files, and if we fail then re-initialize them with the
        // built-in files.
        // TODO: Detect which files have an issue and only re-initialize those, instead of all of them.
        let updated = match update_files(&data_dir).await {
            Ok(updated) => updated,
            Err(error) => {
                tracing::error!("Failed to update ad-blocker files: {error}. Re-initializing.");
                init_files(&data_dir, true).await?;
                true
            }
        };

        if updated {
            let filter_set = load_filter_set(&data_dir).await?;
            let engine = Engine::from_filter_set(filter_set, true);
            Ok(Some(Self { engine }))
        } else {
            Ok(None)
        }
    }

    pub async fn should_block_domain(&self, domain: &str) -> Result<bool> {
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
