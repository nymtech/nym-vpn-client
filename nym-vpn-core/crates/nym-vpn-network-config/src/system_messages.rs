// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{collections::HashMap, fmt};

use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use nym_vpn_api_client::response::{SystemConfigurationResponse, SystemMessageResponse};
use url::Url;

use crate::system_configuration::{ScoreThresholds, SystemConfiguration};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct SystemMessages {
    pub messages: Vec<SystemMessage>,
}

impl<'de> Deserialize<'de> for SystemMessages {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let messages = Vec::<SystemMessage>::deserialize(deserializer)?;
        Ok(SystemMessages { messages })
    }
}

impl Serialize for SystemMessages {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.messages.serialize(serializer)
    }
}

impl SystemMessages {
    pub fn current_iter(&self) -> impl Iterator<Item = &SystemMessage> {
        self.messages.iter().filter(|msg| msg.is_current())
    }

    pub fn into_current_iter(self) -> impl Iterator<Item = SystemMessage> {
        self.messages.into_iter().filter(|msg| msg.is_current())
    }

    pub fn into_current_messages(self) -> SystemMessages {
        self.into_current_iter().collect::<Vec<_>>().into()
    }
}

impl IntoIterator for SystemMessages {
    type Item = SystemMessage;
    type IntoIter = std::vec::IntoIter<SystemMessage>;

    fn into_iter(self) -> Self::IntoIter {
        self.messages.into_iter()
    }
}

impl<'a> IntoIterator for &'a SystemMessages {
    type Item = &'a SystemMessage;
    type IntoIter = std::slice::Iter<'a, SystemMessage>;

    fn into_iter(self) -> Self::IntoIter {
        self.messages.iter()
    }
}

impl fmt::Display for SystemMessages {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{{[")?;
        for message in self {
            writeln!(f, "   {message},")?;
        }
        write!(f, "]}}")
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemMessage {
    pub name: String,
    pub display_from: Option<OffsetDateTime>,
    pub display_until: Option<OffsetDateTime>,
    pub message: String,
    pub properties: Option<Properties>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Properties(HashMap<String, String>);

impl fmt::Display for Properties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ {} }}",
            itertools::join(self.0.iter().map(|(k, v)| format!("{k}: {v}")), ", ")
        )
    }
}

impl Properties {
    pub fn into_inner(self) -> HashMap<String, String> {
        self.0
    }
}

impl From<HashMap<String, String>> for Properties {
    fn from(map: HashMap<String, String>) -> Self {
        Self(map)
    }
}

impl fmt::Display for SystemMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ name: \"{}\", message: \"{}\", properties: {} }}",
            self.name,
            self.message,
            self.properties.as_ref().unwrap_or(&Properties::default())
        )
    }
}

impl SystemMessage {
    pub fn is_current(&self) -> bool {
        let now = OffsetDateTime::now_utc();
        self.display_from.is_none_or(|from| from <= now)
            && self.display_until.is_none_or(|until| until >= now)
    }
}

impl From<Vec<SystemMessage>> for SystemMessages {
    fn from(messages: Vec<SystemMessage>) -> Self {
        Self { messages }
    }
}

impl From<Vec<SystemMessageResponse>> for SystemMessages {
    fn from(responses: Vec<SystemMessageResponse>) -> Self {
        Self {
            messages: responses.into_iter().map(SystemMessage::from).collect(),
        }
    }
}

impl From<SystemMessageResponse> for SystemMessage {
    fn from(response: SystemMessageResponse) -> Self {
        let display_from = OffsetDateTime::parse(&response.display_from, &Rfc3339)
            .inspect_err(|e| {
                tracing::warn!(
                    "Failed to parse display_from ({}): {}",
                    response.display_from,
                    e
                )
            })
            .ok();

        let display_until = if !response.display_until.is_empty() {
            OffsetDateTime::parse(&response.display_until, &Rfc3339)
                .inspect_err(|e| {
                    tracing::warn!(
                        "Failed to parse display_until ({}): {}",
                        response.display_until,
                        e
                    )
                })
                .ok()
        } else {
            None
        };

        let properties = if !response.properties.is_null() {
            Properties::deserialize(response.properties)
                .inspect_err(|e| tracing::warn!("Failed to parse properties: {}", e))
                .ok()
        } else {
            None
        };

        Self {
            name: response.name,
            display_from,
            display_until,
            message: response.message,
            properties,
        }
    }
}

impl From<SystemConfigurationResponse> for SystemConfiguration {
    fn from(value: SystemConfigurationResponse) -> Self {
        let statistics_api = value.statistics_api.and_then(|url| {
            Url::parse(&url)
                .inspect_err(|err| tracing::warn!("Failed to parse statistics API url: {err}"))
                .ok()
        });

        let min_supported_app_versions = value.min_supported_app_versions;

        SystemConfiguration {
            mix_thresholds: ScoreThresholds {
                high: value.mix_thresholds.high,
                medium: value.mix_thresholds.medium,
                low: value.mix_thresholds.low,
            },
            wg_thresholds: ScoreThresholds {
                high: value.wg_thresholds.high,
                medium: value.wg_thresholds.medium,
                low: value.wg_thresholds.low,
            },
            statistics_api,
            min_supported_app_versions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_system_message() {
        let json = r#"{
            "name": "test_message",
            "displayFrom": "2024-11-05T12:00:00.000Z",
            "displayUntil": "",
            "message": "This is a test message, no need to panic!",
            "properties": {
                "modal": "true"
            }
        }"#;
        let parsed: SystemMessageResponse = serde_json::from_str(json).unwrap();
        let message = SystemMessage::from(parsed);
        assert_eq!(
            message,
            SystemMessage {
                name: "test_message".to_string(),
                display_from: Some(
                    OffsetDateTime::parse("2024-11-05T12:00:00.000Z", &Rfc3339).unwrap()
                ),
                display_until: None,
                message: "This is a test message, no need to panic!".to_string(),
                properties: Some(Properties(HashMap::from_iter(vec![(
                    "modal".to_string(),
                    "true".to_string()
                )]))),
            }
        );
    }

    #[test]
    fn check_current_message() {
        let message = SystemMessage {
            name: "test_message".to_string(),
            // Yesterday
            display_from: Some(OffsetDateTime::now_utc() - time::Duration::days(1)),
            display_until: None,
            message: "This is a test message, no need to panic!".to_string(),
            properties: Some(Properties(HashMap::from_iter(vec![(
                "modal".to_string(),
                "true".to_string(),
            )]))),
        };
        assert!(message.is_current());
    }

    #[test]
    fn check_future_message() {
        let message = SystemMessage {
            name: "test_message".to_string(),
            // Tomorrow
            display_from: Some(OffsetDateTime::now_utc() + time::Duration::days(1)),
            display_until: None,
            message: "This is a test message, no need to panic!".to_string(),
            properties: Some(Properties(HashMap::from_iter(vec![(
                "modal".to_string(),
                "true".to_string(),
            )]))),
        };
        assert!(!message.is_current());
    }

    #[test]
    fn check_expired_message() {
        let message = SystemMessage {
            name: "test_message".to_string(),
            // Yesterday
            display_from: Some(OffsetDateTime::now_utc() - time::Duration::days(1)),
            // Today
            display_until: Some(OffsetDateTime::now_utc()),
            message: "This is a test message, no need to panic!".to_string(),
            properties: Some(Properties(HashMap::from_iter(vec![(
                "modal".to_string(),
                "true".to_string(),
            )]))),
        };
        assert!(!message.is_current());
    }
}
