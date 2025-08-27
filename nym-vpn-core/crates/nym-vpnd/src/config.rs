// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib::nym_config::defaults::NymNetworkDetails;

//
// In order to allow migrations to work in the future:
//
// - Do not remove any fields; instead use `Option<T>`.
// - Do not remove any enum variants; instead just ignore them.
//
// Migration must be performed at a higher level where more knowledge
// of the environment is available.
//
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GlobalConfigFile {
    #[serde(default = "crate::service::default_one")]
    pub version: u32,

    pub network_name: String,

    #[serde(default)]
    pub sentry_monitoring: bool,

    #[serde(default = "crate::service::default_true")]
    pub collect_network_statistics: bool,
}

impl Default for GlobalConfigFile {
    fn default() -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            network_name: NymNetworkDetails::default().network_name,
            sentry_monitoring: false,
            collect_network_statistics: true,
        }
    }
}

impl GlobalConfigFile {
    pub const CURRENT_VERSION: u32 = 1;

    pub fn read_from_file() -> anyhow::Result<Self> {
        let json_config_path =
            crate::service::config_dir().join(crate::service::DEFAULT_GLOBAL_CONFIG_FILE_JSON);
        let json_config_exists = json_config_path.exists();
        let toml_config_path =
            crate::service::config_dir().join(crate::service::DEFAULT_GLOBAL_CONFIG_FILE_TOML);
        let toml_config_exists = toml_config_path.exists();

        let config = if json_config_exists {
            crate::service::read_json_config_file::<GlobalConfigFile>(&json_config_path)
                .map_err(|err| {
                    tracing::error!(
                        "Failed to read global config file {:?}: {:?}",
                        json_config_path,
                        err
                    );
                })
                .unwrap_or_default()
        } else if toml_config_exists {
            crate::service::read_toml_config_file::<GlobalConfigFile>(&toml_config_path)
                .map_err(|err| {
                    tracing::error!(
                        "Failed to read global config file {:?}: {:?}",
                        toml_config_path,
                        err
                    );
                })
                .unwrap_or_default()
        } else {
            tracing::info!("No global configuration file exists; using default configuration");
            GlobalConfigFile::default()
        };

        if toml_config_exists {
            tracing::info!(
                "Removing deprecated global config file {:?}",
                toml_config_path
            );
            let _ = std::fs::remove_file(&toml_config_path);
        }

        crate::service::write_json_config_file(&json_config_path, &config)?;

        Ok(config)
    }

    pub fn write_to_file(&self) -> anyhow::Result<()> {
        let global_config_file_path =
            crate::service::config_dir().join(crate::service::DEFAULT_GLOBAL_CONFIG_FILE_JSON);

        crate::service::write_json_config_file(&global_config_file_path, self).map_err(Into::into)
    }

    // Calling this means the global configuration file is read twice 😒
    pub fn sentry_enabled() -> bool {
        let config = Self::read_from_file().unwrap_or_default();
        config.sentry_monitoring
    }
}
