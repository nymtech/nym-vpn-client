// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VpnServiceConfigExtV1 {
    pub entry_point: EntryPointExtV1,
    pub exit_point: ExitPointExtV1,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryPointExtV1 {
    Gateway { identity: String },
    Location { location: String },
    Random,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExitPointExtV1 {
    Address { address: String },
    Gateway { identity: String },
    Location { location: String },
    Random,
}
