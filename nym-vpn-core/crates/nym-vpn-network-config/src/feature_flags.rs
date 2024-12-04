// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{collections::HashMap, fmt};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub flags: HashMap<String, FlagValue>,
}

impl FeatureFlags {
    pub fn get_flag(&self, flag: &str) -> Option<FlagValue> {
        self.flags.get(flag).cloned()
    }
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
        write!(
            f,
            "{{ {} }}",
            itertools::join(
                self.flags
                    .iter()
                    .map(|(key, value)| { format!("{}: {}", key, value) }),
                ", "
            )
        )
    }
}

impl fmt::Display for FlagValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FlagValue::Value(value) => write!(f, "{}", value),
            FlagValue::Group(group) => {
                write!(
                    f,
                    "{{ {} }}",
                    itertools::join(
                        group
                            .iter()
                            .map(|(key, value)| { format!("{}: {}", key, value) }),
                        ", "
                    )
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use nym_sdk::mixnet::Recipient;
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

    #[test]
    fn parse_statistics() {
        let json = r#"{
            "showaccounts": "true",
            "website": {
                "showaccounts": "true",
                "foo": "bar"
            },
            "zknyms": {
                "credentialmode": "false"
            },
            "statistics": {
                "recipient": "6Yu1b6cb3TJNProLHSL1kAiDcpiRxBrhiqUbP9uDz3xz.8boeihWTpiMNzCzdWmeDgc77yUZio47kdRRaLiqvXqyC@8wH1ScVTGnBVxLjrA3hzZ8m55dvpkiNrpqTet6ccchFV",
                "foo": "bar"
            }
        }"#;
        let parsed: Value = serde_json::from_str(json).unwrap();
        let flags = FeatureFlags::try_from(parsed).unwrap();

        let recipient = "6Yu1b6cb3TJNProLHSL1kAiDcpiRxBrhiqUbP9uDz3xz\
                         .8boeihWTpiMNzCzdWmeDgc77yUZio47kdRRaLiqvXqyC\
                         @8wH1ScVTGnBVxLjrA3hzZ8m55dvpkiNrpqTet6ccchFV";
        assert_eq!(
            flags.flags["statistics"],
            FlagValue::Group(HashMap::from([
                ("recipient".to_owned(), recipient.to_owned()),
                ("foo".to_owned(), "bar".to_owned()),
            ]))
        );
        assert_eq!(
            match flags.flags.get("statistics").unwrap() {
                FlagValue::Group(group) => group
                    .get("recipient")
                    .and_then(|v| v.parse::<Recipient>().ok())
                    .unwrap(),
                _ => panic!("unexpected flag value"),
            },
            Recipient::try_from_base58_string(recipient).unwrap(),
        );
    }
}
