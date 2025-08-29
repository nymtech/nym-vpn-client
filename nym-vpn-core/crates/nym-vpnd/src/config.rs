// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Context, anyhow};
use nym_vpn_lib::nym_config::defaults::NymNetworkDetails;
use serde::{Deserialize, Serialize};

// Derserialize is only for reading the deprecated TOML version.
#[derive(Clone, Debug, Deserialize)]
pub struct GlobalConfig {
    pub network_name: String,
    pub sentry_monitoring: bool,
    pub collect_network_statistics: bool,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            network_name: NymNetworkDetails::default().network_name,
            sentry_monitoring: false,
            collect_network_statistics: true,
        }
    }
}

impl GlobalConfig {
    pub fn read_from_file() -> anyhow::Result<Self> {
        let json_config_path =
            crate::service::config_dir().join(crate::service::DEFAULT_GLOBAL_CONFIG_FILE_JSON);
        let json_config_exists = json_config_path.exists();
        let toml_config_path =
            crate::service::config_dir().join(crate::service::DEFAULT_GLOBAL_CONFIG_FILE_TOML);
        let toml_config_exists = toml_config_path.exists();

        let config = if json_config_exists {
            let ext_config =
                crate::service::read_json_config_file::<GlobalConfigExt>(&json_config_path)
                    .context(anyhow!(
                        "Failed to read global config file {}",
                        json_config_path.display()
                    ))?;
            GlobalConfig::try_from(ext_config).context(anyhow!(
                "Failed to parse global config file {}",
                json_config_path.display()
            ))?
        } else if toml_config_exists {
            crate::service::read_toml_config_file::<GlobalConfig>(&toml_config_path).context(
                anyhow!(
                    "Failed to read global config file {}",
                    toml_config_path.display()
                ),
            )?
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

        // Always write back the config file back using the latest JSON version
        config.write_to_file()?;

        Ok(config)
    }

    pub fn write_to_file(&self) -> anyhow::Result<()> {
        let json_config_path =
            crate::service::config_dir().join(crate::service::DEFAULT_GLOBAL_CONFIG_FILE_JSON);

        let ext_config = GlobalConfigExt::try_from(self).context(anyhow!(
            "Failed to convert global config to external representation for writing"
        ))?;

        crate::service::write_json_config_file(&json_config_path, &ext_config).context(anyhow!(
            "Failed to write global config file {}",
            json_config_path.display()
        ))
    }

    // Calling this means the global configuration file is read twice 😒
    pub fn sentry_enabled() -> bool {
        let config = Self::read_from_file().unwrap_or_default();
        config.sentry_monitoring
    }
}

//
// External, versioned, representation of the global config file.
//

type GlobalConfigExtLatest = GlobalConfigExtV1;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "version")]
#[serde(rename_all = "snake_case")]
enum GlobalConfigExt {
    V1(GlobalConfigExtV1),
}

impl TryFrom<GlobalConfigExt> for GlobalConfig {
    type Error = crate::service::ConfigSetupError;

    fn try_from(value: GlobalConfigExt) -> Result<Self, Self::Error> {
        match value {
            GlobalConfigExt::V1(v1) => GlobalConfig::try_from(v1),
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
            collect_network_statistics: value.collect_network_statistics,
        })
    }
}

impl TryFrom<&GlobalConfig> for GlobalConfigExtLatest {
    type Error = crate::service::ConfigSetupError;

    fn try_from(value: &GlobalConfig) -> Result<Self, Self::Error> {
        Ok(GlobalConfigExtLatest {
            network_name: value.network_name.clone(),
            sentry_monitoring: value.sentry_monitoring,
            collect_network_statistics: value.collect_network_statistics,
        })
    }
}

//
// Because of the way these tests configure the config directory, then need to be run single-threaded:
//
// cargo test --package nym-vpnd config::tests -- --nocapture --test-threads=1
//

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env::set_var, fs, path::PathBuf};
    use tempfile::tempdir;

    // Config directory will be deleted on drop
    fn setup() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path();

        println!("Using config dir: {config_path:?}");

        unsafe { set_var("NYM_VPND_CONFIG_DIR", config_path) }; // See service::config::config_dir()

        let toml_path = config_path.join(crate::service::DEFAULT_GLOBAL_CONFIG_FILE_TOML);
        let _ = fs::remove_file(&toml_path);

        let json_path = config_path.join(crate::service::DEFAULT_GLOBAL_CONFIG_FILE_JSON);
        let _ = fs::remove_file(&json_path);

        (temp_dir, toml_path, json_path)
    }

    #[test]
    fn test_global_config_migrate() {
        let (_temp_dir, toml_path, json_path) = setup();

        // Write the TOML config file
        let toml_content = r#"
network_name = "tulips"
sentry_monitoring = false
collect_network_statistics = true
"#;
        fs::write(&toml_path, toml_content).unwrap();

        // Read the TOML config and migrate it to JSON
        let config = GlobalConfig::read_from_file().unwrap();
        assert_eq!(config.network_name, "tulips");
        assert!(!config.sentry_monitoring);
        assert!(config.collect_network_statistics);

        // The TOML file should be deleted and replaced with a JSON version
        assert!(!toml_path.exists());
        assert!(json_path.exists());

        // Read the JSON config
        let config = GlobalConfig::read_from_file().unwrap();
        assert_eq!(config.network_name, "tulips");
        assert!(!config.sentry_monitoring);
        assert!(config.collect_network_statistics);
    }
}
