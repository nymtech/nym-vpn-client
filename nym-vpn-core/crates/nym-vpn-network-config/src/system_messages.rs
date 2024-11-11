// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{collections::HashMap, fmt};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::response::SystemMessageResponse;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SystemMessages {
    pub messages: Vec<SystemMessage>,
}

impl SystemMessages {
    pub fn all_messages(&self) -> impl Iterator<Item = &SystemMessage> {
        self.messages.iter()
    }

    pub fn current_messages(&self) -> impl Iterator<Item = &SystemMessage> {
        self.messages.iter().filter(|msg| msg.is_current())
    }

    pub fn into_current_messages(self) -> impl Iterator<Item = SystemMessage> {
        self.messages.into_iter().filter(|msg| msg.is_current())
    }
}

impl fmt::Display for SystemMessages {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "System messages: [")?;
        for message in self.current_messages() {
            writeln!(f, "   {}", message)?;
        }
        writeln!(f, "]")?;
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemMessage {
    pub name: String,
    pub display_from: Option<OffsetDateTime>,
    pub display_until: Option<OffsetDateTime>,
    pub message: String,
    pub properties: Properties,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Properties(HashMap<String, String>);

impl fmt::Display for Properties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "properties {{")?;
        for (key, value) in &self.0 {
            write!(f, " {}: {},", key, value)?;
        }
        write!(f, "}}")
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
        writeln!(
            f,
            "SystemMessage(name: {}, message: {}, properties: {}",
            self.name, self.message, self.properties
        )
    }
}

impl SystemMessage {
    pub fn is_current(&self) -> bool {
        let now = OffsetDateTime::now_utc();
        self.display_from.map_or(true, |from| from <= now)
            && self.display_until.map_or(true, |until| until >= now)
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
            messages: responses
                .into_iter()
                .filter_map(|m| {
                    SystemMessage::try_from(m)
                        .inspect_err(|err| tracing::warn!("Failed to parse system message: {err}"))
                        .ok()
                })
                .collect(),
        }
    }
}

impl TryFrom<SystemMessageResponse> for SystemMessage {
    type Error = anyhow::Error;

    fn try_from(response: SystemMessageResponse) -> Result<Self, Self::Error> {
        let display_from = OffsetDateTime::parse(&response.display_from, &Rfc3339)
            .with_context(|| format!("Failed to parse display_from: {}", response.display_from))
            .ok();
        let display_until = OffsetDateTime::parse(&response.display_until, &Rfc3339)
            .with_context(|| format!("Failed to parse display_until: {}", response.display_until))
            .ok();

        let properties =
            Properties::deserialize(response.properties).unwrap_or(Properties::default());

        Ok(Self {
            name: response.name,
            display_from,
            display_until,
            message: response.message,
            properties,
        })
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
        let message = SystemMessage::try_from(parsed).unwrap();
        assert_eq!(
            message,
            SystemMessage {
                name: "test_message".to_string(),
                display_from: Some(
                    OffsetDateTime::parse("2024-11-05T12:00:00.000Z", &Rfc3339).unwrap()
                ),
                display_until: None,
                message: "This is a test message, no need to panic!".to_string(),
                properties: Properties(HashMap::from_iter(vec![(
                    "modal".to_string(),
                    "true".to_string()
                )])),
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
            properties: Properties(HashMap::from_iter(vec![(
                "modal".to_string(),
                "true".to_string(),
            )])),
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
            properties: Properties(HashMap::from_iter(vec![(
                "modal".to_string(),
                "true".to_string(),
            )])),
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
            properties: Properties(HashMap::from_iter(vec![(
                "modal".to_string(),
                "true".to_string(),
            )])),
        };
        assert!(!message.is_current());
    }
}
