// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod feature_flags;
pub mod system_messages;

mod account_management;
mod discovery;
mod discovery_refresher;
mod envs;
mod fetcher;
mod nym_vpn_network;
mod persistent_discovery;
mod persistent_envs;
mod persistent_network_details;
mod serialization;
mod system_configuration;

use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    net::{IpAddr, SocketAddr},
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};

use serde::{Deserialize, Serialize};
use time::UtcDateTime;

pub use account_management::{AccountManagement, ParsedAccountLinks};
pub use discovery::Discovery;
pub use discovery_refresher::{DiscoveryRefresher, DiscoveryRefresherCommand};
pub use envs::RegisteredNetworks;
pub use feature_flags::{FeatureFlags, FlagValue};
pub use fetcher::Fetcher;
pub use nym_network_defaults::v2::NymNetworkDetails;
pub use nym_vpn_network::NymVpnNetwork;
pub use system_configuration::{ScoreThresholds, SystemConfiguration};
pub use system_messages::{SystemMessage, SystemMessages};

use nym_common::trace_err_chain;
use nym_http_api_client::HttpClientError;
use nym_network_defaults::v2::DnsFallback;
use nym_sdk::{UserAgent, mixnet::Recipient};
use nym_vpn_api_client::str_to_socket_addr;

use crate::{
    discovery::DiscoveryFromNymWellknownDiscoveryError,
    nym_vpn_network::{NymVpnNetworkAccountLinksConversionError, NymVpnNetworkFromDetailsError},
    persistent_discovery::PersistentDiscovery,
    persistent_envs::PersistentEnvs,
    persistent_network_details::PersistentNetworkDetails,
};

// Refresh the discovery and network details files periodically
const MAX_FILE_AGE: Duration = Duration::from_secs(60 * 60);
const NETWORKS_SUBDIR: &str = "networks";

pub type ApiUrl = nym_vpn_api_client::response::ApiUrl;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Network {
    pub nym_network: NymNetworkDetails,
    pub nyxd_url: url::Url,
    pub nym_vpn_network: NymVpnNetwork,
    pub feature_flags: Option<FeatureFlags>,
    pub system_configuration: Option<SystemConfiguration>,
    dns_fallbacks: HashMap<String, HashSet<IpAddr>>,
}

fn dns_fallback_addr_map(fallbacks: &[DnsFallback]) -> HashMap<String, HashSet<IpAddr>> {
    fallbacks
        .iter()
        .filter_map(|fallback| {
            let addrs: HashSet<IpAddr> = fallback
                .addresses
                .iter()
                .filter_map(|addr| {
                    addr.parse()
                        .inspect_err(|err| {
                            tracing::warn!(
                                "Invalid dns fallback address '{addr}' for '{}': {err}",
                                fallback.url
                            );
                        })
                        .ok()
                })
                .collect();

            if addrs.is_empty() {
                None
            } else {
                Some((fallback.url.clone(), addrs))
            }
        })
        .collect()
}

impl Network {
    /// Returns pre-bundled mainnet network configuration.
    /// This call must never fail unless the bundled data is bogus.
    pub fn mainnet_default() -> Result<Self> {
        Self::new_from_discovery(
            Discovery::default_mainnet(),
            NymNetworkDetails::new_mainnet(),
        )
    }

    /// Create new network configuration from discovery and network details.
    pub fn new_from_discovery(
        discovery: Discovery,
        network_details: NymNetworkDetails,
    ) -> Result<Self> {
        if discovery.network_name != network_details.network_name {
            return Err(Error::NetworkNameMismatch {
                expected: discovery.network_name,
                actual: network_details.network_name,
            });
        }

        let feature_flags = discovery.feature_flags.clone();
        let system_configuration = discovery.system_configuration.clone();
        let dns_fallbacks = dns_fallback_addr_map(&network_details.networking.dns_fallbacks);
        let endpoint = network_details
            .endpoints
            .first()
            .ok_or(Error::NoEndpointsFound)?;
        let nyxd_url = endpoint.nyxd_url();
        let nym_vpn_network = NymVpnNetwork::from(discovery);

        Ok(Self {
            nym_network: network_details,
            nyxd_url,
            nym_vpn_network,
            feature_flags,
            system_configuration,
            dns_fallbacks,
        })
    }

    /// Map of hostname to fallback IP addresses to use for DNS resolution when the primary
    /// resolver fails, as configured by discovery.
    pub fn dns_fallback_addr_map(&self) -> HashMap<String, HashSet<IpAddr>> {
        self.dns_fallbacks.clone()
    }

    pub fn nym_network_details(&self) -> &NymNetworkDetails {
        &self.nym_network
    }

    pub fn export_to_env(&self) {
        nym_network_defaults::NymNetworkDetails::from(self.nym_network.clone()).export_to_env();
        self.nym_vpn_network.export_to_env();
    }

    pub fn nyxd_url(&self) -> url::Url {
        self.nyxd_url.clone()
    }

    pub fn nym_api_urls(&self) -> Option<Vec<nym_network_defaults::ApiUrl>> {
        let urls = self.nym_network.nym_api_urls();
        (!urls.is_empty()).then_some(urls)
    }

    pub fn nym_api_urls_as_urls(&self) -> Option<Vec<url::Url>> {
        self.nym_api_urls().map(|urls| {
            urls.iter()
                .filter_map(|api_url| url::Url::parse(&api_url.url).ok())
                .collect()
        })
    }

    pub fn nym_vpn_api_urls(&self) -> Option<Vec<nym_network_defaults::ApiUrl>> {
        let urls = self.nym_network.nym_vpn_api_urls();
        (!urls.is_empty()).then_some(urls)
    }

    pub fn nym_vpn_api_urls_as_urls(&self) -> Option<Vec<url::Url>> {
        self.nym_vpn_api_urls().map(|urls| {
            urls.iter()
                .filter_map(|api_url| url::Url::parse(&api_url.url).ok())
                .collect()
        })
    }

    pub fn get_simple_feature_flag<T>(&self, flag: &str) -> Option<T>
    where
        T: FromStr + Debug,
        <T as FromStr>::Err: Debug,
    {
        tracing::debug!("Getting simple feature flag: flag={}", flag);
        self.feature_flags
            .as_ref()
            .and_then(|ff| ff.get_flag(flag))
            .and_then(|value| match value {
                FlagValue::Value(value) => value
                    .parse::<T>()
                    .inspect_err(|e| tracing::warn!("Failed to parse flag value: {e:#?}"))
                    .ok(),
                _ => None,
            })
    }

    pub fn stats_recipient(&self) -> Option<Recipient> {
        self.feature_flags
            .as_ref()
            .and_then(|ff| ff.stats_recipient())
    }

    /// Get the version of the gateway from where the metadata endpoint should start to be used
    pub fn gw_update_version(&self) -> Option<semver::Version> {
        self.feature_flags
            .as_ref()
            .and_then(|ff| ff.gw_update_version())
    }

    pub fn domain_fronting_enabled(&self) -> Option<bool> {
        self.feature_flags
            .as_ref()
            .and_then(|ff| ff.domain_fronting_enabled())
    }

    pub fn quic_enabled(&self) -> Option<bool> {
        self.feature_flags.as_ref().and_then(|ff| ff.quic_enabled())
    }

    pub fn privy_enabled(&self) -> Option<bool> {
        self.feature_flags
            .as_ref()
            .and_then(|ff| ff.privy_enabled())
    }

    pub async fn vpn_api_addresses(&self) -> Vec<SocketAddr> {
        let mut unique: HashSet<SocketAddr> = HashSet::with_capacity(16);

        for api_url in self.nym_vpn_network.nym_vpn_api_urls.iter() {
            if let Some(fronts) = api_url.front_hosts.as_ref() {
                for front in fronts {
                    match str_to_socket_addr(front, Some((1, 1))).await {
                        Ok(addrs) => {
                            for addr in addrs {
                                unique.insert(addr);
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to resolve address of front: {front}: {e}");
                        }
                    }
                }
            }
        }

        unique.into_iter().collect()
    }
}

/// Supervisor type over persistent stores concerning network configuration, such as registered network environments, discovery, and network details.
#[derive(Debug)]
pub struct NetworkCache {
    cache_dir: PathBuf,
    persistent_envs: PersistentEnvs,
    persistent_discovery: PersistentDiscovery,
    persistent_network_details: Option<PersistentNetworkDetails>,
    fetcher: Fetcher,
}

impl NetworkCache {
    pub async fn new(
        cache_dir: PathBuf,
        network_name: &str,
        user_agent: Option<UserAgent>,
    ) -> Result<Self> {
        Self::clean_up_change_introduced_in_pr4226(&cache_dir).await;

        let persistent_envs = PersistentEnvs::new_from_cache(cache_dir.clone()).await?;
        let persistent_discovery =
            PersistentDiscovery::new_from_cache(cache_dir.clone(), network_name).await?;
        let persistent_network_details =
            PersistentNetworkDetails::new_from_cache(cache_dir.clone(), network_name)
                .await
                .map(Some)
                .or_else(|err| {
                    if err.is_no_default_network_details() {
                        Ok(None)
                    } else {
                        Err(err)
                    }
                })?;

        let fetcher = Fetcher::new(persistent_discovery.value().clone(), user_agent)?;

        Ok(Self {
            cache_dir,
            persistent_envs,
            persistent_discovery,
            persistent_network_details,
            fetcher,
        })
    }

    pub async fn fetch_if_stale(&mut self) -> Result<()> {
        // Refresh registered networks
        if self.persistent_envs.is_stale() {
            let new_networks = self.fetcher.fetch_registered_networks().await?;
            self.persistent_envs.update(new_networks).await?;
        }

        // Refresh discovery
        if self.persistent_discovery.is_stale() {
            let network_name = self.persistent_discovery.network_name();
            let new_discovery = self.fetcher.fetch_discovery(network_name).await?;

            // Update fetcher discovery so that it could pick up new API endpoints if they changed.
            if new_discovery != *self.persistent_discovery.value()
                && let Err(err) = self.fetcher.set_discovery(new_discovery.clone())
            {
                trace_err_chain!(err, "failed to update fetcher discovery");
            }

            self.persistent_discovery.update(new_discovery).await?;
        }

        // Refresh network details
        match self.persistent_network_details {
            Some(ref mut details) => {
                if details.is_stale() {
                    let new_network_details = self.fetcher.fetch_network_details().await?;
                    details.update(*new_network_details).await?;
                }
            }
            ref mut details @ None => {
                let new_network_details = self.fetcher.fetch_network_details().await?;
                let new_persistent_network_details =
                    PersistentNetworkDetails::new_with_newly_fetched(
                        self.cache_dir.clone(),
                        *new_network_details,
                    )
                    .await?;
                details.replace(new_persistent_network_details);
            }
        };

        Ok(())
    }

    /// Returns current network configuration based on discovery and network details held in persistent store.
    /// This call will fail if the network details are not fetched yet which can the case for non-mainnet environments.
    /// In such case use `fetch_if_stale` to fetch the network details from network.
    pub fn network(&self) -> Result<Box<Network>> {
        let discovery = self.persistent_discovery.value().clone();
        let mut network_details = self
            .persistent_network_details
            .as_ref()
            .ok_or(Error::NetworkDetailsNotFetched)?
            .value()
            .clone();

        Self::patch_network_details_from_discovery(&mut network_details, &discovery);

        let network_env = Network::new_from_discovery(discovery, network_details)?;

        Ok(Box::new(network_env))
    }

    // Patch network details from discovery.
    //
    // Sometimes deployments go wrong and nym-vpn-api-urls aren't set properly which can be a show stopper.
    // Patch it up manually from discovery since some of VpnClient initializers use network details.
    fn patch_network_details_from_discovery(
        network_details: &mut NymNetworkDetails,
        discovery: &Discovery,
    ) {
        if network_details.nym_vpn_api_urls().is_empty() {
            tracing::debug!(
                "Patching up network details from discovery due to missing network details!"
            );
            network_details.networking.nym_vpn_api_urls = discovery.nym_vpn_api_urls();
        }
    }

    /// Query registered networks held in persistent store.
    pub fn registered_networks(&self) -> &RegisteredNetworks {
        self.persistent_envs.value()
    }

    /// Query discovery held in persistent store.
    pub fn discovery(&self) -> &Discovery {
        self.persistent_discovery.value()
    }

    // Clean up change introduced in https://github.com/nymtech/nym-vpn-client/pull/4226
    // Network files were moved from <cache_dir>/networks/<env> to <cache_dir>/<env>
    async fn clean_up_change_introduced_in_pr4226(cache_dir: &Path) {
        for env in ["mainnet", "sandbox", "canary"] {
            let path = cache_dir.join(env);

            tokio::fs::remove_file(path.join(format!("{env}.json",)))
                .await
                .ok();
            tokio::fs::remove_file(path.join(format!("{env}_discovery.json")))
                .await
                .ok();
            tokio::fs::remove_dir(path).await.ok();
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("no endpoints found in nym network")]
    NoEndpointsFound,

    #[error("no default network details available for {0}")]
    NoDefaultNetworkDetails(String),

    #[error("network name mismatch between requested and fetched discovery")]
    NetworkNameMismatch { expected: String, actual: String },

    #[error("failed to create vpn api client")]
    CreateVpnApiClient(#[source] nym_vpn_api_client::error::VpnApiClientError),

    #[error("failed to create http api client")]
    CreateHttpApiClient(#[source] Box<HttpClientError>),

    #[error("failed to fetch well known envs")]
    GetWellKnownEnvs(#[source] nym_vpn_api_client::error::VpnApiClientError),

    #[error("failed to fetch well known discovery")]
    GetWellKnownDiscovery(#[source] nym_vpn_api_client::error::VpnApiClientError),

    #[error("failed to get network details")]
    GetNetworkDetails(#[source] Box<HttpClientError>),

    #[error("failed to create parent directories for discovery file: {path}")]
    CreateParentDirs {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to open file: {path}")]
    OpenFile {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to serialize data to file: {path}")]
    Serialize {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("failed to deserialize file: {path}")]
    Deserialize {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("failed to obtain account links")]
    GetAccountLinks(#[from] NymVpnNetworkAccountLinksConversionError),

    #[error("failed to convert well known discovery response into discovery")]
    ConvertWellKnownDiscovery(#[from] DiscoveryFromNymWellknownDiscoveryError),

    #[error("failed to convert nym network details to nym vpn network")]
    ConvertNetworkDetailsToNetwork(#[source] NymVpnNetworkFromDetailsError),

    #[error("unknown discovery: {0}")]
    UnknownDiscovery(String),

    #[error("network details are not fetched")]
    NetworkDetailsNotFetched,
}

impl Error {
    /// Returns true if file cannot be opened because it's not found.
    pub(crate) fn is_file_not_found(&self) -> bool {
        matches!(self, Self::OpenFile { source, .. } if source.kind() == std::io::ErrorKind::NotFound)
    }

    /// Returns true if file contents cannot be deserialized indicating that the file is likely corrupt.
    /// For convenience, returns true if file does not exist.
    pub(crate) fn should_overwrite_file(&self) -> bool {
        match self {
            Self::OpenFile { source, .. } => source.kind() == std::io::ErrorKind::NotFound,
            Self::Deserialize { source, .. } => {
                // everything except i/o error indicates deserialization problem
                !source.is_io()
            }
            _ => false,
        }
    }

    /// Returns true if construction of persistent network details failed because cache was empty
    /// and no pre-bundled default network details available. This is typically the case for non-mainnet environments.
    pub(crate) fn is_no_default_network_details(&self) -> bool {
        matches!(self, Self::NoDefaultNetworkDetails(_))
    }

    /// Returns true if network data are inconsistent.
    #[cfg(test)]
    pub(crate) fn is_inconsistent_network(&self) -> bool {
        matches!(self, Self::NetworkNameMismatch { .. })
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// On-disk representation of persistent store record.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct PersistentRecord<T> {
    /// Timestamp of the last update.
    /// When `None`, indicates that the entry is stale, which can be useful when the store is initialized from pre-bundled defaults.
    updated_at: Option<UtcDateTime>,

    /// Value held in the record.
    value: T,
}

impl<T> PersistentRecord<T> {
    /// Returns new PersistentRecord with provided value and without timestamp.
    fn stale(value: T) -> Self {
        Self {
            updated_at: None,
            value,
        }
    }

    /// Returns new PersistentRecord with provided value and current timestamp.
    fn up_to_date(value: T) -> Self {
        Self {
            updated_at: Some(UtcDateTime::now()),
            value,
        }
    }

    fn is_stale(&self) -> bool {
        match self.updated_at {
            Some(updated_at) => {
                let diff = UtcDateTime::now() - updated_at;
                diff > MAX_FILE_AGE
            }
            None => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[tokio::test]
    async fn test_network_cache_handles_cleanup_pr4226() {
        let cache_dir = tempdir().unwrap();

        let envs = ["mainnet", "sandbox", "canary"];

        for env in envs {
            let base_dir = cache_dir.path().join(env);
            tokio::fs::create_dir(&base_dir).await.unwrap();
            let _ = tokio::fs::File::create(base_dir.join(format!("{env}.json"))).await;
            let _ = tokio::fs::File::create(base_dir.join(format!("{env}_discovery.json"))).await;
        }

        let _ = tokio::fs::File::create(cache_dir.path().join("test.txt")).await;

        let _network_cache = NetworkCache::new(cache_dir.path().to_path_buf(), "mainnet", None)
            .await
            .unwrap();

        // ensure network cache removed old directories
        for env in envs {
            let base_dir = cache_dir.path().join(env);
            assert!(!tokio::fs::try_exists(base_dir).await.unwrap())
        }

        // ensure network cache does not remove anything else
        assert!(
            tokio::fs::try_exists(cache_dir.path().join("networks/mainnet"))
                .await
                .unwrap()
        );
        assert!(
            tokio::fs::try_exists(cache_dir.path().join("test.txt"))
                .await
                .unwrap()
        );
    }

    #[test]
    fn test_mainnet_default_network_has_dns_fallback_addrs() {
        let network = Network::mainnet_default().unwrap();
        let fallbacks = network.dns_fallback_addr_map();

        assert!(!fallbacks.is_empty());
        for (host, addrs) in &fallbacks {
            assert!(!host.is_empty());
            assert!(!addrs.is_empty());
        }
    }
}
