// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::Paths;

use std::path::PathBuf;

#[cfg(not(windows))]
const DEFAULT_DATA_DIR: &str = "/var/lib/nym-vpnd";
#[cfg(not(windows))]
const DEFAULT_LOG_DIR: &str = "/var/log/nym-vpnd";
#[cfg(not(windows))]
const DEFAULT_CONFIG_DIR: &str = "/etc/nym";

#[cfg(windows)]
pub fn get_paths() -> Paths {
    let program_data_dir = PathBuf::from(
        std::env::var("ProgramData").unwrap_or_else(|_| "C:\\ProgramData".to_string()),
    );

    let nym_vpnd_dir = program_data_dir.join("nym_vpnd");

    let data_dir = std::env::var("NYM_VPND_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| nym_vpnd_dir.join("data"));

    let config_dir = std::env::var("NYM_VPND_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| nym_vpnd_dir.join("config"));

    let log_dir = std::env::var("NYM_VPND_LOG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| nym_vpnd_dir.join("log"));

    Paths {
        data_dir,
        config_dir,
        log_dir,
    }
}

#[cfg(not(windows))]
pub fn get_paths() -> Paths {
    let data_dir = std::env::var("NYM_VPND_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| DEFAULT_DATA_DIR.into());

    let config_dir = std::env::var("NYM_VPND_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| DEFAULT_CONFIG_DIR.into());

    let log_dir = std::env::var("NYM_VPND_LOG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| DEFAULT_LOG_DIR.into());

    Paths {
        data_dir,
        config_dir,
        log_dir,
    }
}
