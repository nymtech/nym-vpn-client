use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fmt;
use ts_rs::TS;

pub static FASTEST_NODE_LOCATION: Lazy<Country> = Lazy::new(|| Country {
    code: String::from("DE"),
    name: String::from("Germany"),
});

// When the app receives the countries data, the selected countries
// are checked against the available countries, and if needed changed to
// available ones (logic handled by the frontend)
pub static DEFAULT_ENTRY_COUNTRY: Lazy<Country> = Lazy::new(|| Country {
    code: String::from("CH"),
    name: String::from("Switzerland"),
});

#[derive(Serialize, Deserialize, Debug, Clone, TS, Eq, PartialEq, Hash)]
#[ts(export)]
pub struct Country {
    pub name: String,
    pub code: String,
}

impl fmt::Display for Country {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Country: {} [{}]", self.name, self.code)
    }
}

impl Default for Country {
    fn default() -> Self {
        DEFAULT_ENTRY_COUNTRY.clone()
    }
}

impl Country {
    pub fn try_new_from_code(code: &str) -> Option<Self> {
        rust_iso3166::from_alpha2(code).map(|country| Country {
            name: country.name.to_string(),
            code: country.alpha2.to_string(),
        })
    }
}
