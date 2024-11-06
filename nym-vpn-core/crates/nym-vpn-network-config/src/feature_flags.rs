// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FeatureFlags {
    pub(super) flags: serde_json::Value,
}

impl From<serde_json::Value> for FeatureFlags {
    fn from(value: serde_json::Value) -> Self {
        Self { flags: value }
    }
}
