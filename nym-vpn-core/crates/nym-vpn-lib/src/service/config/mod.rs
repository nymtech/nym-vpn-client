// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod circumvention;
mod config_manager;
mod entry_exit;
mod gateway_independence;
mod gateway_selection_algorithm;
mod geo_exclusion_settings;
mod legacy;
mod mixnet_traffic;
mod network_stats;
mod split_tunnel_settings;
mod v1;
mod v10;
mod v11;
mod v12;
mod v2;
mod v3;
mod v4;
mod v5;
mod v6;
mod v7;
mod v8;
mod v9;

#[cfg(test)]
mod tests;

pub use config_manager::VpnServiceConfigManager;

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    fmt,
    path::{Path, PathBuf},
};
use tokio::{
    fs,
    io::{self, AsyncWriteExt},
};

use crate::service::config::{
    circumvention::v9::FrontingMode,
    entry_exit::v3::{EntryPoint, ExitPoint},
    gateway_independence::v11::GatewayIndependence,
    gateway_selection_algorithm::v12::GatewaySelectionAlgorithmConfig,
    geo_exclusion_settings::v9::GeoExclusionSettings,
    mixnet_traffic::v5::MixnetTrafficConfig,
    network_stats::v1::NetworkStatisticsConfig,
    split_tunnel_settings::v8::SplitTunnelSettings,
};

pub const DEFAULT_CONFIG_FILE_TOML: &str = "nym-vpnd.toml";
pub const DEFAULT_CONFIG_FILE_JSON: &str = "nym-vpnd.json";

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
}

impl fmt::Display for NetworkEnvironments {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkEnvironments::Mainnet => write!(f, "mainnet"),
            NetworkEnvironments::Sandbox => write!(f, "sandbox"),
            NetworkEnvironments::Canary => write!(f, "canary"),
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
            _ => Err("Invalid network environment"),
        }
    }
}

//
// External, versioned, representation of the vpn service config file.
//

/// Represents the version of the vpn service config file.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum VpnServiceConfigVersion {
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
    V8,
    V9,
    V10,
    V11,
    V12,
}

impl VpnServiceConfigVersion {
    /// Returns the latest version of the config file.
    pub fn latest() -> Self {
        VpnServiceConfigVersion::V12
    }
}

impl fmt::Display for VpnServiceConfigVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            VpnServiceConfigVersion::V1 => "v1",
            VpnServiceConfigVersion::V2 => "v2",
            VpnServiceConfigVersion::V3 => "v3",
            VpnServiceConfigVersion::V4 => "v4",
            VpnServiceConfigVersion::V5 => "v5",
            VpnServiceConfigVersion::V6 => "v6",
            VpnServiceConfigVersion::V7 => "v7",
            VpnServiceConfigVersion::V8 => "v8",
            VpnServiceConfigVersion::V9 => "v9",
            VpnServiceConfigVersion::V10 => "v10",
            VpnServiceConfigVersion::V11 => "v11",
            VpnServiceConfigVersion::V12 => "v12",
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "version")]
#[serde(rename_all = "snake_case")]
enum VpnServiceConfigExt {
    V1(v1::VpnServiceConfig),
    V2(v2::VpnServiceConfig),
    V3(v3::VpnServiceConfig),
    V4(v4::VpnServiceConfig),
    V5(v5::VpnServiceConfig),
    V6(v6::VpnServiceConfig),
    V7(v7::VpnServiceConfig),
    V8(v8::VpnServiceConfig),
    V9(v9::VpnServiceConfig),
    V10(v10::VpnServiceConfig),
    V11(v11::VpnServiceConfig),
    V12(v12::VpnServiceConfig),
}

impl VpnServiceConfigExt {
    fn version(&self) -> VpnServiceConfigVersion {
        match self {
            VpnServiceConfigExt::V1(_) => VpnServiceConfigVersion::V1,
            VpnServiceConfigExt::V2(_) => VpnServiceConfigVersion::V2,
            VpnServiceConfigExt::V3(_) => VpnServiceConfigVersion::V3,
            VpnServiceConfigExt::V4(_) => VpnServiceConfigVersion::V4,
            VpnServiceConfigExt::V5(_) => VpnServiceConfigVersion::V5,
            VpnServiceConfigExt::V6(_) => VpnServiceConfigVersion::V6,
            VpnServiceConfigExt::V7(_) => VpnServiceConfigVersion::V7,
            VpnServiceConfigExt::V8(_) => VpnServiceConfigVersion::V8,
            VpnServiceConfigExt::V9(_) => VpnServiceConfigVersion::V9,
            VpnServiceConfigExt::V10(_) => VpnServiceConfigVersion::V10,
            VpnServiceConfigExt::V11(_) => VpnServiceConfigVersion::V11,
            VpnServiceConfigExt::V12(_) => VpnServiceConfigVersion::V12,
        }
    }
}

impl TryFrom<VpnServiceConfigExt> for nym_vpn_lib_types::VpnServiceConfig {
    type Error = ConfigSetupError;

    fn try_from(value: VpnServiceConfigExt) -> Result<Self, Self::Error> {
        match value {
            VpnServiceConfigExt::V1(v1) => nym_vpn_lib_types::VpnServiceConfig::try_from(v1),
            VpnServiceConfigExt::V2(v2) => nym_vpn_lib_types::VpnServiceConfig::try_from(v2),
            VpnServiceConfigExt::V3(v3) => nym_vpn_lib_types::VpnServiceConfig::try_from(v3),
            VpnServiceConfigExt::V4(v4) => nym_vpn_lib_types::VpnServiceConfig::try_from(v4),
            VpnServiceConfigExt::V5(v5) => nym_vpn_lib_types::VpnServiceConfig::try_from(v5),
            VpnServiceConfigExt::V6(v6) => nym_vpn_lib_types::VpnServiceConfig::try_from(v6),
            VpnServiceConfigExt::V7(v7) => nym_vpn_lib_types::VpnServiceConfig::try_from(v7),
            VpnServiceConfigExt::V8(v8) => nym_vpn_lib_types::VpnServiceConfig::try_from(v8),
            VpnServiceConfigExt::V9(v9) => nym_vpn_lib_types::VpnServiceConfig::try_from(v9),
            VpnServiceConfigExt::V10(v10) => nym_vpn_lib_types::VpnServiceConfig::try_from(v10),
            VpnServiceConfigExt::V11(v11) => nym_vpn_lib_types::VpnServiceConfig::try_from(v11),
            VpnServiceConfigExt::V12(v12) => nym_vpn_lib_types::VpnServiceConfig::try_from(v12),
        }
    }
}

impl TryFrom<&nym_vpn_lib_types::VpnServiceConfig> for VpnServiceConfigExt {
    type Error = ConfigSetupError;

    fn try_from(value: &nym_vpn_lib_types::VpnServiceConfig) -> Result<Self, Self::Error> {
        let entry_point = EntryPoint::try_from(&value.entry_point)?;

        let exit_point = ExitPoint::try_from(&value.exit_point)?;

        let custom_dns = value
            .custom_dns
            .iter()
            .map(|ip| ip.to_string())
            .collect::<Vec<_>>();

        let mixnet_traffic = MixnetTrafficConfig::from(&value.mixnet_traffic);

        let network_stats = NetworkStatisticsConfig::from(&value.network_stats);

        let split_tunnel = SplitTunnelSettings::from(&value.split_tunnel);
        let geo_exclusion = GeoExclusionSettings::from(&value.geo_exclusion);

        let gateway_selection_algorithm_config =
            GatewaySelectionAlgorithmConfig::from(&value.gateway_selection_algorithm_config);

        let fronting_mode = FrontingMode::from(&value.fronting_mode);

        let gateway_independence = GatewayIndependence::from(&value.gateway_independence);

        let v12 = v12::VpnServiceConfig {
            entry_point,
            exit_point,
            allow_lan: value.allow_lan,
            disable_ipv6: value.disable_ipv6,
            enable_two_hop: value.enable_two_hop,
            enable_bridges: value.enable_bridges,
            enable_ad_blocking: value.enable_ad_blocking,
            fronting_mode,
            netstack: value.netstack,
            min_gateway_vpn_performance: value.min_gateway_vpn_performance,
            residential_exit: value.residential_exit,
            enable_custom_dns: value.enable_custom_dns,
            custom_dns,
            mixnet_traffic,
            network_stats,
            split_tunnel,
            geo_exclusion,
            gateway_selection_algorithm_config,
            gateway_independence,
        };

        Ok(VpnServiceConfigExt::V12(v12))
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
        error: io::Error,
    },

    #[error("failed to get parent directory of {file}")]
    GetParentDirectory { file: PathBuf },

    #[error("failed to create directory {dir}")]
    CreateDirectory {
        dir: PathBuf,
        #[source]
        error: io::Error,
    },

    #[error("failed to write file {file}")]
    WriteFile { file: PathBuf, error: io::Error },

    #[cfg(unix)]
    #[error("failed to set permissions for directory {dir}")]
    SetPermissions {
        dir: PathBuf,
        #[source]
        error: io::Error,
    },

    #[cfg(windows)]
    #[error("failed to set permissions for directory {dir}")]
    SetPermissions {
        dir: PathBuf,
        #[source]
        error: nym_windows::security::Error,
    },

    #[error("failed to convert entry point: {0}")]
    EntryPoint(String),

    #[error("failed to convert exit point: {0}")]
    ExitPoint(String),

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

pub async fn read_toml_config_file<C>(file_path: &Path) -> Result<C, ConfigSetupError>
where
    C: DeserializeOwned,
{
    let file_content =
        fs::read_to_string(file_path)
            .await
            .map_err(|error| ConfigSetupError::ReadConfig {
                file: file_path.to_path_buf(),
                error,
            })?;
    toml::from_str(&file_content).map_err(|error| ConfigSetupError::ParseToml {
        file: file_path.to_path_buf(),
        error: Box::new(error),
    })
}

pub async fn read_json_config_file<C>(file_path: &Path) -> Result<C, ConfigSetupError>
where
    C: DeserializeOwned,
{
    let bytes = tokio::fs::read(file_path)
        .await
        .map_err(|error| ConfigSetupError::ReadConfig {
            file: file_path.to_path_buf(),
            error,
        })?;

    serde_json::from_slice(&bytes).map_err(|error| ConfigSetupError::ParseJson {
        file: file_path.to_path_buf(),
        error: Box::new(error),
    })
}

pub async fn write_json_config_file<C>(file_path: &Path, config: &C) -> Result<(), ConfigSetupError>
where
    C: Serialize,
{
    let json_bytes =
        serde_json::to_vec_pretty(&config).map_err(|error| ConfigSetupError::SerializeJson {
            file: file_path.to_path_buf(),
            error: Box::new(error),
        })?;

    // Ensure parent directory exists
    let config_dir = file_path
        .parent()
        .ok_or_else(|| ConfigSetupError::GetParentDirectory {
            file: file_path.to_path_buf(),
        })?;

    fs::create_dir_all(config_dir)
        .await
        .map_err(|error| ConfigSetupError::CreateDirectory {
            dir: config_dir.to_path_buf(),
            error,
        })?;

    let file = fs::File::create(file_path)
        .await
        .map_err(|error| ConfigSetupError::WriteFile {
            file: file_path.to_path_buf(),
            error,
        })?;

    let mut writer = io::BufWriter::new(file);

    writer
        .write_all(&json_bytes)
        .await
        .map_err(|error| ConfigSetupError::WriteFile {
            file: file_path.to_path_buf(),
            error,
        })?;
    writer
        .flush() // This is important!
        .await
        .map_err(|error| ConfigSetupError::WriteFile {
            file: file_path.to_path_buf(),
            error,
        })
}
