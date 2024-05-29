use std::fmt;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub static FASTEST_NODE_LOCATION: Lazy<Country> = Lazy::new(|| Country {
    code: String::from("DE"),
    name: String::from("Germany"),
});

// TODO use countries requested from the backend instead of hardcoded ones
pub static DEFAULT_ENTRY_COUNTRY: Lazy<Country> = Lazy::new(|| Country {
    code: String::from("CH"),
    name: String::from("Switzerland"),
});
pub static DEFAULT_EXIT_COUNTRY: Lazy<Country> = Lazy::new(|| Country {
    code: String::from("FR"),
    name: String::from("France"),
});

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
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
