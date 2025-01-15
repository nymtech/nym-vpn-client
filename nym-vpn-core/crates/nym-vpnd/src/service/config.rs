// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{fmt, fs, path::PathBuf};

use nym_vpn_lib::gateway_directory;
use serde::{de::DeserializeOwned, Serialize};

#[cfg(not(windows))]
const DEFAULT_DATA_DIR: &str = "/var/lib/nym-vpnd";
#[cfg(not(windows))]
const DEFAULT_LOG_DIR: &str = "/var/log/nym-vpnd";
#[cfg(not(windows))]
const DEFAULT_CONFIG_DIR: &str = "/etc/nym";
pub(crate) const DEFAULT_CONFIG_FILE: &str = "nym-vpnd.toml";
pub(crate) const DEFAULT_LOG_FILE: &str = "nym-vpnd.log";

pub(crate) const DEFAULT_GLOBAL_CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Clone)]
pub(crate) enum NetworkEnvironments {
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

#[cfg(windows)]
pub(crate) fn program_data_path() -> PathBuf {
    PathBuf::from(std::env::var("ProgramData").unwrap_or(std::env::var("PROGRAMDATA").unwrap()))
}

fn default_data_dir() -> PathBuf {
    #[cfg(windows)]
    return program_data_path().join("nym-vpnd").join("data");

    #[cfg(not(windows))]
    return DEFAULT_DATA_DIR.into();
}

pub(crate) fn data_dir() -> PathBuf {
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

pub(crate) fn log_dir() -> PathBuf {
    std::env::var("NYM_VPND_LOG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| default_log_dir())
}

pub(crate) fn default_config_dir() -> PathBuf {
    #[cfg(windows)]
    return program_data_path().join("nym-vpnd").join("config");

    #[cfg(not(windows))]
    return DEFAULT_CONFIG_DIR.into();
}

pub(crate) fn config_dir() -> PathBuf {
    std::env::var("NYM_VPND_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| default_config_dir())
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigSetupError {
    #[error("failed to parse config file {file}: {error}")]
    Parse {
        file: PathBuf,
        error: Box<toml::de::Error>,
    },

    #[error("failed to read config file {file}: {error}")]
    ReadConfig {
        file: PathBuf,
        error: std::io::Error,
    },

    #[error("failed to get parent directory of {file}")]
    GetParentDirectory { file: PathBuf },

    #[error("failed to create directory {dir}: {error}")]
    CreateDirectory { dir: PathBuf, error: std::io::Error },

    #[error("failed to write file {file}: {error}")]
    WriteFile {
        file: PathBuf,
        error: std::io::Error,
    },

    #[cfg(unix)]
    #[error("failed to set permissions for directory {dir}: {error}")]
    SetPermissions { dir: PathBuf, error: std::io::Error },

    #[error("missing nym-api URL")]
    MissingApiUrl,

    #[error("missing nyxd URL")]
    MissingNyxdUrl,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct NymVpnServiceConfig {
    pub(super) entry_point: gateway_directory::EntryPoint,
    pub(super) exit_point: gateway_directory::ExitPoint,
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

// Create the TOML representation of the provided config, only if it doesn't already exists
pub(crate) fn create_config_file<C>(file_path: &PathBuf, config: C) -> Result<C, ConfigSetupError>
where
    C: Serialize,
{
    let config_str = toml::to_string(&config).unwrap();
    tracing::info!("Creating config file at {}", file_path.display());

    // Create path
    let config_dir = file_path
        .parent()
        .ok_or_else(|| ConfigSetupError::GetParentDirectory {
            file: file_path.clone(),
        })?;
    fs::create_dir_all(config_dir).map_err(|error| ConfigSetupError::CreateDirectory {
        dir: config_dir.to_path_buf(),
        error,
    })?;

    if !file_path.exists() {
        fs::write(file_path, config_str).map_err(|error| ConfigSetupError::WriteFile {
            file: file_path.clone(),
            error,
        })?;
        tracing::info!("Config file created at {:?}", file_path.display());
    }
    Ok(config)
}

pub(crate) fn read_config_file<C>(file_path: &PathBuf) -> Result<C, ConfigSetupError>
where
    C: DeserializeOwned,
{
    let file_content =
        fs::read_to_string(file_path).map_err(|error| ConfigSetupError::ReadConfig {
            file: file_path.clone(),
            error,
        })?;
    toml::from_str(&file_content).map_err(|error| ConfigSetupError::Parse {
        file: file_path.clone(),
        error: Box::new(error),
    })
}

pub(crate) fn write_config_file<C>(file_path: &PathBuf, config: C) -> Result<C, ConfigSetupError>
where
    C: Serialize,
{
    let config_str = toml::to_string(&config).unwrap();
    fs::write(file_path, config_str).map_err(|error| ConfigSetupError::WriteFile {
        file: file_path.clone(),
        error,
    })?;
    tracing::info!("Config file updated at {:?}", file_path);
    Ok(config)
}

pub(super) fn create_data_dir(data_dir: &PathBuf) -> Result<(), ConfigSetupError> {
    fs::create_dir_all(data_dir).map_err(|error| ConfigSetupError::CreateDirectory {
        dir: data_dir.clone(),
        error,
    })?;
    tracing::debug!("Making sure data dir exists at {:?}", data_dir);

    #[cfg(unix)]
    {
        // Set directory permissions to 700 (rwx------)
        let permissions = fs::Permissions::from_mode(0o700);
        fs::set_permissions(data_dir, permissions).map_err(|error| {
            ConfigSetupError::SetPermissions {
                dir: data_dir.clone(),
                error,
            }
        })?;
    }

    // TODO: same for windows?

    Ok(())
}
