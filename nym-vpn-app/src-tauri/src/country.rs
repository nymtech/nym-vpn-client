use serde::{Deserialize, Serialize};
use std::fmt;
use ts_rs::TS;

#[derive(Serialize, Deserialize, Debug, Clone, TS, Eq, PartialEq, Hash)]
#[ts(export, export_to = "tauri.ts")]
pub struct Country {
    pub name: String,
    pub code: String,
}

impl fmt::Display for Country {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.name)
    }
}

impl Country {
    pub fn try_new_from_code(code: &str) -> Option<Self> {
        // XK (and its alias KS) is Kosovo — not in ISO 3166-1 but widely accepted (EU, CLDR, etc.)
        if matches!(code, "XK" | "KS") {
            return Some(Country {
                name: "Kosovo".to_string(),
                code: "XK".to_string(),
            });
        }
        rust_iso3166::from_alpha2(code).map(|country| Country {
            name: country.name.to_string(),
            code: country.alpha2.to_string(),
        })
    }
}
