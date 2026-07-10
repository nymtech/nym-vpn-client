// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::LogPath;
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Clone, Debug)]
pub struct Paths {
    pub data_dir: PathBuf,
    pub config_dir: PathBuf,
    pub log_dir: PathBuf,
    pub log_path: Option<LogPath>,
}

#[cfg(not(windows))]
const DEFAULT_DATA_DIR: &str = "/var/lib/nym-vpnd";
#[cfg(not(windows))]
const DEFAULT_LOG_DIR: &str = "/var/log/nym-vpnd";
#[cfg(not(windows))]
const DEFAULT_CONFIG_DIR: &str = "/etc/nym";

impl Paths {
    #[cfg(windows)]
    pub fn new() -> Self {
        let program_data_dir = PathBuf::from(
            std::env::var("ProgramData").unwrap_or_else(|_| "C:\\ProgramData".to_string()),
        );

        let nym_vpnd_dir = program_data_dir.join("nym-vpnd");

        let data_dir = std::env::var("NYM_VPND_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| nym_vpnd_dir.join("data"));

        let config_dir = std::env::var("NYM_VPND_CONFIG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| nym_vpnd_dir.join("config"));

        let log_dir = std::env::var("NYM_VPND_LOG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| nym_vpnd_dir.join("log"));

        Self {
            data_dir,
            config_dir,
            log_dir,
            log_path: None,
        }
    }

    #[cfg(not(windows))]
    pub fn new() -> Self {
        let data_dir = std::env::var("NYM_VPND_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| DEFAULT_DATA_DIR.into());

        let config_dir = std::env::var("NYM_VPND_CONFIG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| DEFAULT_CONFIG_DIR.into());

        let log_dir = std::env::var("NYM_VPND_LOG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| DEFAULT_LOG_DIR.into());

        Self {
            data_dir,
            config_dir,
            log_dir,
            log_path: None,
        }
    }
}

impl Default for Paths {
    fn default() -> Self {
        Self::new()
    }
}

impl Paths {
    pub async fn create_directories(&self) -> Result<(), PathsSetupError> {
        for dir in [&self.data_dir, &self.config_dir, &self.log_dir] {
            tracing::debug!("Making sure directory exists at {}", dir.display());

            fs::create_dir_all(dir)
                .await
                .map_err(|error| PathsSetupError::CreateDirectory {
                    dir: dir.to_path_buf(),
                    error,
                })?;

            set_permissions(dir)
                .await
                .map_err(|error| PathsSetupError::SetPermissions {
                    dir: dir.to_path_buf(),
                    error,
                })?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct NymConfigPaths {
    pub data_dir: PathBuf,
    pub network_data_dir: PathBuf,
    pub config_dir: PathBuf,
    pub log_dir: PathBuf,
    pub log_path: Option<LogPath>,
}

impl NymConfigPaths {
    pub async fn create_directories(&self) -> Result<(), PathsSetupError> {
        let dir = &self.network_data_dir;
        tracing::debug!("Making sure directory exists at {}", dir.display());

        fs::create_dir_all(dir)
            .await
            .map_err(|error| PathsSetupError::CreateDirectory {
                dir: dir.to_path_buf(),
                error,
            })?;

        set_permissions(dir)
            .await
            .map_err(|error| PathsSetupError::SetPermissions {
                dir: dir.to_path_buf(),
                error,
            })?;

        Ok(())
    }
}

#[cfg(unix)]
async fn set_permissions(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    // Set directory permissions to 700 (rwx------)
    let permissions = std::fs::Permissions::from_mode(0o700);
    fs::set_permissions(path, permissions).await
}

#[cfg(windows)]
async fn set_permissions(path: &Path) -> nym_windows::security::Result<()> {
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
        path,
        SecurityObjectType::FileObject,
        SecurityInfo::DACL | SecurityInfo::PROTECTED_DACL,
        None,
        None,
        Some(&acl),
    )?;

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum PathsSetupError {
    #[error("failed to create directory {dir}")]
    CreateDirectory {
        dir: PathBuf,
        #[source]
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
