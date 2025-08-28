// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::error::{Error, Result};
use nym_vpn_lib::{NodeIdentity, Recipient, gateway_directory};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[cfg(windows)]
use std::path::Path;
use std::{fmt, fs, path::PathBuf};

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
// NymVpnServiceConfig
//

// Derserialize is only for reading the deprecated TOML version.
#[derive(Clone, Debug, Deserialize)]
pub struct NymVpnServiceConfig {
    pub(super) entry_point: gateway_directory::EntryPoint,
    pub(super) exit_point: gateway_directory::ExitPoint,
}

impl NymVpnServiceConfig {
    pub(super) fn write_to_file(&self, config_path: &Path) -> Result<()> {
        let ext_config = NymVpnServiceConfigExt::try_from(self).map_err(Error::ConfigSetup)?;
        write_json_config_file(config_path, &ext_config).map_err(Error::ConfigSetup)
    }
}

impl fmt::Display for NymVpnServiceConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "entry point: {}, exit point: {}",
            self.entry_point, self.exit_point
        )
    }
}

impl Default for NymVpnServiceConfig {
    fn default() -> Self {
        Self {
            entry_point: gateway_directory::EntryPoint::Random,
            exit_point: gateway_directory::ExitPoint::Random,
        }
    }
}

//
// External, versioned, representation of the vpn service config file.
//

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "version")]
#[serde(rename_all = "snake_case")]
pub(super) enum NymVpnServiceConfigExt {
    V1(NymVpnServiceConfigExtV1),
}

impl TryFrom<NymVpnServiceConfigExt> for NymVpnServiceConfig {
    type Error = ConfigSetupError;

    fn try_from(value: NymVpnServiceConfigExt) -> Result<Self, Self::Error> {
        match value {
            NymVpnServiceConfigExt::V1(v1) => NymVpnServiceConfig::try_from(v1),
        }
    }
}

impl TryFrom<&NymVpnServiceConfig> for NymVpnServiceConfigExt {
    type Error = ConfigSetupError;

    fn try_from(value: &NymVpnServiceConfig) -> Result<Self, Self::Error> {
        //
        // This is the version of the configuration that will be written to disk.
        //
        let v1 = NymVpnServiceConfigExtV1 {
            entry_point: EntryPointExtV1::try_from(&value.entry_point)?,
            exit_point: ExitPointExtV1::try_from(&value.exit_point)?,
        };
        Ok(NymVpnServiceConfigExt::V1(v1))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct NymVpnServiceConfigExtV1 {
    entry_point: EntryPointExtV1,
    exit_point: ExitPointExtV1,
}

impl TryFrom<NymVpnServiceConfigExtV1> for NymVpnServiceConfig {
    type Error = ConfigSetupError;

    fn try_from(value: NymVpnServiceConfigExtV1) -> Result<Self, Self::Error> {
        let entry_point = gateway_directory::EntryPoint::try_from(value.entry_point)?;
        let exit_point = gateway_directory::ExitPoint::try_from(value.exit_point)?;
        Ok(NymVpnServiceConfig {
            entry_point,
            exit_point,
        })
    }
}

impl TryFrom<&NymVpnServiceConfig> for NymVpnServiceConfigExtV1 {
    type Error = ConfigSetupError;

    fn try_from(value: &NymVpnServiceConfig) -> Result<Self, Self::Error> {
        let entry_point = EntryPointExtV1::try_from(&value.entry_point)?;
        let exit_point = ExitPointExtV1::try_from(&value.exit_point)?;
        Ok(NymVpnServiceConfigExtV1 {
            entry_point,
            exit_point,
        })
    }
}

//
// EntryPointExtV1
//

#[derive(Clone, Debug, Serialize, Deserialize)]
enum EntryPointExtV1 {
    // An explicit entry gateway identity.
    Gateway { identity: NodeIdentity },
    // Select a random entry gateway in a specific location.
    Location { location: String },
    // Select an entry gateway at random.
    Random,
}

impl TryFrom<EntryPointExtV1> for gateway_directory::EntryPoint {
    type Error = ConfigSetupError;

    fn try_from(value: EntryPointExtV1) -> Result<Self, Self::Error> {
        match value {
            EntryPointExtV1::Gateway { identity } => {
                Ok(gateway_directory::EntryPoint::Gateway { identity })
            }
            EntryPointExtV1::Location { location } => {
                Ok(gateway_directory::EntryPoint::Location { location })
            }
            EntryPointExtV1::Random => Ok(gateway_directory::EntryPoint::Random),
        }
    }
}

impl TryFrom<&gateway_directory::EntryPoint> for EntryPointExtV1 {
    type Error = ConfigSetupError;

    fn try_from(value: &gateway_directory::EntryPoint) -> Result<Self, Self::Error> {
        match value {
            gateway_directory::EntryPoint::Gateway { identity } => Ok(EntryPointExtV1::Gateway {
                identity: *identity,
            }),
            gateway_directory::EntryPoint::Location { location } => Ok(EntryPointExtV1::Location {
                location: location.clone(),
            }),
            gateway_directory::EntryPoint::Random => Ok(EntryPointExtV1::Random),
        }
    }
}

//
// ExitPointExtV1
//

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
enum ExitPointExtV1 {
    // An explicit exit address. This is useful when the exit ip-packet-router is running as a
    // standalone entity (private).
    Address { address: Box<Recipient> },

    // An explicit exit gateway identity. This is useful when the exit ip-packet-router is running
    // embedded on a gateway.
    Gateway { identity: NodeIdentity },

    // NOTE: Consider using a crate with strongly typed country codes instead of strings
    Location { location: String },

    // Select an exit gateway at random.
    Random,
}

impl TryFrom<ExitPointExtV1> for gateway_directory::ExitPoint {
    type Error = ConfigSetupError;

    fn try_from(value: ExitPointExtV1) -> Result<Self, Self::Error> {
        match value {
            ExitPointExtV1::Address { address } => {
                Ok(gateway_directory::ExitPoint::Address { address })
            }
            ExitPointExtV1::Gateway { identity } => {
                Ok(gateway_directory::ExitPoint::Gateway { identity })
            }
            ExitPointExtV1::Location { location } => {
                Ok(gateway_directory::ExitPoint::Location { location })
            }
            ExitPointExtV1::Random => Ok(gateway_directory::ExitPoint::Random),
        }
    }
}

impl TryFrom<&gateway_directory::ExitPoint> for ExitPointExtV1 {
    type Error = ConfigSetupError;

    fn try_from(value: &gateway_directory::ExitPoint) -> Result<Self, Self::Error> {
        match value {
            gateway_directory::ExitPoint::Address { address } => Ok(ExitPointExtV1::Address {
                address: address.clone(),
            }),
            gateway_directory::ExitPoint::Gateway { identity } => Ok(ExitPointExtV1::Gateway {
                identity: *identity,
            }),
            gateway_directory::ExitPoint::Location { location } => Ok(ExitPointExtV1::Location {
                location: location.clone(),
            }),
            gateway_directory::ExitPoint::Random => Ok(ExitPointExtV1::Random),
        }
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
    let file_content =
        fs::read_to_string(file_path).map_err(|error| ConfigSetupError::ReadConfig {
            file: file_path.to_path_buf(),
            error,
        })?;
    serde_json::from_str(&file_content).map_err(|error| ConfigSetupError::ParseJson {
        file: file_path.to_path_buf(),
        error: Box::new(error),
    })
}

pub fn write_json_config_file<C>(file_path: &Path, config: &C) -> Result<(), ConfigSetupError>
where
    C: Serialize,
{
    let config_str =
        serde_json::to_string_pretty(&config).map_err(|error| ConfigSetupError::SerializeJson {
            file: file_path.to_path_buf(),
            error: Box::new(error),
        })?;

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

    fs::write(file_path, config_str).map_err(|error| ConfigSetupError::WriteFile {
        file: file_path.to_path_buf(),
        error,
    })?;
    tracing::info!("Wrote config file {:?}", file_path);
    Ok(())
}

pub(super) fn create_data_dir(data_dir: &Path, network_name: &str) -> Result<(), ConfigSetupError> {
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
                    dir: dir_path.clone(),
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

pub fn default_true() -> bool {
    true
}
