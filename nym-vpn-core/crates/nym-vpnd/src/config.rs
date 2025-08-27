// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib::nym_config::defaults::NymNetworkDetails;

pub const CURRENT_GLOBAL_CONFIG_VERSION: u32 = 1;

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
            version: CURRENT_GLOBAL_CONFIG_VERSION,
            network_name: NymNetworkDetails::default().network_name,
            sentry_monitoring: false,
            collect_network_statistics: true,
        }
    }
}

impl GlobalConfigFile {
    pub fn read_from_file() -> anyhow::Result<Self> {
        let config_json_path =
            crate::service::config_dir().join(crate::service::DEFAULT_GLOBAL_CONFIG_FILE_JSON);
        let config_json_exists = config_json_path.exists();
        let config_toml_path =
            crate::service::config_dir().join(crate::service::DEFAULT_GLOBAL_CONFIG_FILE_TOML);
        let config_toml_exists = config_toml_path.exists();

        let config = if config_json_exists {
            crate::service::read_json_config_file::<GlobalConfigFile>(&config_json_path)?
        } else if config_toml_exists {
            crate::service::read_toml_config_file::<GlobalConfigFile>(&config_toml_path)?
        } else {
            GlobalConfigFile::default()
        };

        if config_toml_exists {
            tracing::info!(
                "Removing deprecated TOML config file at {}",
                config_toml_path.display()
            );
            let _ = std::fs::remove_file(&config_toml_path);
        }

        crate::service::write_json_config_file(&config_json_path, &config)?;

        Ok(config)
    }

    pub fn write_to_file(&self) -> anyhow::Result<()> {
        let global_config_file_path =
            crate::service::config_dir().join(crate::service::DEFAULT_GLOBAL_CONFIG_FILE_JSON);

        crate::service::write_json_config_file(&global_config_file_path, self).map_err(Into::into)
    }

    // TODO: Use the value in `&self`!
    pub fn sentry_enabled() -> bool {
        let global_config_file_path =
            crate::service::config_dir().join(crate::service::DEFAULT_GLOBAL_CONFIG_FILE_TOML);

        crate::service::read_toml_config_file::<GlobalConfigFile>(&global_config_file_path)
            .inspect_err(|e| {
                eprintln!("failed to read global config file: {e}");
            })
            .ok()
            .is_some_and(|cfg| cfg.sentry_monitoring)
    }
}
