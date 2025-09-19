// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::error::{Error, Result};
use nym_common::trace_err_chain;
use nym_vpn_lib::{
    MixnetClientConfig, NodeIdentity,
    gateway_directory::{self, EntryPoint, ExitPoint},
    tunnel_state_machine::{
        DnsOptions, GatewayPerformanceOptions, MixnetTunnelOptions, TunnelSettings,
        WireguardMultihopMode, WireguardTunnelOptions,
    },
};
use nym_vpn_lib_types::TunnelType;
use nym_vpnd_types::service::VpnServiceConfig;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    fmt, fs,
    net::IpAddr,
    path::{Path, PathBuf},
    str::FromStr,
};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[cfg(not(windows))]
const DEFAULT_DATA_DIR: &str = "/var/lib/nym-vpnd";
#[cfg(not(windows))]
const DEFAULT_LOG_DIR: &str = "/var/log/nym-vpnd";
#[cfg(not(windows))]
const DEFAULT_CONFIG_DIR: &str = "/etc/nym";
pub const DEFAULT_CONFIG_FILE_TOML: &str = "nym-vpnd.toml";
pub const DEFAULT_CONFIG_FILE_JSON: &str = "nym-vpnd.json";
pub const DEFAULT_LOG_FILE: &str = "nym-vpnd.log";
pub const DEFAULT_OLD_LOG_FILE: &str = "nym-vpnd.old.log";

pub const DEFAULT_GLOBAL_CONFIG_FILE_TOML: &str = "config.toml";
pub const DEFAULT_GLOBAL_CONFIG_FILE_JSON: &str = "config.json";

//
// NetworkEnvironments
//

#[derive(Debug, Clone)]
pub enum NetworkEnvironments {
    Mainnet,
    Sandbox,
    Canary,
    Qa,
}

impl fmt::Display for NetworkEnvironments {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkEnvironments::Mainnet => write!(f, "mainnet"),
            NetworkEnvironments::Sandbox => write!(f, "sandbox"),
            NetworkEnvironments::Canary => write!(f, "canary"),
            NetworkEnvironments::Qa => write!(f, "qa"),
        }
    }
}

impl TryFrom<&str> for NetworkEnvironments {
    type Error = &'static str;

    fn try_from(env: &str) -> Result<Self, Self::Error> {
        match env {
            "mainnet" => Ok(NetworkEnvironments::Mainnet),
            "sandbox" => Ok(NetworkEnvironments::Sandbox),
            "canary" => Ok(NetworkEnvironments::Canary),
            "qa" => Ok(NetworkEnvironments::Qa),
            _ => Err("Invalid network environment"),
        }
    }
}

//
// VpnServiceConfigManager
//

pub struct VpnServiceConfigManager {
    json_config_path: PathBuf,
    config: VpnServiceConfig,
}

#[allow(dead_code)]
impl VpnServiceConfigManager {
    pub fn new(network_config_dir: &Path) -> Result<Self> {
        let toml_config_path = network_config_dir.join(DEFAULT_CONFIG_FILE_TOML);
        let json_config_path = network_config_dir.join(DEFAULT_CONFIG_FILE_JSON);
        let (config, version) = Self::read_from_file(&toml_config_path, &json_config_path)?;

        let config_manager = Self {
            json_config_path,
            config,
        };

        // If we didn't read the latest version then write the config straight back to file
        if version != LATEST_CONFIG_VERSION {
            config_manager.write_to_file();
        }

        // If the deprecated TOML file exists then remove it
        if toml_config_path.exists() {
            tracing::info!(
                "Removing deprecated config file {}",
                toml_config_path.display()
            );
            if let Err(e) = fs::remove_file(&toml_config_path) {
                trace_err_chain!(e, "Failed to remove deprecated config file");
            }
        }

        Ok(config_manager)
    }

    pub fn config(&self) -> &VpnServiceConfig {
        &self.config
    }

    pub fn set_config(&mut self, config: VpnServiceConfig) {
        self.config = config;
        let _ = self.write_to_file();
    }

    pub fn set_entry_point(&mut self, entry_point: EntryPoint) {
        self.config.entry_point = entry_point;
        let _ = self.write_to_file();
    }

    pub fn set_exit_point(&mut self, exit_point: ExitPoint) {
        self.config.exit_point = exit_point;
        let _ = self.write_to_file();
    }

    pub fn set_dns(&mut self, dns: Option<IpAddr>) {
        self.config.dns = dns;
        let _ = self.write_to_file();
    }

    pub fn set_disable_ipv6(&mut self, disable_ipv6: bool) {
        self.config.disable_ipv6 = disable_ipv6;
        let _ = self.write_to_file();
    }

    pub fn set_enable_two_hop(&mut self, enable_two_hop: bool) {
        self.config.enable_two_hop = enable_two_hop;
        let _ = self.write_to_file();
    }

    pub fn set_netstack(&mut self, netstack: bool) {
        self.config.netstack = netstack;
        let _ = self.write_to_file();
    }

    pub fn set_disable_poisson_rate(&mut self, disable_poisson_rate: bool) {
        self.config.disable_poisson_rate = disable_poisson_rate;
        let _ = self.write_to_file();
    }

    pub fn set_disable_background_cover_traffic(&mut self, disable: bool) {
        self.config.disable_background_cover_traffic = disable;
        let _ = self.write_to_file();
    }

    pub fn set_min_mixnode_performance(&mut self, min_mixnode_performance: Option<u8>) {
        self.config.min_mixnode_performance = min_mixnode_performance.map(|u| u.min(100));
        let _ = self.write_to_file();
    }

    pub fn set_min_gateway_mixnet_performance(
        &mut self,
        min_gateway_mixnet_performance: Option<u8>,
    ) {
        self.config.min_gateway_mixnet_performance =
            min_gateway_mixnet_performance.map(|u| u.min(100));
        let _ = self.write_to_file();
    }

    pub fn set_min_gateway_vpn_performance(&mut self, min_gateway_vpn_performance: Option<u8>) {
        self.config.min_gateway_vpn_performance = min_gateway_vpn_performance.map(|u| u.min(100));
        let _ = self.write_to_file();
    }

    /// Returns the configuration as well as the version read from file.
    fn read_from_file(
        toml_config_path: &Path,
        json_config_path: &Path,
    ) -> Result<(VpnServiceConfig, u8)> {
        let (config, version) = if json_config_path.exists() {
            let ext_config = read_json_config_file::<VpnServiceConfigExt>(json_config_path)
                .map_err(Error::ConfigSetup)?;
            let version = ext_config.version();

            tracing::info!("Loaded service config from {}", json_config_path.display());

            let config = VpnServiceConfig::try_from(ext_config).map_err(Error::ConfigSetup)?;

            (config, version)
        } else if toml_config_path.exists() {
            let legacy_config = read_toml_config_file::<LegacyVpnServiceConfig>(toml_config_path)
                .map_err(Error::ConfigSetup)?;

            tracing::info!("Loaded service config from {}", toml_config_path.display());

            let config = VpnServiceConfig::try_from(legacy_config).map_err(Error::ConfigSetup)?;

            (config, 0)
        } else {
            tracing::info!("Using default service config");

            (VpnServiceConfig::default(), 0)
        };

        Ok((config, version))
    }

    fn write_to_file(&self) -> bool {
        let ext_config = match VpnServiceConfigExt::try_from(&self.config)
            .map_err(Error::ConfigSetup)
        {
            Ok(ext_config) => ext_config,
            Err(e) => {
                tracing::error!("Failed to convert service config to external representation: {e}");
                return false;
            }
        };

        match write_json_config_file(&self.json_config_path, &ext_config)
            .map_err(Error::ConfigSetup)
        {
            Ok(_) => {
                tracing::info!(
                    "Saved service config to {}",
                    self.json_config_path.display()
                );
                true
            }
            Err(e) => {
                tracing::error!(
                    "Failed to write service config to {}: {e}",
                    self.json_config_path.display()
                );
                false
            }
        }
    }

    pub fn generate_tunnel_settings(&self) -> TunnelSettings {
        tracing::debug!("Using config: {:?}", self.config);

        let gateway_options = GatewayPerformanceOptions {
            mixnet_min_performance: self.config.min_gateway_mixnet_performance,
            vpn_min_performance: self.config.min_gateway_vpn_performance,
        };

        let mixnet_client_config = MixnetClientConfig {
            disable_poisson_rate: self.config.disable_poisson_rate,
            disable_background_cover_traffic: self.config.disable_background_cover_traffic,
            min_mixnode_performance: self.config.min_mixnode_performance,
            min_gateway_performance: self.config.min_gateway_mixnet_performance,
        };

        let tunnel_type = if self.config.enable_two_hop {
            TunnelType::Wireguard
        } else {
            TunnelType::Mixnet
        };

        let dns = self
            .config
            .dns
            .map(|addr| DnsOptions::Custom(vec![addr]))
            .unwrap_or_default();

        TunnelSettings {
            enable_ipv6: !self.config.disable_ipv6,
            tunnel_type,
            mixnet_tunnel_options: MixnetTunnelOptions { mtu: None },
            wireguard_tunnel_options: WireguardTunnelOptions {
                multihop_mode: if self.config.netstack {
                    WireguardMultihopMode::Netstack
                } else {
                    WireguardMultihopMode::TunTun
                },
            },
            gateway_performance_options: gateway_options,
            mixnet_client_config: Some(mixnet_client_config),
            entry_point: Box::new(self.config.entry_point.clone()),
            exit_point: Box::new(self.config.exit_point.clone()),
            dns,
            user_agent: None,
        }
    }
}

//
// External, versioned, representation of the vpn service config file.
//

type VpnServiceConfigExtLatest = VpnServiceConfigExtV2;
const LATEST_CONFIG_VERSION: u8 = 2;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "version")]
#[serde(rename_all = "snake_case")]
enum VpnServiceConfigExt {
    V1(VpnServiceConfigExtV1),
    V2(VpnServiceConfigExtV2),
}

impl VpnServiceConfigExt {
    fn version(&self) -> u8 {
        match self {
            VpnServiceConfigExt::V1(_) => 1,
            VpnServiceConfigExt::V2(_) => 2,
        }
    }
}

impl TryFrom<VpnServiceConfigExt> for VpnServiceConfig {
    type Error = ConfigSetupError;

    fn try_from(value: VpnServiceConfigExt) -> Result<Self, Self::Error> {
        match value {
            VpnServiceConfigExt::V1(v1) => VpnServiceConfig::try_from(v1),
            VpnServiceConfigExt::V2(v2) => VpnServiceConfig::try_from(v2),
        }
    }
}

impl TryFrom<&VpnServiceConfig> for VpnServiceConfigExt {
    type Error = ConfigSetupError;

    fn try_from(value: &VpnServiceConfig) -> Result<Self, Self::Error> {
        // Always construct the latest external representation
        let latest = VpnServiceConfigExtLatest::try_from(value)?;
        Ok(latest.into())
    }
}

//
// v1
//

#[derive(Clone, Debug, Serialize, Deserialize)]
struct VpnServiceConfigExtV1 {
    entry_point: EntryPointExtV1,
    exit_point: ExitPointExtV1,
}

impl From<VpnServiceConfigExtV1> for VpnServiceConfigExt {
    fn from(v1: VpnServiceConfigExtV1) -> Self {
        VpnServiceConfigExt::V1(v1)
    }
}

impl TryFrom<VpnServiceConfigExtV1> for VpnServiceConfig {
    type Error = ConfigSetupError;

    fn try_from(value: VpnServiceConfigExtV1) -> Result<Self, Self::Error> {
        let config = VpnServiceConfig {
            entry_point: EntryPoint::try_from(value.entry_point)?,
            exit_point: ExitPoint::try_from(value.exit_point)?,
            ..Default::default()
        };
        Ok(config)
    }
}

//
// v2
//

#[derive(Clone, Debug, Serialize, Deserialize)]
struct VpnServiceConfigExtV2 {
    entry_point: EntryPointExtV1,
    exit_point: ExitPointExtV1,
    dns: Option<String>,
    disable_ipv6: bool,
    enable_two_hop: bool,
    netstack: bool,
    disable_poisson_rate: bool,
    disable_background_cover_traffic: bool,
    min_mixnode_performance: Option<u8>,
    min_gateway_mixnet_performance: Option<u8>,
    min_gateway_vpn_performance: Option<u8>,
}

impl From<VpnServiceConfigExtV2> for VpnServiceConfigExt {
    fn from(v2: VpnServiceConfigExtV2) -> Self {
        VpnServiceConfigExt::V2(v2)
    }
}

impl TryFrom<VpnServiceConfigExtV2> for VpnServiceConfig {
    type Error = ConfigSetupError;

    fn try_from(value: VpnServiceConfigExtV2) -> Result<Self, Self::Error> {
        let dns = value
            .dns
            .map(|addr| {
                IpAddr::from_str(&addr)
                    .map_err(|e| ConfigSetupError::IpAddress { error: Box::new(e) })
            })
            .transpose()?;

        let config = VpnServiceConfig {
            entry_point: EntryPoint::try_from(value.entry_point)?,
            exit_point: ExitPoint::try_from(value.exit_point)?,
            dns,
            disable_ipv6: value.disable_ipv6,
            enable_two_hop: value.enable_two_hop,
            netstack: value.netstack,
            disable_poisson_rate: value.disable_poisson_rate,
            disable_background_cover_traffic: value.disable_background_cover_traffic,
            min_mixnode_performance: value.min_mixnode_performance,
            min_gateway_mixnet_performance: value.min_gateway_mixnet_performance,
            min_gateway_vpn_performance: value.min_gateway_vpn_performance,
        };
        Ok(config)
    }
}

//
// Latest (v2)
//

impl TryFrom<&VpnServiceConfig> for VpnServiceConfigExtLatest {
    type Error = ConfigSetupError;

    fn try_from(value: &VpnServiceConfig) -> Result<Self, Self::Error> {
        let ext_config = VpnServiceConfigExtLatest {
            entry_point: EntryPointExtV1::try_from(&value.entry_point)?,
            exit_point: ExitPointExtV1::try_from(&value.exit_point)?,
            dns: value.dns.map(|addr| addr.to_string()),
            disable_ipv6: value.disable_ipv6,
            enable_two_hop: value.enable_two_hop,
            netstack: value.netstack,
            disable_poisson_rate: value.disable_poisson_rate,
            disable_background_cover_traffic: value.disable_background_cover_traffic,
            min_mixnode_performance: value.min_mixnode_performance,
            min_gateway_mixnet_performance: value.min_gateway_mixnet_performance,
            min_gateway_vpn_performance: value.min_gateway_vpn_performance,
        };
        Ok(ext_config)
    }
}

//
// EntryPointExtV1
//

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum EntryPointExtV1 {
    Gateway { identity: String },
    Location { location: String },
    Random,
}

impl TryFrom<EntryPointExtV1> for EntryPoint {
    type Error = ConfigSetupError;

    fn try_from(value: EntryPointExtV1) -> Result<Self, Self::Error> {
        match value {
            EntryPointExtV1::Gateway { ref identity } => EntryPoint::from_base58_string(identity)
                .map_err(|e| ConfigSetupError::EntryPoint { error: Box::new(e) }),
            EntryPointExtV1::Location { location } => Ok(EntryPoint::Country {
                two_letter_iso_country_code: location,
            }),
            EntryPointExtV1::Random => Ok(EntryPoint::Random),
        }
    }
}

impl TryFrom<&EntryPoint> for EntryPointExtV1 {
    type Error = ConfigSetupError;

    fn try_from(value: &EntryPoint) -> Result<Self, Self::Error> {
        match value {
            EntryPoint::Gateway { identity } => Ok(EntryPointExtV1::Gateway {
                identity: identity.to_base58_string(),
            }),
            EntryPoint::Country {
                two_letter_iso_country_code,
            } => Ok(EntryPointExtV1::Location {
                location: two_letter_iso_country_code.clone(),
            }),
            EntryPoint::Region { .. } => Err(ConfigSetupError::DowngradeEntryPoint),
            EntryPoint::Random => Ok(EntryPointExtV1::Random),
        }
    }
}

//
// ExitPointExtV1
//

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum ExitPointExtV1 {
    Address { address: String },
    Gateway { identity: String },
    Location { location: String },
    Random,
}

impl TryFrom<ExitPointExtV1> for ExitPoint {
    type Error = ConfigSetupError;

    fn try_from(value: ExitPointExtV1) -> Result<Self, Self::Error> {
        match value {
            ExitPointExtV1::Address { address } => {
                let recipient = gateway_directory::Recipient::from_str(&address).map_err(|e| {
                    ConfigSetupError::ExitPoint {
                        error: Box::new(gateway_directory::Error::RecipientFormattingError {
                            address: address.clone(),
                            source: e,
                        }),
                    }
                })?;
                Ok(ExitPoint::Address {
                    address: Box::new(recipient),
                })
            }
            ExitPointExtV1::Gateway { identity } => {
                let node_identity =
                    gateway_directory::NodeIdentity::from_str(&identity).map_err(|e| {
                        ConfigSetupError::ExitPoint {
                            error: Box::new(
                                gateway_directory::Error::NodeIdentityFormattingError {
                                    identity: identity.clone(),
                                    source: e,
                                },
                            ),
                        }
                    })?;
                Ok(ExitPoint::Gateway {
                    identity: node_identity,
                })
            }
            ExitPointExtV1::Location { location } => Ok(ExitPoint::Country {
                two_letter_iso_country_code: location,
            }),
            ExitPointExtV1::Random => Ok(ExitPoint::Random),
        }
    }
}

impl TryFrom<&ExitPoint> for ExitPointExtV1 {
    type Error = ConfigSetupError;

    fn try_from(value: &ExitPoint) -> Result<Self, Self::Error> {
        match value {
            ExitPoint::Address { address } => Ok(ExitPointExtV1::Address {
                address: address.to_string(),
            }),
            ExitPoint::Gateway { identity } => Ok(ExitPointExtV1::Gateway {
                identity: identity.to_string(),
            }),
            ExitPoint::Country {
                two_letter_iso_country_code,
            } => Ok(ExitPointExtV1::Location {
                location: two_letter_iso_country_code.clone(),
            }),
            ExitPoint::Region { .. } => Err(ConfigSetupError::DowngradeExitPoint),
            ExitPoint::Random => Ok(ExitPointExtV1::Random),
        }
    }
}

//
// Legacy TOML version of config file.
//

#[derive(Clone, Debug, Serialize, Deserialize)]
enum LegacyEntryPoint {
    Gateway { identity: NodeIdentity },
    Location { location: String },
    Random,
}

impl TryFrom<LegacyEntryPoint> for EntryPoint {
    type Error = ConfigSetupError;

    fn try_from(value: LegacyEntryPoint) -> Result<Self, Self::Error> {
        match value {
            LegacyEntryPoint::Gateway { identity } => Ok(EntryPoint::Gateway { identity }),
            LegacyEntryPoint::Location { location } => Ok(EntryPoint::Country {
                two_letter_iso_country_code: location,
            }),
            LegacyEntryPoint::Random => Ok(EntryPoint::Random),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
enum LegacyExitPoint {
    Address { address: String },
    Gateway { identity: NodeIdentity },
    Location { location: String },
    Random,
}

impl TryFrom<LegacyExitPoint> for ExitPoint {
    type Error = ConfigSetupError;

    fn try_from(value: LegacyExitPoint) -> Result<Self, Self::Error> {
        match value {
            LegacyExitPoint::Address { address } => {
                let recipient = gateway_directory::Recipient::from_str(&address).map_err(|e| {
                    ConfigSetupError::ExitPoint {
                        error: Box::new(gateway_directory::Error::RecipientFormattingError {
                            address: address.clone(),
                            source: e,
                        }),
                    }
                })?;
                Ok(ExitPoint::Address {
                    address: Box::new(recipient),
                })
            }
            LegacyExitPoint::Gateway { identity } => Ok(ExitPoint::Gateway { identity }),
            LegacyExitPoint::Location { location } => Ok(ExitPoint::Country {
                two_letter_iso_country_code: location,
            }),
            LegacyExitPoint::Random => Ok(ExitPoint::Random),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct LegacyVpnServiceConfig {
    entry_point: LegacyEntryPoint,
    exit_point: LegacyExitPoint,
}

impl TryFrom<LegacyVpnServiceConfig> for VpnServiceConfig {
    type Error = ConfigSetupError;

    fn try_from(value: LegacyVpnServiceConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            entry_point: value.entry_point.try_into()?,
            exit_point: value.exit_point.try_into()?,
            ..Default::default()
        })
    }
}

//
// ConfigSetupError
//

#[derive(thiserror::Error, Debug)]
pub enum ConfigSetupError {
    #[error("failed to serialize JSON config file {file}")]
    SerializeJson {
        file: PathBuf,
        #[source]
        error: Box<serde_json::Error>,
    },

    #[error("failed to parse TOML config file {file}")]
    ParseToml {
        file: PathBuf,
        #[source]
        error: Box<toml::de::Error>,
    },

    #[error("failed to parse JSON config file {file}")]
    ParseJson {
        file: PathBuf,
        #[source]
        error: Box<serde_json::Error>,
    },

    #[error("failed to read config file {file}")]
    ReadConfig {
        file: PathBuf,
        #[source]
        error: std::io::Error,
    },

    #[error("failed to get parent directory of {file}")]
    GetParentDirectory { file: PathBuf },

    #[error("failed to create directory {dir}")]
    CreateDirectory {
        dir: PathBuf,
        #[source]
        error: std::io::Error,
    },

    #[error("failed to write file {file}")]
    WriteFile {
        file: PathBuf,
        error: std::io::Error,
    },

    #[cfg(unix)]
    #[error("failed to set permissions for directory {dir}")]
    SetPermissions {
        dir: PathBuf,
        #[source]
        error: std::io::Error,
    },

    #[cfg(windows)]
    #[error("failed to set permissions for directory {dir}")]
    SetPermissions {
        dir: PathBuf,
        #[source]
        error: nym_windows::security::Error,
    },

    #[error("failed to convert entry point")]
    EntryPoint {
        #[source]
        error: Box<gateway_directory::Error>,
    },

    #[error("failed to convert exit point")]
    ExitPoint {
        #[source]
        error: Box<gateway_directory::Error>,
    },

    #[error("failed to downgrade entry point")]
    DowngradeEntryPoint,

    #[error("failed to downgrade exit point")]
    DowngradeExitPoint,

    #[error("failed to convert IP address")]
    IpAddress {
        #[source]
        error: Box<std::net::AddrParseError>,
    },

    #[error("failed to convert user agent {user_agent}")]
    UserAgent {
        user_agent: String, // Importing UserAgentError seems impossible.
    },
}

#[cfg(windows)]
pub fn program_data_path() -> PathBuf {
    PathBuf::from(std::env::var("ProgramData").unwrap_or(std::env::var("PROGRAMDATA").unwrap()))
}

fn default_data_dir() -> PathBuf {
    #[cfg(windows)]
    return program_data_path().join("nym-vpnd").join("data");

    #[cfg(not(windows))]
    return DEFAULT_DATA_DIR.into();
}

pub fn data_dir() -> PathBuf {
    std::env::var("NYM_VPND_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| default_data_dir())
}

fn default_log_dir() -> PathBuf {
    #[cfg(windows)]
    return program_data_path().join("nym-vpnd").join("log");

    #[cfg(not(windows))]
    return DEFAULT_LOG_DIR.into();
}

pub fn log_dir() -> PathBuf {
    std::env::var("NYM_VPND_LOG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| default_log_dir())
}

pub fn default_config_dir() -> PathBuf {
    #[cfg(windows)]
    return program_data_path().join("nym-vpnd").join("config");

    #[cfg(not(windows))]
    return DEFAULT_CONFIG_DIR.into();
}

pub fn config_dir() -> PathBuf {
    std::env::var("NYM_VPND_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| default_config_dir())
}

pub fn read_toml_config_file<C>(file_path: &Path) -> Result<C, ConfigSetupError>
where
    C: DeserializeOwned,
{
    let file_content =
        fs::read_to_string(file_path).map_err(|error| ConfigSetupError::ReadConfig {
            file: file_path.to_path_buf(),
            error,
        })?;
    toml::from_str(&file_content).map_err(|error| ConfigSetupError::ParseToml {
        file: file_path.to_path_buf(),
        error: Box::new(error),
    })
}

pub fn read_json_config_file<C>(file_path: &Path) -> Result<C, ConfigSetupError>
where
    C: DeserializeOwned,
{
    let file = fs::File::open(file_path).map_err(|error| ConfigSetupError::ReadConfig {
        file: file_path.to_path_buf(),
        error,
    })?;
    let reader = std::io::BufReader::new(file);
    serde_json::from_reader(reader).map_err(|error| ConfigSetupError::ParseJson {
        file: file_path.to_path_buf(),
        error: Box::new(error),
    })
}

pub fn write_json_config_file<C>(file_path: &Path, config: &C) -> Result<(), ConfigSetupError>
where
    C: Serialize,
{
    // Create path
    let config_dir = file_path
        .parent()
        .ok_or_else(|| ConfigSetupError::GetParentDirectory {
            file: file_path.to_path_buf(),
        })?;
    fs::create_dir_all(config_dir).map_err(|error| ConfigSetupError::CreateDirectory {
        dir: config_dir.to_path_buf(),
        error,
    })?;

    let file = fs::File::create(file_path).map_err(|error| ConfigSetupError::WriteFile {
        file: file_path.to_path_buf(),
        error,
    })?;
    let writer = std::io::BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &config).map_err(|error| {
        ConfigSetupError::SerializeJson {
            file: file_path.to_path_buf(),
            error: Box::new(error),
        }
    })?;

    Ok(())
}

pub fn create_data_dir(data_dir: &Path, network_name: &str) -> Result<(), ConfigSetupError> {
    let network_data_dir = data_dir.join(network_name);

    fs::create_dir_all(&network_data_dir).map_err(|error| ConfigSetupError::CreateDirectory {
        dir: network_data_dir.clone(),
        error,
    })?;

    tracing::debug!(
        "Making sure data dir exists at {}",
        network_data_dir.display()
    );

    for dir_path in [&network_data_dir, data_dir] {
        #[cfg(unix)]
        {
            // Set directory permissions to 700 (rwx------)
            let permissions = fs::Permissions::from_mode(0o700);
            fs::set_permissions(dir_path, permissions).map_err(|error| {
                ConfigSetupError::SetPermissions {
                    dir: dir_path.to_path_buf(),
                    error,
                }
            })?;
        }

        #[cfg(windows)]
        {
            set_data_dir_permissions(dir_path).map_err(|error| {
                ConfigSetupError::SetPermissions {
                    dir: dir_path.to_path_buf(),
                    error,
                }
            })?;
        }
    }

    Ok(())
}

/// Set directory permissions to Administrators with Full Control.
#[cfg(windows)]
fn set_data_dir_permissions(data_dir: &Path) -> nym_windows::security::Result<()> {
    use nym_windows::security::{
        AccessMode, AceFlags, Acl, ExplicitAccess, FileAccessRights, SecurityInfo,
        SecurityObjectType, Sid, Trustee, TrusteeType, WellKnownSid, set_named_security_info,
    };

    let administrators_sid = Sid::well_known(WellKnownSid::BuiltinAdministrators)?;

    let allow_admin_group_access = ExplicitAccess::new(
        Trustee::new(administrators_sid.try_clone()?, TrusteeType::WellKnownGroup),
        AccessMode::SetAccess,
        FileAccessRights::FILE_ALL_ACCESS.into(),
        AceFlags::OBJECT_INHERIT_ACE | AceFlags::CONTAINER_INHERIT_ACE,
    );

    let acl = Acl::new(vec![allow_admin_group_access])?;

    set_named_security_info(
        data_dir,
        SecurityObjectType::FileObject,
        SecurityInfo::DACL | SecurityInfo::PROTECTED_DACL,
        None,
        None,
        Some(&acl),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    // Test migrating from TOML to the latest JSON version
    fn run_migrate_toml_test(
        toml_content: &str,
        json_latest_content: &str,
        entry_point: gateway_directory::EntryPoint,
        exit_point: gateway_directory::ExitPoint,
    ) {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path();

        println!("Using config dir: {config_path:?}");

        let network_config_path = config_path.join("tulips");
        let _ = fs::create_dir_all(&network_config_path);
        let toml_path = network_config_path.join(DEFAULT_CONFIG_FILE_TOML);
        let json_path = network_config_path.join(DEFAULT_CONFIG_FILE_JSON);

        // Write the TOML config file
        fs::write(&toml_path, toml_content).unwrap();

        // Read the TOML config and migrate it to latest JSON
        let config_manager = VpnServiceConfigManager::new(&network_config_path).unwrap();
        let config = config_manager.config();
        assert_eq!(config.entry_point, entry_point);
        assert_eq!(config.exit_point, exit_point);

        assert!(config_manager.write_to_file());

        // The TOML file should be deleted and replaced with a JSON version
        assert!(!toml_path.exists());
        assert!(json_path.exists());

        // Read the JSON config
        let config_manager = VpnServiceConfigManager::new(&network_config_path).unwrap();
        let config = config_manager.config();
        assert_eq!(config.entry_point, entry_point);
        assert_eq!(config.exit_point, exit_point);

        // Check the JSON is the right version and all snake-case
        let read_json_content = fs::read_to_string(&json_path).unwrap();
        assert_eq!(json_latest_content, read_json_content);
    }

    // Test migrating from JSON v1 to the latest JSON version
    fn run_migrate_json_v1_test(json_v1_content: &str, json_latest_content: &str) {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path();

        println!("Using config dir: {config_path:?}");

        let network_config_path = config_path.join("tulips");
        let _ = fs::create_dir_all(&network_config_path);
        let json_path = network_config_path.join(DEFAULT_CONFIG_FILE_JSON);

        // Write the JSON v1 config file
        fs::write(&json_path, json_v1_content).unwrap();

        // Read the JSON v1 config and migrate it to latest JSON.  The latest version of the
        // JSON will be written straight back to disk.
        let _config_manager = VpnServiceConfigManager::new(&network_config_path).unwrap();

        // Check the JSON is the right version and all snake-case (ignore whitespace/order)
        let read_json_content = fs::read_to_string(&json_path).unwrap();
        let expected: serde_json::Value = serde_json::from_str(json_latest_content).unwrap();
        let actual: serde_json::Value = serde_json::from_str(&read_json_content).unwrap();
        assert_eq!(expected, actual);
    }

    fn run_serialize_test(config: VpnServiceConfig) {
        let temp_dir = tempdir().unwrap();
        let network_config_path = temp_dir.path();

        // Write the config to disk
        let mut config_manager = VpnServiceConfigManager::new(network_config_path).unwrap();
        config_manager.set_config(config.clone());
        assert!(config_manager.write_to_file());
        drop(config_manager);

        // Read it back and compare it
        let config_manager = VpnServiceConfigManager::new(network_config_path).unwrap();
        let read_config = config_manager.config();
        assert_eq!(&config, read_config);
    }

    #[test]
    fn test_service_config_migrate_toml_location() {
        let toml_content = r#"
[entry_point.Location]
location = "FR"

[exit_point.Location]
location = "BE"
"#;

        let json_content = r#"{
  "version": "v2",
  "entry_point": {
    "location": {
      "location": "FR"
    }
  },
  "exit_point": {
    "location": {
      "location": "BE"
    }
  },
  "dns": null,
  "disable_ipv6": false,
  "enable_two_hop": false,
  "netstack": false,
  "disable_poisson_rate": false,
  "disable_background_cover_traffic": false,
  "min_mixnode_performance": null,
  "min_gateway_mixnet_performance": null,
  "min_gateway_vpn_performance": null
}"#;

        let entry_point = gateway_directory::EntryPoint::Country {
            two_letter_iso_country_code: "FR".to_string(),
        };

        let exit_point = gateway_directory::ExitPoint::Country {
            two_letter_iso_country_code: "BE".to_string(),
        };

        run_migrate_toml_test(toml_content, json_content, entry_point, exit_point);
    }

    #[test]
    fn test_service_config_migrate_gateway() {
        let toml_content = r#"
[entry_point.Gateway]
identity = [ 92, 25, 33, 77, 4, 117, 82, 117, 246, 239, 233, 11, 129, 183, 86, 194, 140, 95, 21, 196, 121, 130, 232, 195, 71, 173, 66, 124, 5, 14, 114, 107, ]

[exit_point.Gateway]
identity = [ 99, 23, 98, 234, 66, 161, 195, 63, 155, 161, 250, 207, 17, 158, 136, 114, 215, 90, 236, 161, 231, 176, 140, 190, 147, 182, 64, 171, 145, 31, 245, 186, ]
"#;

        let json_content = r#"{
  "version": "v2",
  "entry_point": {
    "gateway": {
      "identity": "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42"
    }
  },
  "exit_point": {
    "gateway": {
      "identity": "7fp3cmzCvgeRgbB1ycTnK6RokjHNqPmCCSBG23gyxshj"
    }
  },
  "dns": null,
  "disable_ipv6": false,
  "enable_two_hop": false,
  "netstack": false,
  "disable_poisson_rate": false,
  "disable_background_cover_traffic": false,
  "min_mixnode_performance": null,
  "min_gateway_mixnet_performance": null,
  "min_gateway_vpn_performance": null
}"#;

        let entry_point = gateway_directory::EntryPoint::Gateway {
            identity: gateway_directory::NodeIdentity::from_str(
                "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42",
            )
            .unwrap(),
        };

        let exit_point = gateway_directory::ExitPoint::Gateway {
            identity: gateway_directory::NodeIdentity::from_str(
                "7fp3cmzCvgeRgbB1ycTnK6RokjHNqPmCCSBG23gyxshj",
            )
            .unwrap(),
        };

        run_migrate_toml_test(toml_content, json_content, entry_point, exit_point);
    }

    #[test]
    #[ignore] // Temporarily disabled due to issues with ExitPoint::Address (de)serialisation.
    fn test_service_config_migrate_toml_address() {
        let toml_content = r#"
[entry_point.Gateway]
identity = [92, 25, 33, 77, 4, 117, 82, 117, 246, 239, 233, 11, 129, 183, 86, 194, 140, 95, 21, 196, 121, 130, 232, 195, 71, 173, 66, 124, 5, 14, 114, 107]

[exit_point.Address]
address = [5, 56, 84, 195, 94, 238, 210, 124, 65, 143, 209, 144, 22, 255, 91, 188, 35, 50, 144, 234, 226, 114, 99, 40, 10, 102, 200, 170, 19, 162, 86, 134, 84, 20, 195, 193, 42, 194, 230, 153, 163, 90, 214, 216, 196, 166, 87, 132, 206, 215, 91, 89, 51, 98, 72, 156, 159, 248, 109, 225, 152, 204, 80, 97, 9, 62, 22, 108, 155, 95, 153, 29, 143, 48, 208, 5, 101, 231, 176, 93, 107, 229, 11, 225, 145, 1, 14, 219, 44, 88, 199, 206, 40, 185, 150, 151]
"#;

        let json_content = r#"{
  "version": "v2",
  "entry_point": {
    "gateway": {
      "identity": "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42"
    }
  },
  "exit_point": {
    "address": {
            "address": "MNrmKzuKjNdbEhfPUzVNfjw63oBQNSayqoQKGL4JjAV.6fDcSN6faGpvA3pd3riCwjpzXc7RQfWmGMa82UVoEwKE@d5adfJNtcdZW2XwK85JAAU8nXAs9JCPYn2RNvDLZn4e"
    }
  },
  "dns": null,
  "disable_ipv6": false,
  "enable_two_hop": false,
  "netstack": false,
  "disable_poisson_rate": false,
  "disable_background_cover_traffic": false,
  "min_mixnode_performance": null,
  "min_gateway_mixnet_performance": null,
  "min_gateway_vpn_performance": null
}"#;

        let entry_point = gateway_directory::EntryPoint::Gateway {
            identity: gateway_directory::NodeIdentity::from_str(
                "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42",
            )
            .unwrap(),
        };

        let exit_point = gateway_directory::ExitPoint::Address {
            address: Box::new(
                gateway_directory::Recipient::from_str("MNrmKzuKjNdbEhfPUzVNfjw63oBQNSayqoQKGL4JjAV.6fDcSN6faGpvA3pd3riCwjpzXc7RQfWmGMa82UVoEwKE@d5adfJNtcdZW2XwK85JAAU8nXAs9JCPYn2RNvDLZn4e").unwrap(),
            )
        };

        run_migrate_toml_test(toml_content, json_content, entry_point, exit_point);
    }

    #[test]
    fn test_service_config_migrate_toml_random() {
        let toml_content = r#"
entry_point = "Random"
exit_point = "Random"
"#;

        let json_content = r#"{
  "version": "v2",
  "entry_point": "random",
  "exit_point": "random",
  "dns": null,
  "disable_ipv6": false,
  "enable_two_hop": false,
  "netstack": false,
  "disable_poisson_rate": false,
  "disable_background_cover_traffic": false,
  "min_mixnode_performance": null,
  "min_gateway_mixnet_performance": null,
  "min_gateway_vpn_performance": null
}"#;

        let entry_point = gateway_directory::EntryPoint::Random;

        let exit_point = gateway_directory::ExitPoint::Random;

        run_migrate_toml_test(toml_content, json_content, entry_point, exit_point);
    }

    #[test]
    fn test_service_config_migrate_from_v1() {
        let json_v1_content = r#"{
  "version": "v1",
  "entry_point": {
    "gateway": {
      "identity": "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42"
    }
  },
  "exit_point": {
    "address": {
      "address": "MNrmKzuKjNdbEhfPUzVNfjw63oBQNSayqoQKGL4JjAV.6fDcSN6faGpvA3pd3riCwjpzXc7RQfWmGMa82UVoEwKE@d5adfJNtcdZW2XwK85JAAU8nXAs9JCPYn2RNvDLZn4e"
    }
  }
}"#;

        let json_latest_content = r#"{
  "version": "v2",
  "entry_point": {
    "gateway": {
      "identity": "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42"
    }
  },
  "exit_point": {
    "address": {
      "address": "MNrmKzuKjNdbEhfPUzVNfjw63oBQNSayqoQKGL4JjAV.6fDcSN6faGpvA3pd3riCwjpzXc7RQfWmGMa82UVoEwKE@d5adfJNtcdZW2XwK85JAAU8nXAs9JCPYn2RNvDLZn4e"
    }
  },
  "dns": null,
  "disable_ipv6": false,
  "enable_two_hop": false,
  "netstack": false,
  "disable_poisson_rate": false,
  "disable_background_cover_traffic": false,
  "min_mixnode_performance": null,
  "min_gateway_mixnet_performance": null,
  "min_gateway_vpn_performance": null
}"#;

        run_migrate_json_v1_test(json_v1_content, json_latest_content);
    }

    #[test]
    fn test_service_config_serialize_defaults() {
        let config = VpnServiceConfig::default();
        run_serialize_test(config);
    }

    #[test]
    fn test_service_config_serialize_full() {
        let config = VpnServiceConfig {
            entry_point: gateway_directory::EntryPoint::Country {
                two_letter_iso_country_code: "US".to_string(),
            },
            exit_point: gateway_directory::ExitPoint::Gateway {
                identity: gateway_directory::NodeIdentity::from_str(
                    "7fp3cmzCvgeRgbB1ycTnK6RokjHNqPmCCSBG23gyxshj",
                )
                .unwrap(),
            },
            dns: Some(IpAddr::from_str("192.168.50.1").unwrap()),
            disable_ipv6: true,
            enable_two_hop: true,
            netstack: true,
            disable_poisson_rate: true,
            disable_background_cover_traffic: true,
            min_mixnode_performance: Some(55u8),
            min_gateway_mixnet_performance: Some(64u8),
            min_gateway_vpn_performance: Some(1u8),
        };
        run_serialize_test(config);
    }
}
