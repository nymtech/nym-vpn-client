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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_a_valid_alpha2_code() {
        let country = Country::try_new_from_code("US").expect("US should resolve");
        assert_eq!(country.code, "US");
        assert!(!country.name.is_empty());
    }

    #[test]
    fn treats_xk_and_ks_as_kosovo() {
        for code in ["XK", "KS"] {
            let country = Country::try_new_from_code(code).expect("Kosovo should resolve");
            assert_eq!(country.code, "XK");
            assert_eq!(country.name, "Kosovo");
        }
    }

    #[test]
    fn returns_none_for_an_unknown_code() {
        assert!(Country::try_new_from_code("ZZ").is_none());
    }

    #[test]
    fn display_uses_the_code_and_name() {
        let country = Country {
            name: "Germany".to_string(),
            code: "DE".to_string(),
        };
        assert_eq!(country.to_string(), "[DE] Germany");
    }
}
