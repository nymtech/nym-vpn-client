use super::path::APP_CONFIG_DIR;
use crate::APP_CONFIG_FILE;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct AppConfig {
    /// Whether Sentry error monitoring is enabled
    pub sentry_monitoring: bool,
    /// Whether app debug logging to a file is enabled
    pub debug_logging: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            sentry_monitoring: false,
            debug_logging: crate::DEFAULT_DEBUG_LOGGING,
        }
    }
}

impl AppConfig {
    /// Read the config file in a raw way, ignoring any errors
    /// If something goes wrong return silently
    /// This method is used to read the config file (if it exists)
    /// at early stages on app start
    pub fn read() -> anyhow::Result<Self> {
        let mut path = APP_CONFIG_DIR
            .clone()
            .ok_or(anyhow!("failed to get app config dir"))?;
        path.push(APP_CONFIG_FILE);
        let content = fs::read_to_string(&path)?;
        toml::from_str(&content).map_err(|e| anyhow!(e))
    }
}
