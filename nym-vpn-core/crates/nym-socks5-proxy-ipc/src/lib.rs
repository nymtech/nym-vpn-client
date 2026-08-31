// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};
use std::{
    fmt,
    net::{Ipv4Addr, Ipv6Addr},
    path::PathBuf,
    str::FromStr,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum DaemonMessage {
    Configure(ProxyConfig),
    SetTunnelAddresses(InterfaceAddresses),
    SetExcludedCountries(Vec<String>),
    Terminate,
}

impl fmt::Display for DaemonMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(
            &serde_json::to_string(self).expect("DaemonMessage serialisation should not fail"),
        )
    }
}

impl FromStr for DaemonMessage {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub listen_port: u16,
    pub data_dir: PathBuf,
    pub log_dir: PathBuf,
    pub log_level: String,
    pub excluded_countries: Vec<String>,
}

impl ProxyConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.listen_port == 0 {
            return Err("listen_port must be a valid non-zero port number".into());
        }

        if self.data_dir.as_os_str().is_empty() || !self.data_dir.is_dir() {
            return Err("data_dir must be a valid path".into());
        }

        if self.log_dir.as_os_str().is_empty() || !self.log_dir.is_dir() {
            return Err("log_dir must be a valid path".into());
        }

        // The log_level can be more than just "info", "debug", etc., so just check it's not empty
        if self.log_level.is_empty() {
            return Err("log_level cannot be empty".into());
        }

        validate_country_codes(&self.excluded_countries)
    }
}

pub fn validate_country_codes(countries: &[String]) -> Result<(), String> {
    for country in countries {
        if country.len() != 2 || !country.chars().all(|c| c.is_ascii_uppercase()) {
            return Err(format!(
                "Invalid excluded country code '{}': must be a 2-letter uppercase string",
                country
            ));
        }
    }

    let mut seen = std::collections::HashSet::new();
    for country in countries {
        if !seen.insert(country) {
            return Err(format!("Duplicate excluded country code: '{}'", country));
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InterfaceAddresses {
    pub v4_addr: Option<Ipv4Addr>,
    pub v6_addr: Option<Ipv6Addr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ProxyMessage {
    Ack,
    Status(StatusData),
    Error(ErrorData),
}

impl fmt::Display for ProxyMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(
            &serde_json::to_string(self).expect("ProxyMessage serialisation should not fail"),
        )
    }
}

impl FromStr for ProxyMessage {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusData {
    /// Number of currently active SOCKS5 client connections.
    pub active_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorData {
    pub message: String,
}
