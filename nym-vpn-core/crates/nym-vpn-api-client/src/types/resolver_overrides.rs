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
    /// Resolve every primary and fronting domain found in `urls` in parallel, without ever
    /// failing outright.
    ///
    /// Returns the successfully resolved overrides together with the set of primary domains
    /// that failed to resolve. Callers that need "at least one usable primary route" semantics
    /// (`from_urls`) turn a complete primary failure into an error; callers that only need to
    /// prune unusable routes (`resolve_and_prune`) can use the partial overrides as-is, even
    /// when every primary failed but a front still resolved.
    async fn resolve_all(urls: &[Url]) -> (Self, HashSet<String>) {
        let mut join_set = JoinSet::new();

        tracing::debug!("getting overrides for {urls:?}");

        // A domain counts as primary if it's the primary URL for at least one candidate,
        // as opposed to only ever appearing as a fronting decoy.
        let primaries: HashSet<url::Url> = urls.iter().map(|url| url.inner_url().clone()).collect();

        let candidates: HashSet<url::Url> = urls
            .iter()
            .flat_map(|url| {
                [url.inner_url().clone()]
                    .into_iter()
                    .chain(url.fronts().unwrap_or_default().iter().cloned())
            })
            .collect();

        for url in candidates {
            let is_primary = primaries.contains(&url);
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
                (domain, is_primary, result)
            });
        }

        // Execute all resolution tasks in parallel
        let results = join_set.join_all().await;

        let mut overrides = HashMap::new();
        let mut failed_primaries = HashSet::new();
        let mut failed_fronts = HashSet::new();

        for (domain, is_primary, result) in results {
            match result {
                Ok(addresses) => {
                    overrides.insert(domain, HashSet::from_iter(addresses));
                }
                Err(_) if is_primary => {
                    failed_primaries.insert(domain);
                }
                Err(_) => {
                    failed_fronts.insert(domain);
                }
            }
        }

        if !failed_fronts.is_empty() {
            tracing::warn!(
                "Ignoring unresolvable fronting domain(s), continuing without them as fallback routes: {:?}",
                failed_fronts
            );
        }

        tracing::debug!(
            "Successfully resolved domains in parallel: {:?}",
            overrides.keys().collect::<Vec<_>>()
        );

        (Self { overrides }, failed_primaries)
    }

    /// Create a new set of resolver overrides from the provided URLs.
    /// Resolves all domains in parallel for faster startup and reconnection.
    ///
    /// Fronting domains are only ever used as a fallback route, so a front that fails to
    /// resolve is dropped rather than treated as fatal. Resolution only fails outright if
    /// none of the primary URLs could be resolved either, meaning there is no usable route
    /// at all.
    pub async fn from_urls(urls: &[Url]) -> Result<Self, VpnApiClientError> {
        let (overrides, failed_primaries) = Self::resolve_all(urls).await;

        let any_primary_expected = urls.iter().any(|url| url.inner_url().domain().is_some());
        let resolved_any_primary = urls.iter().any(|url| {
            url.inner_url()
                .domain()
                .is_some_and(|domain| overrides.addresses(domain).is_some())
        });

        if any_primary_expected && !resolved_any_primary {
            tracing::warn!(
                "Failed to resolve any usable primary API URL: {:?}",
                failed_primaries
            );
            return Err(VpnApiClientError::HostnamesResolutionError {
                hostnames: failed_primaries,
            });
        }

        if !failed_primaries.is_empty() {
            tracing::warn!(
                "Failed to resolve some primary API URL(s), continuing with the remaining ones: {:?}",
                failed_primaries
            );
        }

        Ok(overrides)
    }

    /// Create resolver overrides from the provided ApiUrls
    pub async fn from_api_urls(api_urls: &[ApiUrl]) -> Result<Self, VpnApiClientError> {
        let urls = api_urls_to_urls(api_urls)?;
        Self::from_urls(&urls).await
    }

    /// Resolve the given URLs and return a pruned copy with every front that failed to
    /// resolve removed, and any candidate whose primary and every front failed to resolve
    /// dropped entirely.
    ///
    /// This lets an HTTP client built from the result never attempt a host we already know
    /// is unreachable, instead of finding out at request time (DNS lookup, then most likely
    /// a firewall-blocked connection attempt).
    ///
    /// Never errors: if resolution fails so completely that pruning would leave nothing at
    /// all, the original, unpruned list is returned so callers still have something to try.
    ///
    /// Uses [`Self::resolve_all`] directly rather than [`Self::from_urls`], so a front that
    /// did resolve is still usable for pruning even when every primary failed.
    pub async fn resolve_and_prune(urls: &[Url]) -> Vec<Url> {
        let (overrides, _failed_primaries) = Self::resolve_all(urls).await;
        let pruned = Self::prune(urls, &overrides);
        if pruned.is_empty() && !urls.is_empty() {
            tracing::warn!(
                "Pruning unresolvable API URLs would leave none usable; keeping the original list instead"
            );
            urls.to_vec()
        } else {
            pruned
        }
    }

    /// Filter out fronts (and whole candidates) that `overrides` couldn't resolve.
    pub fn prune(urls: &[Url], overrides: &ResolverOverrides) -> Vec<Url> {
        urls.iter()
            .filter_map(|url| {
                let primary_resolved = url
                    .inner_url()
                    .domain()
                    .is_some_and(|domain| overrides.addresses(domain).is_some());

                let resolved_fronts: Vec<url::Url> = url
                    .fronts()
                    .unwrap_or_default()
                    .iter()
                    .filter(|front| {
                        front
                            .domain()
                            .is_some_and(|domain| overrides.addresses(domain).is_some())
                    })
                    .cloned()
                    .collect();

                if !primary_resolved && resolved_fronts.is_empty() {
                    tracing::warn!(
                        "Dropping unusable API URL '{url}': neither its primary domain nor any of its fronts could be resolved"
                    );
                    return None;
                }

                Url::new(
                    url.inner_url().clone(),
                    (!resolved_fronts.is_empty()).then_some(resolved_fronts),
                )
                .inspect_err(|err| {
                    tracing::warn!("Failed to rebuild pruned URL for '{url}': {err}")
                })
                .ok()
            })
            .collect()
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
    async fn unresolvable_front_is_tolerated() -> Result<(), VpnApiClientError> {
        // A front is only ever a fallback route. If it fails to resolve but the primary
        // URLs are fine, the call should still succeed, just without that front available.
        let urls: Vec<Url> = vec![
            Url::new("https://nymvpn.com", None).unwrap(),
            Url::new(
                "https://validator.nymtech.net",
                Some(vec!["https://non-existent.nymtech.net"]),
            )
            .unwrap(),
        ];

        let overrides = ResolverOverrides::from_urls(&urls).await?;
        assert!(overrides.addresses("nymvpn.com").is_some());
        assert!(overrides.addresses("validator.nymtech.net").is_some());
        assert!(overrides.addresses("non-existent.nymtech.net").is_none());

        Ok(())
    }

    #[tokio::test]
    async fn overrides_return_error_when_no_primary_resolves() -> Result<(), VpnApiClientError> {
        let urls: Vec<Url> = vec![
            Url::new("https://non-existent-1.nymtech.net", None).unwrap(),
            Url::new(
                "https://non-existent-2.nymtech.net",
                Some(vec!["https://non-existent-3.nymtech.net"]),
            )
            .unwrap(),
        ];

        let result = ResolverOverrides::from_urls(&urls).await;
        assert!(result.is_err());

        let mut expected = HashSet::new();
        expected.insert("non-existent-1.nymtech.net".to_string());
        expected.insert("non-existent-2.nymtech.net".to_string());
        match result {
            Ok(_) => panic!("unreachable"),
            Err(VpnApiClientError::HostnamesResolutionError { hostnames }) => {
                assert_eq!(hostnames, expected)
            }
            Err(e) => panic!("unexpected err: {e}"),
        }
        Ok(())
    }

    #[tokio::test]
    async fn prune_drops_unresolvable_front_but_keeps_working_candidates() {
        let urls: Vec<Url> = vec![
            Url::new("https://nymvpn.com", None).unwrap(),
            Url::new(
                "https://validator.nymtech.net",
                Some(vec!["https://non-existent.nymtech.net"]),
            )
            .unwrap(),
        ];

        let pruned = ResolverOverrides::resolve_and_prune(&urls).await;
        assert_eq!(pruned.len(), 2);

        let validator = pruned
            .iter()
            .find(|url| url.inner_url().domain() == Some("validator.nymtech.net"))
            .expect("validator.nymtech.net should still be present");
        assert!(validator.fronts().unwrap_or_default().is_empty());

        let nymvpn = pruned
            .iter()
            .find(|url| url.inner_url().domain() == Some("nymvpn.com"))
            .expect("nymvpn.com should still be present");
        assert!(nymvpn.fronts().unwrap_or_default().is_empty());
    }

    #[tokio::test]
    async fn prune_drops_candidate_with_no_resolvable_route() {
        let urls: Vec<Url> = vec![
            Url::new("https://nymvpn.com", None).unwrap(),
            Url::new(
                "https://non-existent-1.nymtech.net",
                Some(vec!["https://non-existent-2.nymtech.net"]),
            )
            .unwrap(),
        ];

        let pruned = ResolverOverrides::resolve_and_prune(&urls).await;
        assert_eq!(pruned.len(), 1);
        assert_eq!(pruned[0].inner_url().domain(), Some("nymvpn.com"));
    }

    #[tokio::test]
    async fn resolve_and_prune_falls_back_to_original_when_nothing_resolves() {
        let urls: Vec<Url> = vec![
            Url::new("https://non-existent-1.nymtech.net", None).unwrap(),
            Url::new(
                "https://non-existent-2.nymtech.net",
                Some(vec!["https://non-existent-3.nymtech.net"]),
            )
            .unwrap(),
        ];

        let pruned = ResolverOverrides::resolve_and_prune(&urls).await;
        assert_eq!(pruned.len(), urls.len());
    }

    #[tokio::test]
    async fn resolve_and_prune_keeps_working_front_when_every_primary_fails() {
        // The primary is unresolvable but its front is fine: pruning should keep just the
        // front rather than discarding the resolved front data and falling back to the
        // original, unpruned (and known-bad) primary URL.
        let urls: Vec<Url> = vec![
            Url::new(
                "https://non-existent.nymtech.net",
                Some(vec!["https://nymvpn.com"]),
            )
            .unwrap(),
        ];

        let pruned = ResolverOverrides::resolve_and_prune(&urls).await;
        assert_eq!(pruned.len(), 1);
        assert_eq!(
            pruned[0].inner_url().domain(),
            Some("non-existent.nymtech.net")
        );
        assert_eq!(
            pruned[0]
                .fronts()
                .unwrap_or_default()
                .iter()
                .map(|f| f.domain())
                .collect::<Vec<_>>(),
            vec![Some("nymvpn.com")]
        );
    }
}
