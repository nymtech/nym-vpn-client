// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only
use std::{
    collections::{HashMap, HashSet},
    net::{IpAddr, SocketAddr},
};

use tokio::task::JoinSet;

use nym_common::ErrorExt;
use nym_http_api_client::Url;
use nym_network_defaults::ApiUrl;

use crate::{api_urls_to_urls, error::VpnApiClientError, url_to_socket_addr};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ResolverOverrides {
    overrides: HashMap<String, HashSet<SocketAddr>>,
}

impl ResolverOverrides {
    /// Create a new set of resolver overrides from the provided URLs.
    /// Resolves all domains in parallel for faster startup and reconnection.
    pub async fn from_urls(urls: &[Url]) -> Result<Self, VpnApiClientError> {
        let mut join_set = JoinSet::new();

        tracing::debug!("getting overrides for {urls:?}");

        let urls_to_resolve = urls
            .iter()
            .flat_map(|url| {
                [url.inner_url().clone()]
                    .into_iter()
                    .chain(url.fronts().unwrap_or_default().iter().cloned())
            })
            .collect::<HashSet<_>>();

        for url in urls_to_resolve {
            let Some(domain) = url.domain().map(|s| s.to_owned()) else {
                tracing::warn!(
                    "Ignoring API URL '{}' for resolver overrides as it does not have a valid domain",
                    url.to_string()
                );
                continue;
            };

            join_set.spawn(async move {
                let result = url_to_socket_addr(&url, Some((1, 1)))
                    .await
                    .inspect_err(|err| {
                        tracing::warn!(
                            "{}",
                            err.display_chain_with_msg(format!(
                                "Failed to resolve domain {domain}"
                            ))
                        );
                    });
                (domain, result)
            });
        }

        // Execute all resolution tasks in parallel
        let results = join_set.join_all().await;

        let mut overrides = HashMap::new();
        let mut failures = Vec::new();
        for (domain, result) in results {
            match result {
                Ok(addresses) => {
                    overrides.insert(domain, HashSet::from_iter(addresses));
                }
                Err(VpnApiClientError::DnsResolution {
                    failures: domain_failures,
                }) => failures.extend(domain_failures),
                Err(err) => return Err(err),
            }
        }

        if failures.is_empty() {
            tracing::debug!(
                "Successfully resolved domains in parallel: {:?}",
                overrides.keys().collect::<Vec<_>>()
            );

            Ok(Self { overrides })
        } else {
            Err(VpnApiClientError::DnsResolution { failures })
        }
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

    pub fn addr_map(&self) -> HashMap<String, Vec<IpAddr>> {
        let mut out = HashMap::new();
        self.overrides.iter().for_each(|(d, v)| {
            let addrs = v.iter().map(|sa| sa.ip()).collect();
            out.insert(d.clone(), addrs);
        });
        out
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
            Err(VpnApiClientError::DnsResolution { failures }) => {
                let hostnames = failures
                    .into_iter()
                    .map(|failure| failure.hostname)
                    .collect::<HashSet<_>>();
                assert_eq!(hostnames, expected)
            }
            Err(e) => panic!("unexpected err: {e}"),
        }
        Ok(())
    }
}
