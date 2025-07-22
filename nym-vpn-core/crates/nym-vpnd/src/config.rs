// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib::nym_config::defaults::NymNetworkDetails;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GlobalConfigFile {
    pub network_name: String,
    #[serde(default)]
    pub sentry_monitoring: bool,
    #[serde(default)]
    pub collect_network_statistics: bool,
}

impl Default for GlobalConfigFile {
    fn default() -> Self {
        Self {
            network_name: NymNetworkDetails::default().network_name,
            sentry_monitoring: false,
            collect_network_statistics: false,
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
                eprintln!("failed to read global config file: {e}");
            })
            .ok()
            .is_some_and(|cfg| cfg.sentry_monitoring)
    }
}

// ---------------- JSON config & migration ----------------

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct GatewaySelector {
    pub r#type: String,    // "Gateway"
    pub identity: Vec<u8>, // bytes
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct GlobalConfigJson {
    pub schema_version: u8,
    pub network_name: String,
    pub sentry_monitoring: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct VpnConfigJson {
    pub schema_version: u8,
    pub entry_point: GatewaySelector,
    pub exit_point:  GatewaySelector,
}

use std::{fs, path::Path};
use toml::Value as TomlValue;

pub fn migrate_toml_to_json(global_toml: &Path, vpn_toml: &Path) -> anyhow::Result<()> {
    let global_raw = fs::read_to_string(global_toml)?;
    let global_val: TomlValue = toml::from_str(&global_raw)?;
    let vpn_raw    = fs::read_to_string(vpn_toml)?;
    let vpn_val: TomlValue = toml::from_str(&vpn_raw)?;

    let global_json = GlobalConfigJson {
        schema_version: 1,
        network_name: global_val["network_name"]
            .as_str()
            .unwrap_or("mainnet")
            .to_string(),
        sentry_monitoring: global_val
            .get("sentry_monitoring")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
    };

    let make_selector = |tbl: &TomlValue| GatewaySelector {
        r#type: "Gateway".into(),
        identity: tbl["identity"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|n| n.as_integer().map(|i| i as u8))
            .collect(),
    };

    let vpn_json = VpnConfigJson {
        schema_version: 1,
        entry_point: make_selector(&vpn_val["entry_point"]),
        exit_point:  make_selector(&vpn_val["exit_point"]),
    };

    fs::write(global_toml.with_extension("json"),
              serde_json::to_vec_pretty(&global_json)?)?;
    fs::write(vpn_toml.with_extension("json"),
              serde_json::to_vec_pretty(&vpn_json)?)?;

    fs::rename(global_toml, global_toml.with_extension("toml.bak"))?;
    fs::rename(vpn_toml,    vpn_toml.with_extension("toml.bak"))?;

    Ok(())
}
