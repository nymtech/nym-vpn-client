// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{collections::HashMap, fmt};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub flags: Flags,
}

impl TryFrom<serde_json::Value> for FeatureFlags {
    type Error = serde_json::Error;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        Flags::deserialize(value).map(|flags| Self { flags })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Flags {
    String(String),
    Map(HashMap<String, Flags>),
}

impl fmt::Display for Flags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Flags::String(s) => write!(f, "{}", s),
            Flags::Map(m) => {
                write!(f, "{{")?;
                for (key, value) in m {
                    write!(f, "{}: {}, ", key, value)?;
                }
                write!(f, "}}")
            }
        }
    }
}
