use std::fmt;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

// TODO use hardcoded country list for now
pub static COUNTRIES: Lazy<Vec<Country>> = Lazy::new(|| {
    vec![
        Country {
            name: "France".to_string(),
            code: "FR".to_string(),
        },
        Country {
            name: "Germany".to_string(),
            code: "DE".to_string(),
        },
        Country {
            name: "Ireland".to_string(),
            code: "IE".to_string(),
        },
        Country {
            name: "Japan".to_string(),
            code: "JP".to_string(),
        },
        Country {
            name: "United Kingdom".to_string(),
            code: "GB".to_string(),
        },
    ]
});

pub static FASTEST_NODE_LOCATION: Lazy<Country> = Lazy::new(|| Country {
    code: "DE".to_string(),
    name: "Germany".to_string(),
});

pub const DEFAULT_COUNTRY_CODE: &str = "FR";

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub struct Country {
    pub name: String,
    pub code: String,
}

// retrieve a country from two letters code
impl TryFrom<&str> for Country {
    type Error = String;

    fn try_from(code: &str) -> Result<Self, Self::Error> {
        let country = COUNTRIES
            .iter()
            .find(|c| c.code == code)
            .ok_or(format!("No matching country for code [{code}]"))?;
        Ok(country.clone())
    }
}

impl fmt::Display for Country {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Country: {} [{}]", self.name, self.code)
    }
}

impl Default for Country {
    fn default() -> Self {
        Country {
            name: "France".to_string(),
            code: "FR".to_string(),
        }
    }
}
