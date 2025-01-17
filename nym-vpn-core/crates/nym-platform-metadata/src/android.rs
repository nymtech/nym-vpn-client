// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::collections::HashMap;

mod command;
use command::command_stdout_lossy;

pub fn version() -> String {
    let version = os_version();
    let api_level = get_prop("ro.build.version.sdk").unwrap_or_else(|| "N/A".to_owned());

    let manufacturer =
        get_prop("ro.product.manufacturer").unwrap_or_else(|| "Unknown brand".to_owned());
    let product = get_prop("ro.product.model").unwrap_or_else(|| "Unknown model".to_owned());

    format!(
        "Android {} (API: {}) - {} {}",
        version, api_level, manufacturer, product
    )
}

pub fn short_version() -> String {
    let version = os_version();

    format!("Android {}", version)
}

fn os_version() -> String {
    get_prop("ro.build.version.release").unwrap_or_else(|| "N/A".to_owned())
}

pub fn extra_metadata() -> HashMap<String, String> {
    let mut metadata = HashMap::new();
    metadata.insert(
        "abi".to_owned(),
        get_prop("ro.product.cpu.abilist").unwrap_or_else(|| "N/A".to_owned()),
    );
    metadata
}

fn get_prop(property: &str) -> Option<String> {
    command_stdout_lossy("getprop", &[property]).ok()
}
