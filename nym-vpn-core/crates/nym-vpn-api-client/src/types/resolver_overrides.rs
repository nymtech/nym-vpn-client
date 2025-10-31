use crate::{api_urls_to_urls, error::VpnApiClientError, url_to_socket_addr};
use futures::future::{self, BoxFuture};
use nym_http_api_client::Url;
use nym_network_defaults::ApiUrl;
use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
};

/// A boxed future that resolves to an optional domain name and its resolved socket addresses.
type ResolutionTask<'a> = BoxFuture<'a, Option<(String, Vec<SocketAddr>)>>;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ResolverOverrides {
    overrides: HashMap<String, HashSet<SocketAddr>>,
}

impl ResolverOverrides {
    /// Create a new set of resolver overrides from the provided URLs.
    /// Resolves all domains in parallel for faster startup and reconnection.
    pub async fn from_urls(urls: &[Url]) -> Result<Self, VpnApiClientError> {
        // Collect all resolution tasks to run in parallel
        let mut resolution_tasks: Vec<ResolutionTask<'_>> = Vec::new();

        for url in urls {
            let Some(domain) = url.inner_url().domain() else {
                tracing::warn!(
                    "Ignoring API URL '{}' for resolver overrides as it does not have a valid domain",
                    url.to_string()
                );
                continue;
            };

            // Task for main domain
            let main_url = url.inner_url().clone();
            let main_domain = domain.to_string();
            resolution_tasks.push(Box::pin(async move {
                match url_to_socket_addr(&main_url, Some((1, 1))).await {
                    Ok(addresses) => Some((main_domain.clone(), addresses)),
                    Err(e) => {
                        tracing::warn!("Failed to resolve domain {}: {}", main_domain, e);
                        None
                    }
                }
            }));

            // Tasks for front URLs
            if let Some(fronts) = url.fronts() {
                for front_url in fronts {
                    let Some(front_domain) = front_url.domain() else {
                        tracing::warn!(
                            "Ignoring front host URL '{}' for resolver overrides as it does not have a valid domain",
                            front_url
                        );
                        continue;
                    };
                    let front_url_clone = front_url.clone();
                    let front_domain_str = front_domain.to_string();
                    resolution_tasks.push(Box::pin(async move {
                        match url_to_socket_addr(&front_url_clone, Some((1, 1))).await {
                            Ok(addresses) => Some((front_domain_str.clone(), addresses)),
                            Err(e) => {
                                tracing::warn!(
                                    "Failed to resolve front domain {}: {}",
                                    front_domain_str,
                                    e
                                );
                                None
                            }
                        }
                    }));
                }
            }
        }

        // Execute all resolution tasks in parallel
        let total_tasks = resolution_tasks.len();
        let results = future::join_all(resolution_tasks).await;

        // Collect successful resolutions
        let mut overrides = HashMap::new();
        let mut successful_resolutions = 0;

        for (domain, addresses) in results.into_iter().flatten() {
            overrides.insert(domain, HashSet::from_iter(addresses.into_iter()));
            successful_resolutions += 1;
        }

        if overrides.is_empty() {
            return Err(VpnApiClientError::HostnameResolutionTimeout {
                hostname: "all domains".to_string(),
            });
        }

        tracing::debug!(
            "Successfully resolved {}/{} domains in parallel",
            successful_resolutions,
            total_tasks
        );

        Ok(Self { overrides })
    }

    /// Create resolver overrides from the provided ApiUrls
    pub async fn from_api_urls(api_urls: &[ApiUrl]) -> Result<Self, VpnApiClientError> {
        let urls = api_urls_to_urls(api_urls)?;
        Self::from_urls(&urls).await
    }

    /// Extend the current overrides with another set of overrides.
    pub fn extend(&mut self, other: &ResolverOverrides) {
        for (domain, addresses) in other.overrides.iter() {
            self.overrides
                .entry(domain.clone())
                .or_default()
                .extend(addresses.iter().cloned());
        }
    }

    /// Are there any overrides present?
    pub fn is_empty(&self) -> bool {
        self.overrides.is_empty()
    }

    /// Get all the domains
    pub fn domains(&self) -> Vec<String> {
        self.overrides.keys().cloned().collect()
    }

    // Get all the addresses for a domain
    pub fn addresses(&self, domain: &str) -> Option<Vec<SocketAddr>> {
        self.overrides
            .get(domain)
            .map(|addrs| addrs.iter().cloned().collect())
    }

    /// Get all the addresses
    pub fn all_addresses(&self) -> Vec<SocketAddr> {
        self.overrides
            .values()
            .flat_map(|addrs| addrs.iter().cloned())
            .collect()
    }
}
