// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{fs, path::PathBuf};

use nym_vpn_lib::gateway_directory;
use tracing::info;

pub(super) const DEFAULT_DATA_DIR: &str = "/var/lib/nym-vpnd";
pub(super) const DEFAULT_CONFIG_DIR: &str = "/etc/nym";
pub(super) const DEFAULT_CONFIG_FILE: &str = "nym-vpnd.toml";

#[derive(thiserror::Error, Debug)]
pub(super) enum ConfigSetupError {
    #[error("failed to parse config file {file}: {error}")]
    Parse {
        file: PathBuf,
        error: toml::de::Error,
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
        error,
    })
}

pub(super) fn create_data_dir(data_dir: &PathBuf) -> Result<(), ConfigSetupError> {
    fs::create_dir_all(data_dir).map_err(|error| ConfigSetupError::CreateDirectory {
        dir: data_dir.clone(),
        error,
    })?;
    info!("Data directory created at {:?}", data_dir);
    Ok(())
}
