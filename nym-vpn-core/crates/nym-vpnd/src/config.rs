// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Context, anyhow};
use nym_vpn_lib::nym_config::defaults::NymNetworkDetails;
use serde::{Deserialize, Serialize};
use std::path::Path;

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
    pub async fn read_from_default_config_dir() -> anyhow::Result<Self> {
        let config_dir = crate::service::config_dir();
        Self::read_from_config_dir(&config_dir).await
    }

    pub async fn read_from_config_dir(config_dir: &Path) -> anyhow::Result<Self> {
        let config = Self::read_config(config_dir).await.unwrap_or_else(|err| {
            tracing::error!("Failed to read global config file; using default : {err}");
            GlobalConfig::default()
        });

        // Always write back config file back using the latest JSON version
        // TODO: Avoid doing this as it's double-writing the config file.
        config.write_to_config_dir(config_dir).await?;

        Ok(config)
    }

    async fn read_config(config_dir: &Path) -> anyhow::Result<Self> {
        let json_config_path = config_dir.join(crate::service::DEFAULT_GLOBAL_CONFIG_FILE_JSON);
        let json_config_exists = json_config_path.exists();
        let toml_config_path = config_dir.join(crate::service::DEFAULT_GLOBAL_CONFIG_FILE_TOML);
        let toml_config_exists = toml_config_path.exists();

        let config = if json_config_exists {
            let ext_config =
                crate::service::read_json_config_file::<GlobalConfigExt>(&json_config_path)
                    .await
                    .context(anyhow!(
                        "Failed to read global config file {}",
                        json_config_path.display()
                    ))?;
            GlobalConfig::try_from(ext_config).context(anyhow!(
                "Failed to parse global config file {}",
                json_config_path.display()
            ))?
        } else if toml_config_exists {
            let legacy_config =
                crate::service::read_toml_config_file::<LegacyGlobalConfig>(&toml_config_path)
                    .await
                    .context(anyhow!(
                        "Failed to read global config file {}",
                        toml_config_path.display()
                    ))?;
            GlobalConfig::try_from(legacy_config).context(anyhow!(
                "Failed to parse global config file {}",
                toml_config_path.display()
            ))?
        } else {
            GlobalConfig::default()
        };

        if toml_config_exists {
            tracing::info!(
                "Removing deprecated global config file {}",
                toml_config_path.display()
            );
            let _ = std::fs::remove_file(&toml_config_path);
        }

        Ok(config)
    }

    pub async fn write_to_default_config_dir(&self) -> anyhow::Result<()> {
        let config_dir = crate::service::config_dir();
        self.write_to_config_dir(&config_dir).await
    }

    pub async fn write_to_config_dir(&self, config_dir: &Path) -> anyhow::Result<()> {
        let json_config_path = config_dir.join(crate::service::DEFAULT_GLOBAL_CONFIG_FILE_JSON);

        let ext_config = GlobalConfigExt::try_from(self).context(anyhow!(
            "Failed to convert global config to external representation for writing"
        ))?;

        crate::service::write_json_config_file(&json_config_path, &ext_config)
            .await
            .context(anyhow!(
                "Failed to write global config file {}",
                json_config_path.display()
            ))
    }

    // Calling this means the global configuration file is read twice 😒
    pub async fn sentry_enabled() -> bool {
        let config = Self::read_from_default_config_dir()
            .await
            .unwrap_or_default();
        config.sentry_monitoring
    }
}

//
// External, versioned, representation of the global config file.
//

type GlobalConfigExtLatest = GlobalConfigExtV2;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "version")]
#[serde(rename_all = "snake_case")]
enum GlobalConfigExt {
    V1(GlobalConfigExtV1),
    V2(GlobalConfigExtV2),
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
        let _ = fs::remove_file(&toml_path).await;

        let json_path = config_path.join(crate::service::DEFAULT_GLOBAL_CONFIG_FILE_JSON);
        let _ = fs::remove_file(&json_path).await;

        (temp_dir, toml_path, json_path)
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
