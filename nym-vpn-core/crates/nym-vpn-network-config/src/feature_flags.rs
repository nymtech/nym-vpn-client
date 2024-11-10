// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{collections::HashMap, fmt};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub flags: HashMap<String, FlagValue>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FlagValue {
    Value(String),
    Group(HashMap<String, String>),
}

// Struct used during deserialization to handle the nested structure of the flags
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct InnerFlags(HashMap<String, FlagValue>);

impl TryFrom<serde_json::Value> for FeatureFlags {
    type Error = serde_json::Error;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        InnerFlags::deserialize(value).map(|flags| Self { flags: flags.0 })
    }
}

//impl fmt::Display for Flags {
//    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//        match self {
//            Flags::String(s) => write!(f, "{}", s),
//            Flags::Map(m) => {
//                write!(f, "{{")?;
//                for (key, value) in m {
//                    write!(f, "{}: {}, ", key, value)?;
//                }
//                write!(f, "}}")
//            }
//        }
//    }
//}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn parse_single_flag() {
        let json = r#"
        {
            "showaccounts": "true"
        }"#;
        let parsed: Value = serde_json::from_str(json).unwrap();
        // let flags = FeatureFlags::deserialize(parsed).unwrap();
        let flags = FeatureFlags::try_from(parsed).unwrap();
        dbg!(&flags);
    }

    // Example:
    //
    // "feature_flags": {
    //   "website": {
    //     "showaccounts": "true"
    //   },
    //   "zknyms": {
    //     "credentialmode": "false"
    //   }
    // }

    #[test]
    fn parse_flags_with_groups() {
        let json = r#"
        {
            "website": {
                "showaccounts": "true",
                "foo": "bar"
            },
            "zknyms": {
                "credentialmode": "false"
            }
        }"#;
        let parsed: Value = serde_json::from_str(json).unwrap();
        let flags = FeatureFlags::try_from(parsed).unwrap();
        dbg!(&flags);
    }

    #[test]
    fn parse_mixed_flags() {
        let json = r#"
        {
            "showaccounts": "true",
            "website": {
                "showaccounts": "true",
                "foo": "bar"
            },
            "zknyms": {
                "credentialmode": "false"
            }
        }"#;
        let parsed: Value = serde_json::from_str(json).unwrap();
        // let flags = FeatureFlags::try_from(parsed).unwrap();
        // dbg!(&flags);
    }
}
