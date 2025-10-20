use crate::{api_urls_to_urls, error::VpnApiClientError, url_to_socket_addr};
use nym_http_api_client::Url;
use nym_network_defaults::ApiUrl;
use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
};

#[derive(Clone, Debug, Default)]
pub struct ResolverOverrides {
    overrides: HashMap<String, HashSet<SocketAddr>>,
}

impl ResolverOverrides {
    /// Create a new set of resolver overrides from the provided URLs.
    pub async fn from_urls(urls: &[Url]) -> Result<Self, VpnApiClientError> {
        let mut overrides = HashMap::new();

        for url in urls {
            let Some(domain) = url.inner_url().domain() else {
                tracing::warn!(
                    "Ignoring API URL '{}' for resolver overrides as it does not have a valid domain",
                    url.to_string()
                );
                continue;
            };

            let addresses = url_to_socket_addr(url.inner_url(), Some((1, 1))).await?;
            overrides.insert(
                domain.to_string(),
                HashSet::from_iter(addresses.into_iter()),
            );

            if let Some(fronts) = url.fronts() {
                for front_url in fronts {
                    let Some(front_domain) = front_url.domain() else {
                        tracing::warn!(
                            "Ignoring front host URL '{}' for resolver overrides as it does not have a valid domain",
                            front_url
                        );
                        continue;
                    };
                    let front_addresses = url_to_socket_addr(front_url, Some((1, 1))).await?;
                    overrides.insert(
                        front_domain.to_string(),
                        HashSet::from_iter(front_addresses.into_iter()),
                    );
                }
            }
        }

        Ok(Self { overrides })
    }

    /// Create resolver overrides from the provides ApiUrls
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
    pub fn domain_addrs(&self, domain: &str) -> Option<Vec<SocketAddr>> {
        self.overrides
            .get(domain)
            .map(|addrs| addrs.iter().cloned().collect())
    }

    /// Get all the socket addresses
    pub fn all_socket_addrs(&self) -> Vec<SocketAddr> {
        self.overrides
            .values()
            .flat_map(|addrs| addrs.iter().cloned())
            .collect()
    }
}
