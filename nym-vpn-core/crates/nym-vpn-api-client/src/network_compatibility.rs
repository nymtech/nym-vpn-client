use serde::{Deserialize, Serialize};
use std::fmt;

use crate::response::NetworkCompatibilityResponse;

// A platform that is missing from discovery parses as an empty minimum version, which never prompts.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct NetworkCompatibility {
    pub core: String,
    pub ios: String,
    pub macos: String,
    pub tauri: String,
    pub android: String,
}

impl fmt::Display for NetworkCompatibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "core: {}, ios: {}, macos: {}, tauri: {}, android: {}",
            self.core, self.ios, self.macos, self.tauri, self.android
        )
    }
}

impl From<NetworkCompatibilityResponse> for NetworkCompatibility {
    fn from(response: NetworkCompatibilityResponse) -> Self {
        NetworkCompatibility {
            core: response.core,
            ios: response.ios,
            macos: response.macos,
            tauri: response.tauri,
            android: response.android,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_platform_minimum_parses_as_empty_instead_of_failing() {
        let parsed: NetworkCompatibility =
            serde_json::from_str(r#"{"android": "2026.14.0"}"#).unwrap();
        assert_eq!(parsed.android, "2026.14.0");
        assert_eq!(parsed.ios, "");
    }
}
