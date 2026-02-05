// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use strum::{IntoDiscriminant, IntoEnumIterator};

use crate::{
    nym_config::defaults::NymNetworkDetails,
    service::{ConfigSetupError, DEFAULT_GLOBAL_CONFIG_FILE_JSON, DEFAULT_GLOBAL_CONFIG_FILE_TOML},
};

#[derive(Debug, thiserror::Error)]
pub enum GlobalConfigError {
    #[error("failed to write global config file: {file_path}")]
    Write {
        file_path: PathBuf,
        #[source]
        source: ConfigSetupError,
    },

    #[error("failed to read global config file: {file_path}")]
    Read {
        file_path: PathBuf,
        #[source]
        source: ConfigSetupError,
    },

    #[error("failed to parse global config file: {file_path}")]
    Parse {
        file_path: PathBuf,
        #[source]
        source: ConfigSetupError,
    },

    #[error("failed to convert global config to external representation for writing")]
    ExtRepr(#[source] ConfigSetupError),
}

pub type Result<T, E = GlobalConfigError> = std::result::Result<T, E>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlobalConfig {
    pub network_name: String,
    pub sentry_monitoring: bool,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            network_name: NymNetworkDetails::default().network_name,
            sentry_monitoring: false,
        }
    }
}

impl GlobalConfig {
    pub async fn read_from_config_dir(config_dir: &Path) -> Result<Self> {
        match Self::read_config(config_dir).await {
            Ok(config) => Ok(config),
            Err(err) => {
                tracing::error!("Failed to read global config file; using default : {err}");
                let config = GlobalConfig::default();
                config.write_to_config_dir(config_dir).await?;
                Ok(config)
            }
        }
    }

    async fn read_config(config_dir: &Path) -> Result<Self> {
        let json_config_path = config_dir.join(DEFAULT_GLOBAL_CONFIG_FILE_JSON);
        let json_config_exists = json_config_path.exists();
        let toml_config_path = config_dir.join(DEFAULT_GLOBAL_CONFIG_FILE_TOML);
        let toml_config_exists = toml_config_path.exists();

        let config = if json_config_exists {
            let ext_config =
                crate::service::read_json_config_file::<GlobalConfigExt>(&json_config_path)
                    .await
                    .map_err(|err| GlobalConfigError::Read {
                        file_path: json_config_path.clone(),
                        source: err,
                    })?;

            let should_overwrite = !ext_config.is_latest_version();
            let config =
                GlobalConfig::try_from(ext_config).map_err(|err| GlobalConfigError::Parse {
                    file_path: json_config_path,
                    source: err,
                })?;
            // Write migrated config back to disk
            if should_overwrite {
                config.write_to_config_dir(config_dir).await?;
            }
            config
        } else if toml_config_exists {
            let legacy_config =
                crate::service::read_toml_config_file::<LegacyGlobalConfig>(&toml_config_path)
                    .await
                    .map_err(|err| GlobalConfigError::Read {
                        file_path: toml_config_path.clone(),
                        source: err,
                    })?;
            let migrated_config =
                GlobalConfig::try_from(legacy_config).map_err(|err| GlobalConfigError::Parse {
                    file_path: toml_config_path.clone(),
                    source: err,
                })?;
            // Write migrated config back to disk
            migrated_config.write_to_config_dir(config_dir).await?;
            migrated_config
        } else {
            let config = GlobalConfig::default();
            config.write_to_config_dir(config_dir).await?;
            config
        };

        if toml_config_exists {
            tracing::info!(
                "Removing deprecated global config file {}",
                toml_config_path.display()
            );
            if let Err(e) = tokio::fs::remove_file(&toml_config_path).await {
                tracing::error!("Failed to remove deprecated global config file: {e}");
            }
        }

        Ok(config)
    }

    pub async fn write_to_config_dir(&self, config_dir: &Path) -> Result<()> {
        let json_config_path = config_dir.join(crate::service::DEFAULT_GLOBAL_CONFIG_FILE_JSON);
        let ext_config = GlobalConfigExt::try_from(self).map_err(GlobalConfigError::ExtRepr)?;

        crate::service::write_json_config_file(&json_config_path, &ext_config)
            .await
            .map_err(|err| GlobalConfigError::Write {
                file_path: json_config_path,
                source: err,
            })
    }
}

//
// External, versioned, representation of the global config file.
//

type GlobalConfigExtLatest = GlobalConfigExtV2;

#[derive(Clone, Debug, Serialize, Deserialize, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::EnumIter))]
#[serde(tag = "version")]
#[serde(rename_all = "snake_case")]
enum GlobalConfigExt {
    V1(GlobalConfigExtV1),
    V2(GlobalConfigExtV2),
}

impl GlobalConfigExt {
    /// Returns true if the config is using the latest version.
    pub fn is_latest_version(&self) -> bool {
        let current_version = self.discriminant();
        let latest_version = GlobalConfigExtDiscriminants::iter().next_back();

        latest_version == Some(current_version)
    }
}

impl TryFrom<GlobalConfigExt> for GlobalConfig {
    type Error = crate::service::ConfigSetupError;

    fn try_from(value: GlobalConfigExt) -> Result<Self, Self::Error> {
        match value {
            GlobalConfigExt::V1(v1) => GlobalConfig::try_from(v1),
            GlobalConfigExt::V2(v2) => GlobalConfig::try_from(v2),
        }
    }
}

impl TryFrom<&GlobalConfig> for GlobalConfigExt {
    type Error = crate::service::ConfigSetupError;

    fn try_from(value: &GlobalConfig) -> Result<Self, Self::Error> {
        // Always construct the latest external representation, for writing to disk
        let latest = GlobalConfigExtLatest::try_from(value)?;
        Ok(latest.into())
    }
}

//
// v2
//
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct GlobalConfigExtV2 {
    network_name: String,
    sentry_monitoring: bool,
}

impl Default for GlobalConfigExtV2 {
    fn default() -> Self {
        Self {
            network_name: NymNetworkDetails::default().network_name,
            sentry_monitoring: false,
        }
    }
}

impl From<GlobalConfigExtV2> for GlobalConfigExt {
    fn from(v2: GlobalConfigExtV2) -> Self {
        GlobalConfigExt::V2(v2)
    }
}

impl TryFrom<GlobalConfigExtV2> for GlobalConfig {
    type Error = crate::service::ConfigSetupError;

    fn try_from(value: GlobalConfigExtV2) -> Result<Self, Self::Error> {
        Ok(GlobalConfig {
            network_name: value.network_name,
            sentry_monitoring: value.sentry_monitoring,
        })
    }
}

impl TryFrom<&GlobalConfig> for GlobalConfigExtLatest {
    type Error = crate::service::ConfigSetupError;

    fn try_from(value: &GlobalConfig) -> Result<Self, Self::Error> {
        Ok(GlobalConfigExtLatest {
            network_name: value.network_name.clone(),
            sentry_monitoring: value.sentry_monitoring,
        })
    }
}

//
// v1
//
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct GlobalConfigExtV1 {
    network_name: String,
    sentry_monitoring: bool,
    collect_network_statistics: bool,
}

impl Default for GlobalConfigExtV1 {
    fn default() -> Self {
        Self {
            network_name: NymNetworkDetails::default().network_name,
            sentry_monitoring: false,
            collect_network_statistics: true,
        }
    }
}

impl From<GlobalConfigExtV1> for GlobalConfigExt {
    fn from(v1: GlobalConfigExtV1) -> Self {
        GlobalConfigExt::V1(v1)
    }
}

impl TryFrom<GlobalConfigExtV1> for GlobalConfig {
    type Error = crate::service::ConfigSetupError;

    fn try_from(value: GlobalConfigExtV1) -> Result<Self, Self::Error> {
        Ok(GlobalConfig {
            network_name: value.network_name,
            sentry_monitoring: value.sentry_monitoring,
        })
    }
}

//
// Legacy TOML version of the config file
//
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct LegacyGlobalConfig {
    network_name: String,

    #[serde(default)]
    sentry_monitoring: bool,

    #[serde(default = "default_true")]
    collect_network_statistics: bool,
}

impl TryFrom<LegacyGlobalConfig> for GlobalConfig {
    type Error = crate::service::ConfigSetupError;

    fn try_from(value: LegacyGlobalConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            network_name: value.network_name,
            sentry_monitoring: value.sentry_monitoring,
        })
    }
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;
    use tokio::fs;

    // Config directory will be deleted on drop
    async fn setup() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path();

        let toml_path = config_path.join(crate::service::DEFAULT_GLOBAL_CONFIG_FILE_TOML);
        let json_path = config_path.join(crate::service::DEFAULT_GLOBAL_CONFIG_FILE_JSON);

        (temp_dir, toml_path, json_path)
    }

    #[test]
    fn test_config_is_latest() {
        let v1 = GlobalConfigExt::V1(GlobalConfigExtV1::default());
        let latest = GlobalConfigExt::from(GlobalConfigExtLatest::default());

        assert!(!v1.is_latest_version());
        assert!(latest.is_latest_version());
    }

    #[tokio::test]
    async fn test_global_config_migrate() {
        let (temp_dir, toml_path, json_path) = setup().await;

        let toml_content = r#"
network_name = "tulips"
sentry_monitoring = false
collect_network_statistics = true
"#;

        let json_content = r#"{
  "version": "v2",
  "network_name": "tulips",
  "sentry_monitoring": false
}"#;

        // Write the TOML config file
        fs::write(&toml_path, toml_content).await.unwrap();

        // Read the TOML config and migrate it to JSON
        let config = GlobalConfig::read_from_config_dir(temp_dir.path())
            .await
            .unwrap();
        assert_eq!(config.network_name, "tulips");
        assert!(!config.sentry_monitoring);

        // The TOML file should be deleted and replaced with a JSON version
        assert!(!toml_path.exists());
        assert!(json_path.exists());

        // Read the JSON config
        let config = GlobalConfig::read_from_config_dir(temp_dir.path())
            .await
            .unwrap();
        assert_eq!(config.network_name, "tulips");
        assert!(!config.sentry_monitoring);

        // Check the JSON is the right version and all snake-case
        let read_json_content = fs::read_to_string(&json_path).await.unwrap();
        assert_eq!(json_content, read_json_content);
    }

    #[tokio::test]
    async fn test_global_config_fallback_default() {
        let (temp_dir, toml_path, json_path) = setup().await;

        let broken_toml_content = r#"
netwoXrk_name = "tulips"
sentry_monitoring = false
collect_network_statistics = true
"#;

        let broken_json_content = r#"{
  "version": "v2",
  "network_name": "tulips",
  "sentry_mXonitoring": false
}"#;

        // Write the (broken) TOML config file
        fs::write(&toml_path, broken_toml_content).await.unwrap();
        let config = GlobalConfig::read_from_config_dir(temp_dir.path())
            .await
            .unwrap();
        assert_eq!(config, GlobalConfig::default());

        // Write the (broken) JSON config file
        fs::write(&json_path, broken_json_content).await.unwrap();
        let config = GlobalConfig::read_from_config_dir(temp_dir.path())
            .await
            .unwrap();
        assert_eq!(config, GlobalConfig::default());
    }
}
