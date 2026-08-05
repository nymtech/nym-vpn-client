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

const DATA_DIR_VAR: &str = "NYM_VPND_DATA_DIR";
const CONFIG_DIR_VAR: &str = "NYM_VPND_CONFIG_DIR";
const LOG_DIR_VAR: &str = "NYM_VPND_LOG_DIR";

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

        let data_dir = std::env::var(DATA_DIR_VAR)
            .map(PathBuf::from)
            .unwrap_or_else(|_| nym_vpnd_dir.join("data"));

        let config_dir = std::env::var(CONFIG_DIR_VAR)
            .map(PathBuf::from)
            .unwrap_or_else(|_| nym_vpnd_dir.join("config"));

        let log_dir = std::env::var(LOG_DIR_VAR)
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
        let data_dir = std::env::var(DATA_DIR_VAR)
            .map(PathBuf::from)
            .unwrap_or_else(|_| DEFAULT_DATA_DIR.into());

        let config_dir = std::env::var(CONFIG_DIR_VAR)
            .map(PathBuf::from)
            .unwrap_or_else(|_| DEFAULT_CONFIG_DIR.into());

        let log_dir = std::env::var(LOG_DIR_VAR)
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
        for (dir, allow_read) in [
            (&self.data_dir, false),
            (&self.config_dir, false),
            (&self.log_dir, true),
        ] {
            tracing::debug!("Making sure directory exists at {}", dir.display());

            fs::create_dir_all(dir)
                .await
                .map_err(|error| PathsSetupError::CreateDirectory {
                    dir: dir.to_path_buf(),
                    error,
                })?;

            set_permissions(dir, allow_read).await.map_err(|error| {
                PathsSetupError::SetPermissions {
                    dir: dir.to_path_buf(),
                    error,
                }
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

        set_permissions(dir, false)
            .await
            .map_err(|error| PathsSetupError::SetPermissions {
                dir: dir.to_path_buf(),
                error,
            })?;

        Ok(())
    }
}

/// Set restrictive permissions on `path`, unless `allow_read` is set, in which case
/// other users are additionally granted read (but not write) access.
#[cfg(unix)]
async fn set_permissions(path: &Path, allow_read: bool) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    // 700 (rwx------) normally, or 755 (rwxr-xr-x) when read access is allowed.
    let mode = if allow_read { 0o755 } else { 0o700 };
    let permissions = std::fs::Permissions::from_mode(mode);
    fs::set_permissions(path, permissions).await
}

/// Set restrictive permissions on `path`, unless `allow_read` is set, in which case
/// the built-in Users group is additionally granted read & execute access.
#[cfg(windows)]
async fn set_permissions(path: &Path, allow_read: bool) -> nym_windows::security::Result<()> {
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

    let mut entries = vec![allow_admin_group_access];

    if allow_read {
        let users_sid = Sid::well_known(WellKnownSid::BuiltinUsers)?;

        let allow_users_read_access = ExplicitAccess::new(
            Trustee::new(users_sid, TrusteeType::WellKnownGroup),
            AccessMode::SetAccess,
            (FileAccessRights::FILE_GENERIC_READ | FileAccessRights::FILE_GENERIC_EXECUTE).into(),
            AceFlags::OBJECT_INHERIT_ACE | AceFlags::CONTAINER_INHERIT_ACE,
        );

        entries.push(allow_users_read_access);
    }

    let acl = Acl::new(entries)?;

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
