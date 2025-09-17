// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::error::{Error, Result};
use nym_vpn_lib::gateway_directory;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    fmt, fs,
    path::{Path, PathBuf},
    str::FromStr,
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

#[derive(Clone, Debug)]
pub struct NymVpnServiceConfig {
    pub(super) entry_point: gateway_directory::EntryPoint,
    pub(super) exit_point: gateway_directory::ExitPoint,
}

impl NymVpnServiceConfig {
    #[allow(clippy::result_large_err)]
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

type NymVpnServiceConfigExtLatest = NymVpnServiceConfigExtV1;

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
        // Always construct the latest external representation, writing to disk
        let latest = NymVpnServiceConfigExtLatest::try_from(value)?;
        Ok(latest.into())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct NymVpnServiceConfigExtV1 {
    entry_point: EntryPointExtV1,
    exit_point: ExitPointExtV1,
}

impl Default for NymVpnServiceConfigExtV1 {
    fn default() -> Self {
        Self {
            entry_point: EntryPointExtV1::Random,
            exit_point: ExitPointExtV1::Random,
        }
    }
}

impl From<NymVpnServiceConfigExtV1> for NymVpnServiceConfigExt {
    fn from(v1: NymVpnServiceConfigExtV1) -> Self {
        NymVpnServiceConfigExt::V1(v1)
    }
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

impl TryFrom<&NymVpnServiceConfig> for NymVpnServiceConfigExtLatest {
    type Error = ConfigSetupError;

    fn try_from(value: &NymVpnServiceConfig) -> Result<Self, Self::Error> {
        let entry_point = EntryPointExtV1::try_from(&value.entry_point)?;
        let exit_point = ExitPointExtV1::try_from(&value.exit_point)?;
        Ok(NymVpnServiceConfigExtLatest {
            entry_point,
            exit_point,
        })
    }
}

//
// EntryPointExtV1
//

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum EntryPointExtV1 {
    Gateway { identity: String },
    Location { location: String },
    Random,
}

impl TryFrom<EntryPointExtV1> for gateway_directory::EntryPoint {
    type Error = ConfigSetupError;

    fn try_from(value: EntryPointExtV1) -> Result<Self, Self::Error> {
        match value {
            EntryPointExtV1::Gateway { ref identity } => {
                gateway_directory::EntryPoint::from_base58_string(identity)
                    .map_err(|e| ConfigSetupError::EntryPoint { error: Box::new(e) })
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
                identity: identity.to_base58_string(),
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
#[serde(rename_all = "snake_case")]
enum ExitPointExtV1 {
    Address { address: String },
    Gateway { identity: String },
    Location { location: String },
    Random,
}

impl TryFrom<ExitPointExtV1> for gateway_directory::ExitPoint {
    type Error = ConfigSetupError;

    fn try_from(value: ExitPointExtV1) -> Result<Self, Self::Error> {
        match value {
            ExitPointExtV1::Address { address } => {
                let recipient = gateway_directory::Recipient::from_str(&address).map_err(|e| {
                    ConfigSetupError::ExitPoint {
                        error: Box::new(gateway_directory::Error::RecipientFormattingError {
                            address: address.clone(),
                            source: e,
                        }),
                    }
                })?;
                Ok(gateway_directory::ExitPoint::Address {
                    address: Box::new(recipient),
                })
            }
            ExitPointExtV1::Gateway { identity } => {
                let node_identity =
                    gateway_directory::NodeIdentity::from_str(&identity).map_err(|e| {
                        ConfigSetupError::ExitPoint {
                            error: Box::new(gateway_directory::Error::NodeIdentityFormattingError {
                                identity: identity.clone(),
                                source: e,
                            }),
                        }
                    })?;
                Ok(gateway_directory::ExitPoint::Gateway {
                    identity: node_identity,
                })
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
                address: address.to_string(),
            }),
            gateway_directory::ExitPoint::Gateway { identity } => Ok(ExitPointExtV1::Gateway {
                identity: identity.to_string(),
            }),
            gateway_directory::ExitPoint::Location { location } => Ok(ExitPointExtV1::Location {
                location: location.clone(),
            }),
            gateway_directory::ExitPoint::Random => Ok(ExitPointExtV1::Random),
        }
    }
}

//
// Legacy TOML version of config file.
//

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct LegacyNymVpnServiceConfig {
    entry_point: gateway_directory::EntryPoint,
    exit_point: gateway_directory::ExitPoint,
}

impl TryFrom<LegacyNymVpnServiceConfig> for NymVpnServiceConfig {
    type Error = ConfigSetupError;

    fn try_from(value: LegacyNymVpnServiceConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            entry_point: value.entry_point,
            exit_point: value.exit_point,
        })
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

    #[error("failed to convert entry point")]
    EntryPoint {
        #[source]
        error: Box<gateway_directory::Error>,
    },
    #[error("failed to convert exit point")]
    ExitPoint {
        #[source]
        error: Box<gateway_directory::Error>,
    },
}

#[allow(clippy::result_large_err)]
pub(super) fn setup_service_config(
    toml_config_path: &Path,
    json_config_path: &Path,
    entry: Option<gateway_directory::EntryPoint>,
    exit: Option<gateway_directory::ExitPoint>,
) -> Result<NymVpnServiceConfig> {
    let json_config_exists = json_config_path.exists();
    let toml_config_exists = toml_config_path.exists();

    let mut config = if json_config_exists {
        let ext_config = read_json_config_file::<NymVpnServiceConfigExt>(json_config_path)
            .map_err(Error::ConfigSetup)?;

        NymVpnServiceConfig::try_from(ext_config).map_err(Error::ConfigSetup)?
    } else if toml_config_exists {
        let legacy_config = read_toml_config_file::<LegacyNymVpnServiceConfig>(toml_config_path)
            .map_err(Error::ConfigSetup)?;
        NymVpnServiceConfig::try_from(legacy_config).map_err(Error::ConfigSetup)?
    } else {
        NymVpnServiceConfig::default()
    };

    config.entry_point = entry.unwrap_or(config.entry_point);
    config.exit_point = exit.unwrap_or(config.exit_point);

    if toml_config_exists {
        tracing::info!(
            "Removing deprecated config file {}",
            toml_config_path.display()
        );
        let _ = fs::remove_file(toml_config_path);
    }

    // Always write back config file back using the latest JSON version
    // TODO: Avoid doing this as it's double-writing the config file.
    config.write_to_file(json_config_path)?;

    Ok(config)
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
    let file = fs::File::open(file_path).map_err(|error| ConfigSetupError::ReadConfig {
        file: file_path.to_path_buf(),
        error,
    })?;
    let reader = std::io::BufReader::new(file);
    serde_json::from_reader(reader).map_err(|error| ConfigSetupError::ParseJson {
        file: file_path.to_path_buf(),
        error: Box::new(error),
    })
}

pub fn write_json_config_file<C>(file_path: &Path, config: &C) -> Result<(), ConfigSetupError>
where
    C: Serialize,
{
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

    let file = fs::File::create(file_path).map_err(|error| ConfigSetupError::WriteFile {
        file: file_path.to_path_buf(),
        error,
    })?;
    let writer = std::io::BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &config).map_err(|error| {
        ConfigSetupError::SerializeJson {
            file: file_path.to_path_buf(),
            error: Box::new(error),
        }
    })?;

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
                    dir: dir_path.to_path_buf(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf};
    use tempfile::tempdir;

    // Config directory will be deleted on drop
    fn setup() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path();

        println!("Using config dir: {config_path:?}");

        let service_path = config_path.join("tulips");
        let _ = fs::create_dir_all(&service_path);

        let toml_path = service_path.join(DEFAULT_CONFIG_FILE_TOML);
        let _ = fs::remove_file(&toml_path);

        let json_path = service_path.join(DEFAULT_CONFIG_FILE_JSON);
        let _ = fs::remove_file(&json_path);

        (temp_dir, toml_path, json_path)
    }

    fn run_test(
        toml_content: &str,
        json_content: &str,
        entry_point: gateway_directory::EntryPoint,
        exit_point: gateway_directory::ExitPoint,
    ) {
        let (_temp_dir, toml_path, json_path) = setup();

        // Write the TOML config file
        fs::write(&toml_path, toml_content).unwrap();

        // Read the TOML config and migrate it to JSON
        let config = setup_service_config(&toml_path, &json_path, None, None).unwrap();
        assert_eq!(config.entry_point, entry_point);
        assert_eq!(config.exit_point, exit_point);

        // The TOML file should be deleted and replaced with a JSON version
        assert!(!toml_path.exists());
        assert!(json_path.exists());

        // Read the JSON config
        let config = setup_service_config(&toml_path, &json_path, None, None).unwrap();
        assert_eq!(config.entry_point, entry_point);
        assert_eq!(config.exit_point, exit_point);

        // Check the JSON is the right version and all snake-case
        let read_json_content = fs::read_to_string(&json_path).unwrap();
        assert_eq!(json_content, read_json_content);
    }

    #[test]
    fn test_service_config_migrate_location() {
        let toml_content = r#"
[entry_point.Location]
location = "FR"

[exit_point.Location]
location = "BE"
"#;

        let json_content = r#"{
  "version": "v1",
  "entry_point": {
    "location": {
      "location": "FR"
    }
  },
  "exit_point": {
    "location": {
      "location": "BE"
    }
  }
}"#;

        let entry_point = gateway_directory::EntryPoint::Location {
            location: "FR".to_string(),
        };

        let exit_point = gateway_directory::ExitPoint::Location {
            location: "BE".to_string(),
        };

        run_test(toml_content, json_content, entry_point, exit_point);
    }

    #[test]
    fn test_service_config_migrate_gateway() {
        let toml_content = r#"
[entry_point.Gateway]
identity = [ 92, 25, 33, 77, 4, 117, 82, 117, 246, 239, 233, 11, 129, 183, 86, 194, 140, 95, 21, 196, 121, 130, 232, 195, 71, 173, 66, 124, 5, 14, 114, 107, ]

[exit_point.Gateway]
identity = [ 99, 23, 98, 234, 66, 161, 195, 63, 155, 161, 250, 207, 17, 158, 136, 114, 215, 90, 236, 161, 231, 176, 140, 190, 147, 182, 64, 171, 145, 31, 245, 186, ]
"#;

        let json_content = r#"{
  "version": "v1",
  "entry_point": {
    "gateway": {
      "identity": "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42"
    }
  },
  "exit_point": {
    "gateway": {
      "identity": "7fp3cmzCvgeRgbB1ycTnK6RokjHNqPmCCSBG23gyxshj"
    }
  }
}"#;

        let entry_point = gateway_directory::EntryPoint::Gateway {
            identity: gateway_directory::NodeIdentity::from_str(
                "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42",
            )
            .unwrap(),
        };

        let exit_point = gateway_directory::ExitPoint::Gateway {
            identity: gateway_directory::NodeIdentity::from_str(
                "7fp3cmzCvgeRgbB1ycTnK6RokjHNqPmCCSBG23gyxshj",
            )
            .unwrap(),
        };

        run_test(toml_content, json_content, entry_point, exit_point);
    }

    #[test]
    #[ignore] // Temporarily disabled due to issues with ExitPoint::Address (de)serialisation.
    fn test_service_config_migrate_address() {
        let toml_content = r#"
[entry_point.Gateway]
identity = [92, 25, 33, 77, 4, 117, 82, 117, 246, 239, 233, 11, 129, 183, 86, 194, 140, 95, 21, 196, 121, 130, 232, 195, 71, 173, 66, 124, 5, 14, 114, 107]

[exit_point.Address]
address = [5, 56, 84, 195, 94, 238, 210, 124, 65, 143, 209, 144, 22, 255, 91, 188, 35, 50, 144, 234, 226, 114, 99, 40, 10, 102, 200, 170, 19, 162, 86, 134, 84, 20, 195, 193, 42, 194, 230, 153, 163, 90, 214, 216, 196, 166, 87, 132, 206, 215, 91, 89, 51, 98, 72, 156, 159, 248, 109, 225, 152, 204, 80, 97, 9, 62, 22, 108, 155, 95, 153, 29, 143, 48, 208, 5, 101, 231, 176, 93, 107, 229, 11, 225, 145, 1, 14, 219, 44, 88, 199, 206, 40, 185, 150, 151]
"#;

        let json_content = r#"{
  "version": "v1",
  "entry_point": {
    "gateway": {
      "identity": "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42"
    }
  },
  "exit_point": {
    "address": {
      "address": "MNrmKzuKjNdbEhfPUzVNfjw63oBQNSayqoQKGL4JjAV.6fDcSN6faGpvA3pd3riCwjpzXc7RQfWmGMa82UVoEwKE@d5adfJNtcdZW2XwK85JAAU8nXAs9JCPYn2RNvDLZn4e
    }
  }
}"#;

        let entry_point = gateway_directory::EntryPoint::Gateway {
            identity: gateway_directory::NodeIdentity::from_str(
                "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42",
            )
            .unwrap(),
        };

        let exit_point = gateway_directory::ExitPoint::Address {
            address: Box::new(
                gateway_directory::Recipient::from_str("MNrmKzuKjNdbEhfPUzVNfjw63oBQNSayqoQKGL4JjAV.6fDcSN6faGpvA3pd3riCwjpzXc7RQfWmGMa82UVoEwKE@d5adfJNtcdZW2XwK85JAAU8nXAs9JCPYn2RNvDLZn4e").unwrap(),
            )
        };

        run_test(toml_content, json_content, entry_point, exit_point);
    }

    #[test]
    fn test_service_config_migrate_random() {
        let toml_content = r#"
entry_point = "Random"
exit_point = "Random"
"#;

        let json_content = r#"{
  "version": "v1",
  "entry_point": "random",
  "exit_point": "random"
}"#;

        let entry_point = gateway_directory::EntryPoint::Random;

        let exit_point = gateway_directory::ExitPoint::Random;

        run_test(toml_content, json_content, entry_point, exit_point);
    }
}
