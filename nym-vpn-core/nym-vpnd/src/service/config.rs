// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    fmt, fs,
    path::{Path, PathBuf},
};

use nym_vpn_lib::gateway_directory;
use tracing::info;

#[cfg(not(windows))]
const DEFAULT_DATA_DIR: &str = "/var/lib/nym-vpnd";
#[cfg(not(windows))]
const DEFAULT_LOG_DIR: &str = "/var/log/nym-vpnd";
#[cfg(not(windows))]
const DEFAULT_CONFIG_DIR: &str = "/etc/nym";
pub(super) const DEFAULT_CONFIG_FILE: &str = "nym-vpnd.toml";
pub(crate) const DEFAULT_LOG_FILE: &str = "nym-vpnd.log";

#[cfg(windows)]
pub(crate) fn program_data_path() -> PathBuf {
    PathBuf::from(std::env::var("ProgramData").unwrap_or(std::env::var("PROGRAMDATA").unwrap()))
}

pub(super) fn default_data_dir() -> PathBuf {
    #[cfg(windows)]
    return program_data_path().join("nym-vpnd").join("data");

    #[cfg(not(windows))]
    return DEFAULT_DATA_DIR.into();
}

pub(crate) fn default_log_dir() -> PathBuf {
    #[cfg(windows)]
    return program_data_path().join("nym-vpnd").join("log");

    #[cfg(not(windows))]
    return DEFAULT_LOG_DIR.into();
}

pub(super) fn default_config_dir() -> PathBuf {
    #[cfg(windows)]
    return program_data_path().join("nym-vpnd").join("config");

    #[cfg(not(windows))]
    return DEFAULT_CONFIG_DIR.into();
}

#[derive(thiserror::Error, Debug)]
pub(super) enum ConfigSetupError {
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
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(super) struct NymVpnServiceConfig {
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

pub(super) fn create_config_file(
    config_file: &PathBuf,
    config: NymVpnServiceConfig,
) -> Result<NymVpnServiceConfig, ConfigSetupError> {
    let config_str = toml::to_string(&config).unwrap();

    // Create path
    let config_dir = config_file
        .parent()
        .ok_or_else(|| ConfigSetupError::GetParentDirectory {
            file: config_file.clone(),
        })?;
    fs::create_dir_all(config_dir).map_err(|error| ConfigSetupError::CreateDirectory {
        dir: config_dir.to_path_buf(),
        error,
    })?;

    fs::write(config_file, config_str).map_err(|error| ConfigSetupError::WriteFile {
        file: config_file.clone(),
        error,
    })?;
    info!("Config file created at {:?}", config_file);
    Ok(config)
}

pub(super) fn read_config_file(
    config_file: &PathBuf,
) -> Result<NymVpnServiceConfig, ConfigSetupError> {
    let file_content =
        fs::read_to_string(config_file).map_err(|error| ConfigSetupError::ReadConfig {
            file: config_file.clone(),
            error,
        })?;
    toml::from_str(&file_content).map_err(|error| ConfigSetupError::Parse {
        file: config_file.clone(),
        error: Box::new(error),
    })
}

pub(super) fn write_config_file(
    config_file: &PathBuf,
    config: &NymVpnServiceConfig,
) -> Result<(), ConfigSetupError> {
    let config_str = toml::to_string(config).unwrap();
    fs::write(config_file, config_str).map_err(|error| ConfigSetupError::WriteFile {
        file: config_file.clone(),
        error,
    })?;
    info!("Config file updated at {:?}", config_file);
    Ok(())
}

pub(super) fn create_data_dir(data_dir: &PathBuf) -> Result<(), ConfigSetupError> {
    fs::create_dir_all(data_dir).map_err(|error| ConfigSetupError::CreateDirectory {
        dir: data_dir.clone(),
        error,
    })?;
    info!("Making sure data dir exists at {:?}", data_dir);
    Ok(())
}

pub(super) async fn create_device_keys(
    data_dir: &Path,
) -> Result<(), nym_vpn_store::KeyStoreError> {
    // Check if the device keys already exists, if not then create them
    if nym_vpn_store::keypair_exists(data_dir)? {
        nym_vpn_store::create_device_keys(data_dir).await?;
    }

    // Check that we can successfully load them
    nym_vpn_store::load_device_keys(data_dir).await.map(|_| ())
}
