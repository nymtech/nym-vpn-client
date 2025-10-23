// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::collections::HashMap;

use super::command::command_stdout_lossy;

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
