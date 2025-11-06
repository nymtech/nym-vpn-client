// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only
use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
};

use nym_http_api_client::Url;
use nym_network_defaults::ApiUrl;
use tokio::task::JoinSet;

use crate::{api_urls_to_urls, error::VpnApiClientError, url_to_socket_addr};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ResolverOverrides {
    overrides: HashMap<String, HashSet<SocketAddr>>,
}

impl ResolverOverrides {
    /// Create a new set of resolver overrides from the provided URLs.
    /// Resolves all domains in parallel for faster startup and reconnection.
    pub async fn from_urls(urls: &[Url]) -> Result<Self, VpnApiClientError> {
        let mut all_domains: HashSet<String> = HashSet::new();
        let mut join_set = JoinSet::new();

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
            all_domains.insert(main_domain.clone());
            join_set.spawn(async move {
                match url_to_socket_addr(&main_url, Some((1, 1))).await {
                    Ok(addresses) => Some((main_domain.clone(), addresses)),
                    Err(e) => {
                        tracing::warn!("Failed to resolve domain {}: {}", main_domain, e);
                        None
                    }
                }
            });

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
                    all_domains.insert(front_domain_str.clone());
                    join_set.spawn(async move {
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
                    });
                }
            }
        }

        // Execute all resolution tasks in parallel
        let total_tasks = join_set.len();
        let results = join_set.join_all().await;

        // Collect successful resolutions
        let mut overrides = HashMap::new();
        let mut successful_resolutions = 0;
        let mut received_domains: HashSet<String> = HashSet::new();

        for (domain, addresses) in results.into_iter().flatten() {
            overrides.insert(domain.clone(), HashSet::from_iter(addresses.into_iter()));
            received_domains.insert(domain);
            successful_resolutions += 1;
        }

        if successful_resolutions < total_tasks {
            // At least one resolution failed.
            let missed_domains: HashSet<String> =
                all_domains.difference(&received_domains).cloned().collect();
            tracing::warn!("failed to resolve one or more URLs: {:?}", missed_domains);
            return Err(VpnApiClientError::HostnamesResolutionError {
                hostnames: missed_domains,
            });
        }

        tracing::debug!(
            "Successfully resolved domains in parallel: {:?}",
            all_domains
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

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn get_overrides_for_empty_url_set() -> Result<(), VpnApiClientError> {
        let urls: Vec<Url> = vec![];

        let overrides = ResolverOverrides::from_urls(&urls).await?;
        assert!(overrides.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn get_overrides_for_url_set() -> Result<(), VpnApiClientError> {
        let urls: Vec<Url> = vec![
            Url::new("https://nymvpn.com", None).unwrap(),
            Url::new(
                "https://validator.nymtech.net",
                Some(vec!["https://example.com"]),
            )
            .unwrap(),
        ];

        let overrides = ResolverOverrides::from_urls(&urls).await?;
        assert!(!overrides.is_empty());
        assert_eq!(overrides.domains().len(), 3);

        Ok(())
    }

    #[tokio::test]
    async fn overrides_return_error() -> Result<(), VpnApiClientError> {
        let urls: Vec<Url> = vec![
            Url::new("https://nymvpn.com", None).unwrap(),
            Url::new(
                "https://validator.nymtech.net",
                Some(vec!["https://non-existent.nymtech.net"]),
            )
            .unwrap(),
        ];

        let result = ResolverOverrides::from_urls(&urls).await;
        assert!(result.is_err());

        let mut expected = HashSet::new();
        expected.insert("non-existent.nymtech.net".to_string());
        match result {
            Ok(_) => panic!("unreachable"),
            Err(VpnApiClientError::HostnamesResolutionError { hostnames }) => {
                assert_eq!(hostnames, expected)
            }
            Err(e) => panic!("unexpected err: {e}"),
        }
        Ok(())
    }
}
