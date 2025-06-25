// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib::nym_config::defaults::NymNetworkDetails;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GlobalConfigFile {
    pub network_name: String,
    pub sentry_monitoring: Option<bool>,
}

impl Default for GlobalConfigFile {
    fn default() -> Self {
        Self {
            network_name: NymNetworkDetails::default().network_name,
            sentry_monitoring: None,
        }
    }
}

impl GlobalConfigFile {
    pub fn read_from_file() -> anyhow::Result<Self> {
        let global_config_file_path =
            crate::service::config_dir().join(crate::service::DEFAULT_GLOBAL_CONFIG_FILE);

        crate::service::create_config_file(&global_config_file_path, &GlobalConfigFile::default())?;
        crate::service::read_config_file(&global_config_file_path).map_err(Into::into)
    }

    pub fn write_to_file(&self) -> anyhow::Result<Self> {
        let global_config = self.clone();
        let global_config_file_path =
            crate::service::config_dir().join(crate::service::DEFAULT_GLOBAL_CONFIG_FILE);

        crate::service::write_config_file(&global_config_file_path, global_config)
            .map_err(Into::into)
    }

    pub fn sentry_enabled() -> bool {
        let global_config_file_path =
            crate::service::config_dir().join(crate::service::DEFAULT_GLOBAL_CONFIG_FILE);

        crate::service::read_config_file::<GlobalConfigFile>(&global_config_file_path)
            .inspect_err(|e| {
                eprintln!("failed to read global config file: {}", e);
            })
            .ok()
            .is_some_and(|cfg| matches!(cfg.sentry_monitoring, Some(true)))
    }
}
