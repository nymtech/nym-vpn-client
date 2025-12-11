// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod config_manager;
mod entry_exit;
mod legacy;
mod network_stats;
mod v1;
mod v2;
mod v3;
mod v4;

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
    entry_exit::v2::{EntryPoint, ExitPoint},
    network_stats::v1::NetworkStatisticsConfig,
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
    Evil,
}

impl fmt::Display for NetworkEnvironments {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkEnvironments::Mainnet => write!(f, "mainnet"),
            NetworkEnvironments::Sandbox => write!(f, "sandbox"),
            NetworkEnvironments::Canary => write!(f, "canary"),
            NetworkEnvironments::Evil => write!(f, "evil"),
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
            "evil" => Ok(NetworkEnvironments::Evil),
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
}

impl VpnServiceConfigVersion {
    /// Returns the latest version of the config file.
    pub fn latest() -> Self {
        VpnServiceConfigVersion::V4
    }
}

impl fmt::Display for VpnServiceConfigVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            VpnServiceConfigVersion::V1 => "v1",
            VpnServiceConfigVersion::V2 => "v2",
            VpnServiceConfigVersion::V3 => "v3",
            VpnServiceConfigVersion::V4 => "v4",
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
}

impl VpnServiceConfigExt {
    fn version(&self) -> VpnServiceConfigVersion {
        match self {
            VpnServiceConfigExt::V1(_) => VpnServiceConfigVersion::V1,
            VpnServiceConfigExt::V2(_) => VpnServiceConfigVersion::V2,
            VpnServiceConfigExt::V3(_) => VpnServiceConfigVersion::V3,
            VpnServiceConfigExt::V4(_) => VpnServiceConfigVersion::V4,
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
        }
    }
}

impl TryFrom<&nym_vpn_lib_types::VpnServiceConfig> for VpnServiceConfigExt {
    type Error = ConfigSetupError;

    fn try_from(value: &nym_vpn_lib_types::VpnServiceConfig) -> Result<Self, Self::Error> {
        let custom_dns = value
            .custom_dns
            .iter()
            .map(|ip| ip.to_string())
            .collect::<Vec<_>>();

        let v4 = v4::VpnServiceConfig {
            entry_point: EntryPoint::try_from(&value.entry_point)?,
            exit_point: ExitPoint::try_from(&value.exit_point)?,
            allow_lan: value.allow_lan,
            disable_ipv6: value.disable_ipv6,
            enable_two_hop: value.enable_two_hop,
            enable_bridges: value.enable_bridges,
            netstack: value.netstack,
            disable_poisson_rate: value.mixnet_traffic.disable_poisson_rate,
            disable_background_cover_traffic: value.mixnet_traffic.disable_background_cover_traffic,
            min_mixnode_performance: value.mixnet_traffic.min_mixnode_performance,
            min_gateway_mixnet_performance: value.mixnet_traffic.min_gateway_mixnet_performance,
            min_gateway_vpn_performance: value.min_gateway_vpn_performance,
            residential_exit: value.residential_exit,
            enable_custom_dns: value.enable_custom_dns,
            custom_dns,
            network_stats: NetworkStatisticsConfig::from(&value.network_stats),
        };
        Ok(VpnServiceConfigExt::V4(v4))
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

pub async fn create_data_dir(data_dir: &Path, network_name: &str) -> Result<(), ConfigSetupError> {
    let network_data_dir = data_dir.join(network_name);

    fs::create_dir_all(&network_data_dir)
        .await
        .map_err(|error| ConfigSetupError::CreateDirectory {
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
            let permissions = std::fs::Permissions::from_mode(0o700);
            fs::set_permissions(dir_path, permissions)
                .await
                .map_err(|error| ConfigSetupError::SetPermissions {
                    dir: dir_path.to_path_buf(),
                    error,
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
