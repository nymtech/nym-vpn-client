use std::fmt;
use std::str::FromStr;

use nym_favorites::FavoritesManager;
use nym_vpn_lib_types as lib;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use ts_rs::TS;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "tauri.ts")]
pub enum Favorite {
    Country { code: String },
    Gateway { id: String },
    Region(String),
}

impl TryFrom<Favorite> for lib::FavoriteSelector {
    type Error = anyhow::Error;

    fn try_from(favorite: Favorite) -> Result<Self, Self::Error> {
        Ok(match favorite {
            Favorite::Country { code } => lib::FavoriteSelector::Country {
                two_letter_iso_country_code: code,
            },
            Favorite::Region(region) => lib::FavoriteSelector::Region { region },
            Favorite::Gateway { id } => lib::FavoriteSelector::Gateway {
                identity: lib::NodeIdentity::from_str(&id)?,
            },
        })
    }
}

impl From<lib::FavoriteSelector> for Favorite {
    fn from(selector: lib::FavoriteSelector) -> Self {
        match selector {
            lib::FavoriteSelector::Country {
                two_letter_iso_country_code: code,
            } => Favorite::Country { code },
            lib::FavoriteSelector::Region { region } => Favorite::Region(region),
            lib::FavoriteSelector::Gateway { identity } => Favorite::Gateway {
                id: identity.to_base58_string(),
            },
        }
    }
}

impl fmt::Display for Favorite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Favorite::Country { code } => write!(f, "country [{}]", code.to_uppercase()),
            Favorite::Region(region) => write!(f, "region {region}"),
            Favorite::Gateway { id } => write!(f, "gateway [{id}]"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, TS)]
#[ts(export, export_to = "tauri.ts")]
pub struct Favorites {
    pub entry: Vec<Favorite>,
    pub exit: Vec<Favorite>,
}

impl From<lib::FavoriteSelectors> for Favorites {
    fn from(selectors: lib::FavoriteSelectors) -> Self {
        Favorites {
            entry: selectors.entry.into_iter().map(Favorite::from).collect(),
            exit: selectors.exit.into_iter().map(Favorite::from).collect(),
        }
    }
}

pub struct FavoritesState(pub Mutex<Option<FavoritesManager>>);

impl FavoritesState {
    pub fn new(manager: Option<FavoritesManager>) -> Self {
        Self(Mutex::new(manager))
    }
}
