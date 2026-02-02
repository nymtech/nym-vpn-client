// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{collections::HashMap, fmt, str::FromStr};

use nym_sdk::mixnet::Recipient;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FeatureFlags {
    flags: HashMap<String, FlagValue>,
}

impl FeatureFlags {
    fn get_group_flag<T>(&self, group: &str, flag: &str) -> Option<T>
    where
        T: FromStr + fmt::Debug,
        <T as FromStr>::Err: fmt::Debug,
    {
        tracing::debug!("Getting feature flag: group={}, flag={}", group, flag);
        self.get_flag(group).and_then(|value| match value {
            FlagValue::Group(group) => group.get(flag).and_then(|v| {
                v.parse::<T>()
                    .inspect_err(|e| tracing::warn!("Failed to parse flag value: {e:#?}"))
                    .ok()
            }),
            _ => None,
        })
    }

    /// Get value for flag, if set
    pub fn get_flag(&self, flag: &str) -> Option<FlagValue> {
        self.flags.get(flag).cloned()
    }

    /// Convert feature flags into a `HashMap<String, FlagValue>`
    pub fn into_hash_map(self) -> HashMap<String, FlagValue> {
        self.flags
    }

    /// Get statistics recipient, if set
    pub fn stats_recipient(&self) -> Option<Recipient> {
        self.get_group_flag("statistics", "recipient")
    }

    /// Get the version of the gateway from where the metadata endpoint should start to be used, if set
    pub fn gw_update_version(&self) -> Option<semver::Version> {
        self.get_group_flag("versions", "gatewayMetadataUpdate")
    }

    /// If domain fronting is enabled or not, if set
    pub fn domain_fronting_enabled(&self) -> Option<bool> {
        self.get_group_flag("domain_fronting", "enabled")
    }

    /// If quic is enabled or not, if set
    pub fn quic_enabled(&self) -> Option<bool> {
        self.get_group_flag("quic", "enabled")
    }

    /// If privy is enabled or not, if set
    pub fn privy_enabled(&self) -> Option<bool> {
        self.get_group_flag("privy", "enabled")
    }

    /// If mixnet tuning is enabled or not, if set
    pub fn mixnet_tuning_enabled(&self) -> Option<bool> {
        self.get_group_flag("mixnet_tuning", "enabled")
    }
}

impl From<HashMap<String, FlagValue>> for FeatureFlags {
    fn from(flags: HashMap<String, FlagValue>) -> Self {
        Self { flags }
    }
}

impl<'a> Deserialize<'a> for FeatureFlags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'a>,
    {
        HashMap::<String, FlagValue>::deserialize(deserializer).map(|flags| Self { flags })
    }
}

impl Serialize for FeatureFlags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        self.flags.serialize(serializer)
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
                    .map(|(key, value)| { format!("{key}: {value}") }),
                ", "
            )
        )
    }
}

impl fmt::Display for FlagValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FlagValue::Value(value) => write!(f, "{value}"),
            FlagValue::Group(group) => {
                write!(
                    f,
                    "{{ {} }}",
                    itertools::join(
                        group
                            .iter()
                            .map(|(key, value)| { format!("{key}: {value}") }),
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
