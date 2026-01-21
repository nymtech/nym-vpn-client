// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only
#![warn(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

pub mod feature_flags;
pub mod system_messages;

mod account_management;
mod discovery;
mod discovery_refresher;
mod envs;
mod filetime;
mod nym_network;
mod nym_vpn_network;
mod serialization;
mod system_configuration;

pub use account_management::{AccountManagement, ParsedAccountLinks};
pub use discovery_refresher::{
    DiscoveryRefresher, DiscoveryRefresherCommand, DiscoveryRefresherEvent,
};
pub use feature_flags::{FeatureFlags, FlagValue};
use futures_util::FutureExt;
pub use nym_network::NymNetwork;
use nym_sdk::mixnet::Recipient;
use nym_vpn_api_client::{ResolverOverrides, str_to_socket_addr};
pub use nym_vpn_network::NymVpnNetwork;
pub use system_configuration::{ScoreThresholds, SystemConfiguration};
pub use system_messages::{SystemMessage, SystemMessages};

use discovery::Discovery;
use envs::RegisteredNetworks;
use nym_network_defaults::NymNetworkDetails;
use tokio::join;

use crate::{
    discovery::DiscoveryFromNymWellknownDiscoveryError,
    nym_vpn_network::{NymVpnNetworkAccountLinksConversionError, NymVpnNetworkFromDetailsError},
};

use nym_http_api_client::HttpClientError;
use std::{
    collections::HashSet,
    fmt::Debug,
    net::SocketAddr,
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};

// Refresh the discovery and network details files periodically
const MAX_FILE_AGE: Duration = Duration::from_secs(60 * 60);

pub type ApiUrl = nym_vpn_api_client::response::ApiUrl;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Network {
    pub nym_network: NymNetwork,
    pub nyxd_url: url::Url,
    pub nym_vpn_network: NymVpnNetwork,
    pub feature_flags: Option<FeatureFlags>,
    pub system_configuration: Option<SystemConfiguration>,
    pub account_management: Option<AccountManagement>,
}

impl Network {
    pub fn mainnet_default() -> Option<Self> {
        let network_details = NymNetworkDetails::new_mainnet();
        let nym_network = NymNetwork::new(network_details.clone());
        let nyxd_url = nym_network
            .network
            .endpoints
            .first()
            .map(|ep| ep.nyxd_url())?;
        Some(Network {
            nym_network,
            nyxd_url,
            nym_vpn_network: NymVpnNetwork::new(network_details),
            feature_flags: None,
            system_configuration: None,
            account_management: None,
        })
    }

    pub fn nym_network_details(&self) -> &NymNetworkDetails {
        &self.nym_network.network
    }

    pub fn export_to_env(&self) {
        self.nym_network.export_to_env();
        self.nym_vpn_network.export_to_env();
    }

    // Query the network name for both urls and check that it matches
    // TODO: integrate with validator-client and/or nym-vpn-api-client
    pub async fn check_consistency(&self) -> Result<bool> {
        tracing::debug!("Checking network consistency");
        let endpoint = self
            .nym_network
            .network
            .endpoints
            .first()
            .ok_or(Error::NoEndpointsFound)?;
        let nym_api_url = endpoint.api_url().ok_or(Error::NoApiUrlFound)?;
        let network_name = discovery::fetch_nym_network_details(nym_api_url)
            .map(|resp| resp.map(|d| d.network.network_name));

        let api_urls = &self.nym_vpn_network.nym_vpn_api_urls;
        let vpn_network_name = discovery::fetch_nym_vpn_network_details(api_urls)
            .map(|resp| resp.map(|d| d.network_name));

        let (network_name, vpn_network_name) = join!(network_name, vpn_network_name);
        let network_name = network_name?;
        let vpn_network_name = vpn_network_name?;

        tracing::debug!("nym network name: {network_name}");
        tracing::debug!("nym-vpn network name: {vpn_network_name}");
        Ok(network_name == vpn_network_name)
    }

    pub fn nyxd_url(&self) -> url::Url {
        self.nyxd_url.clone()
    }

    pub fn nym_api_urls(&self) -> Option<Vec<nym_network_defaults::ApiUrl>> {
        self.nym_network.network.nym_api_urls.clone()
    }

    pub fn nym_api_urls_as_urls(&self) -> Option<Vec<url::Url>> {
        self.nym_network.network.nym_api_urls.as_ref().map(|urls| {
            urls.iter()
                .filter_map(|api_url| url::Url::parse(&api_url.url).ok())
                .collect()
        })
    }

    pub fn nym_vpn_api_urls(&self) -> Option<Vec<nym_network_defaults::ApiUrl>> {
        self.nym_network.network.nym_vpn_api_urls.clone()
    }

    pub fn nym_vpn_api_urls_as_urls(&self) -> Option<Vec<url::Url>> {
        self.nym_network
            .network
            .nym_vpn_api_urls
            .as_ref()
            .map(|urls| {
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

    pub async fn resolver_overrides(&self) -> Result<ResolverOverrides> {
        ResolverOverrides::from_api_urls(&self.nym_vpn_network.nym_vpn_api_urls)
            .await
            .map_err(Error::CreateResolverOverrides)
    }
}

pub async fn discover_networks(config_path: &Path) -> Result<RegisteredNetworks> {
    RegisteredNetworks::ensure_exists(config_path).await
}

pub async fn discover_env(config_path: &Path, network_name: &str) -> Result<Network> {
    tracing::debug!(
        "Discovering network details: config_path={}, network_name={}",
        config_path.display(),
        network_name
    );

    // Lookup network discovery to bootstrap
    let discovery = Discovery::ensure_exists(config_path, network_name).await?;
    tracing::trace!("Discovery: {:#?}", discovery);

    tracing::trace!(
        "System messages: {}",
        discovery.system_messages.clone().into_current_messages()
    );

    network_from_discovery(config_path, discovery).await
}

pub async fn network_from_discovery(config_path: &Path, discovery: Discovery) -> Result<Network> {
    let feature_flags = discovery.feature_flags.clone();
    if let Some(ref feature_flags) = feature_flags {
        tracing::debug!("Feature flags: {}", feature_flags);
    }

    let system_configuration = discovery.system_configuration.clone();
    if let Some(ref system_configuration) = system_configuration {
        tracing::debug!("System configuration: {}", system_configuration);
    }

    // Using discovery, fetch and setup nym network details
    let mut nym_network = NymNetwork::ensure_exists(config_path, &discovery).await?;

    // Patch up the network details with domain fronting data
    // TODO: remove once network details contain domain fronting data
    if nym_network
        .network
        .nym_api_urls
        .as_ref()
        .is_none_or(|nym_api_urls| nym_api_urls.is_empty())
    {
        nym_network.network.nym_api_urls = Some(discovery.nym_api_urls().clone());
    }

    // Patch up the network details with domain fronting
    // TODO: remove once network details contain domain fronting data
    if nym_network
        .network
        .nym_vpn_api_urls
        .as_ref()
        .is_none_or(|nym_vpn_api_urls| nym_vpn_api_urls.is_empty())
    {
        nym_network.network.nym_vpn_api_urls = Some(discovery.nym_vpn_api_urls().clone());
    }

    let endpoint = nym_network
        .network
        .endpoints
        .first()
        .ok_or(Error::NoEndpointsFound)?;
    let nyxd_url = endpoint.nyxd_url();

    let account_management = discovery.account_management.clone();

    // Using discovery, setup nym vpn network details
    let nym_vpn_network = NymVpnNetwork::from(discovery);

    Ok(Network {
        nym_network,
        nyxd_url,
        nym_vpn_network,
        feature_flags,
        system_configuration,
        account_management,
    })
}

pub fn manual_env(network_details: &NymNetworkDetails) -> Result<Network> {
    let nym_network = NymNetwork::from(network_details.clone());
    let endpoint = nym_network
        .network
        .endpoints
        .first()
        .ok_or(Error::NoEndpointsFound)?;
    let nyxd_url = endpoint.nyxd_url();
    let nym_vpn_network =
        NymVpnNetwork::try_from(network_details).map_err(Error::ConvertNetworkDetailsToNetwork)?;

    Ok(Network {
        nym_network,
        nyxd_url,
        nym_vpn_network,
        feature_flags: None,
        system_configuration: None,
        account_management: None,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("no endpoints found in nym network")]
    NoEndpointsFound,

    #[error("no api url found in nym network")]
    NoApiUrlFound,

    #[error("network name mismatch between requested and fetched discovery")]
    NetworkNameMismatch { expected: String, actual: String },

    #[error("failed to obtain file staleness: {path}")]
    GetFileStaleness {
        path: PathBuf,
        source: filetime::FileTimeError,
    },

    #[error("failed to bootstrap api client")]
    CreateBootstrapApiClient(#[source] nym_vpn_api_client::error::VpnApiClientError),

    #[error("failed to create resolver overrides")]
    CreateResolverOverrides(#[source] nym_vpn_api_client::error::VpnApiClientError),

    #[error("failed to create vpn api client")]
    CreateVpnApiClient(#[source] nym_vpn_api_client::error::VpnApiClientError),

    #[error("failed to fetch well known current env")]
    GetWellKnownCurrentEnv(#[source] nym_vpn_api_client::error::VpnApiClientError),

    #[error("failed to fetch well known envs")]
    GetWellKnownEnvs(#[source] nym_vpn_api_client::error::VpnApiClientError),

    #[error("failed to fetch well known discovery")]
    GetWellKnownDiscovery(#[source] nym_vpn_api_client::error::VpnApiClientError),

    #[error("failed to get network details")]
    GetNetworkDetails(#[source] Box<HttpClientError>),

    #[error("failed to build http client: {0}")]
    FailedToBuildHttpClient(String),

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
    ConvertWellKnownDiscovery(#[source] DiscoveryFromNymWellknownDiscoveryError),

    #[error("failed to convert nym network details to nym vpn network")]
    ConvertNetworkDetailsToNetwork(#[source] NymVpnNetworkFromDetailsError),

    #[error("HTTP Client Error: {0}")]
    HttpClient(#[from] Box<HttpClientError>),

    #[error("inconsistent network detected")]
    InconsistentNetwork,

    #[error("failed to refresh discovery file")]
    RefreshDiscoveryFile,

    #[error("failed to parse refreshed discovery file")]
    ParseDiscoveryFile,
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
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
