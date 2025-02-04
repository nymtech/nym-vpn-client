use serde::{Deserialize, Serialize};
use std::fmt;
use ts_rs::TS;

#[derive(Serialize, Deserialize, Debug, Clone, TS, Eq, PartialEq, Hash)]
#[ts(export)]
pub struct Country {
    pub name: String,
    pub code: String,
}

impl fmt::Display for Country {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Country: [{}] {}", self.code, self.name)
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
