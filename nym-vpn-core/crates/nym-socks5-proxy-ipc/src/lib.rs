// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{fmt, net::IpAddr, path::PathBuf, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum DaemonMessage {
    Configure(ProxyConfig),
    VpnConnected(VpnConnectedData),
    VpnDisconnected,
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

        let valid_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_levels.contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log_level: {}", self.log_level));
        }

        for country in &self.excluded_countries {
            if country.len() != 2 || !country.chars().all(|c| c.is_ascii_uppercase()) {
                return Err(format!(
                    "Invalid excluded country code '{}': must be a 2-letter uppercase string",
                    country
                ));
            }
        }

        let mut seen = std::collections::HashSet::new();
        for country in &self.excluded_countries {
            if !seen.insert(country) {
                return Err(format!("Duplicate excluded country code: '{}'", country));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpnConnectedData {
    pub tunnel_addr: IpAddr,
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
