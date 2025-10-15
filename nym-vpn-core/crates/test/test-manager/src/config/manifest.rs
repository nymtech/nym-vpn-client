// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Config definition, see [`Config`].

mod test_locations;
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use test_locations::TestLocationList;

use super::VmConfig;

/// Global configuration for the `test-manager`.
///
/// Can be modified using either the setting file, see
/// [`crate::config::io::ConfigFile::get_config_path`] or
/// the `test-manager config` CLI subcommand.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(skip)]
    pub runtime_opts: RuntimeOptions,
    pub vms: BTreeMap<String, VmConfig>,
    #[serde(default)]
    pub test_locations: TestLocationList,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct RuntimeOptions {
    pub display: Display,
    pub keep_changes: bool,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub enum Display {
    #[default]
    None,
    Local,
    Vnc,
}

impl Config {
    pub fn get_vm(&self, name: &str) -> Option<&VmConfig> {
        self.vms.get(name)
    }
}
