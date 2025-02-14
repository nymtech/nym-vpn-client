// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib::gateway_directory;
use serde::{de::DeserializeOwned, Serialize};
#[cfg(windows)]
use std::os::raw::c_void;
use std::{fmt, fs, path::PathBuf};
#[cfg(windows)]
use std::{mem, ptr};
#[cfg(windows)]
use widestring::U16CString;
#[cfg(windows)]
use winapi::um::winnt::SECURITY_MAX_SID_SIZE;
#[cfg(windows)]
use windows_sys::Win32::Foundation::*;
#[cfg(windows)]
use windows_sys::Win32::Security::Authorization::{SetNamedSecurityInfoW, SE_FILE_OBJECT};
#[cfg(windows)]
use windows_sys::Win32::Security::*;
#[cfg(windows)]
use windows_sys::Win32::Storage::FileSystem::*;
#[cfg(windows)]
use windows_sys::Win32::System::SystemServices::SECURITY_DESCRIPTOR_REVISION;

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
        use std::os::unix::fs::PermissionsExt;
        let permissions = fs::Permissions::from_mode(0o700);
        fs::set_permissions(data_dir, permissions).map_err(|error| {
            ConfigSetupError::SetPermissions {
                dir: data_dir.clone(),
                error,
            }
        })?;
    }

    #[cfg(windows)]
    {
        let wide_path = U16CString::from_os_str(data_dir.as_os_str()).map_err(|e| {
            ConfigSetupError::SetPermissions {
                dir: data_dir.clone(),
                error: std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Invalid UTF-16 conversion: {e}"),
                ),
            }
        })?;

        let mut sid_size: u32 = SECURITY_MAX_SID_SIZE as u32;
        let mut system_sid = vec![0u8; sid_size as usize];

        unsafe {
            if CreateWellKnownSid(
                WinLocalSystemSid,
                ptr::null_mut(),
                system_sid.as_mut_ptr().cast(),
                &mut sid_size,
            ) == 0
            {
                return Err(ConfigSetupError::SetPermissions {
                    dir: data_dir.clone(),
                    error: std::io::Error::last_os_error(),
                });
            }
        }

        let sid_ptr = system_sid.as_mut_ptr().cast();

        let acl_size = mem::size_of::<ACL>() as u32
            + mem::size_of::<ACCESS_ALLOWED_ACE>() as u32
            + unsafe { GetLengthSid(sid_ptr) };

        let mut acl_buffer = vec![0u8; acl_size as usize];
        let acl = acl_buffer.as_mut_ptr() as *mut ACL;

        unsafe {
            if InitializeAcl(acl, acl_size, ACL_REVISION) == 0 {
                return Err(ConfigSetupError::SetPermissions {
                    dir: data_dir.clone(),
                    error: std::io::Error::last_os_error(),
                });
            }

            if AddAccessAllowedAceEx(
                acl,
                ACL_REVISION,
                OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE,
                FILE_GENERIC_READ | FILE_GENERIC_WRITE,
                sid_ptr.cast(),
            ) == 0
            {
                return Err(ConfigSetupError::SetPermissions {
                    dir: data_dir.clone(),
                    error: std::io::Error::last_os_error(),
                });
            }
        }

        let mut security_desc: SECURITY_DESCRIPTOR = unsafe { mem::zeroed() };

        unsafe {
            if InitializeSecurityDescriptor(
                &mut security_desc as *mut _ as *mut _,
                SECURITY_DESCRIPTOR_REVISION,
            ) == 0
            {
                return Err(ConfigSetupError::SetPermissions {
                    dir: data_dir.clone(),
                    error: std::io::Error::last_os_error(),
                });
            }

            if SetSecurityDescriptorDacl(
                &mut security_desc as *mut _ as *mut c_void,
                TRUE,
                acl,
                FALSE,
            ) == 0
            {
                return Err(ConfigSetupError::SetPermissions {
                    dir: data_dir.clone(),
                    error: std::io::Error::last_os_error(),
                });
            }
        }

        unsafe {
            if SetNamedSecurityInfoW(
                wide_path.as_ptr() as *mut _,
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION,
                ptr::null_mut(),
                ptr::null_mut(),
                acl,
                ptr::null_mut(),
            ) != ERROR_SUCCESS
            {
                return Err(ConfigSetupError::SetPermissions {
                    dir: data_dir.clone(),
                    error: std::io::Error::last_os_error(),
                });
            }
        }

        tracing::info!(
            "Successfully set directory permissions for {}",
            data_dir.display()
        );
    }

    Ok(())
}
