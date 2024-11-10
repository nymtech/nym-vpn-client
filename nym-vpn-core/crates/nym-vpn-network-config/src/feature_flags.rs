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

impl TryFrom<serde_json::Value> for FeatureFlags {
    type Error = serde_json::Error;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        HashMap::<String, FlagValue>::deserialize(value).map(|flags| Self { flags })
    }
}

impl fmt::Display for FeatureFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        for (key, value) in &self.flags {
            write!(f, "{}: {}, ", key, value)?;
        }
        write!(f, "}}")
    }
}

impl fmt::Display for FlagValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FlagValue::Value(value) => write!(f, "{}", value),
            FlagValue::Group(group) => {
                write!(f, "{{")?;
                for (key, value) in group {
                    write!(f, "{}: {}, ", key, value)?;
                }
                write!(f, "}}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn parse_flat_list() {
        let json = r#"{
            "showaccounts": "true"
        }"#;
        let parsed: Value = serde_json::from_str(json).unwrap();
        let flags = FeatureFlags::try_from(parsed).unwrap();
        assert_eq!(
            flags.flags["showaccounts"],
            FlagValue::Value("true".to_string())
        );
    }

    #[test]
    fn parse_nested_list() {
        let json = r#"{
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
        assert_eq!(
            flags.flags["website"],
            FlagValue::Group(HashMap::from([
                ("showaccounts".to_owned(), "true".to_owned()),
                ("foo".to_owned(), "bar".to_owned())
            ]))
        );
        assert_eq!(
            flags.flags["zknyms"],
            FlagValue::Group(HashMap::from([(
                "credentialmode".to_owned(),
                "false".to_owned()
            )]))
        );
    }

    #[test]
    fn parse_mixed_list() {
        let json = r#"{
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
        let flags = FeatureFlags::try_from(parsed).unwrap();
        assert_eq!(
            flags.flags["showaccounts"],
            FlagValue::Value("true".to_string())
        );
        assert_eq!(
            flags.flags["website"],
            FlagValue::Group(HashMap::from([
                ("showaccounts".to_owned(), "true".to_owned()),
                ("foo".to_owned(), "bar".to_owned())
            ]))
        );
        assert_eq!(
            flags.flags["zknyms"],
            FlagValue::Group(HashMap::from([(
                "credentialmode".to_owned(),
                "false".to_owned()
            )]))
        );
    }
}
