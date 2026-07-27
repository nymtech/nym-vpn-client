use std::str::FromStr;

use nym_favorites::FavoritesManager;
use nym_vpn_lib_types::{FavoriteSelector, FavoriteSelectors, NodeIdentity};
use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::sync::Mutex;
use tracing::instrument;
use ts_rs::TS;

use crate::error::BackendError;

/// What a favorite points at. `value` is a gateway identity, an ISO-3166
/// two-letter country code, or a region name depending on `kind`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "tauri.ts")]
pub enum FavoriteKind {
    Gateway,
    Country,
    Region,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "tauri.ts")]
pub struct Favorite {
    pub kind: FavoriteKind,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "tauri.ts")]
pub struct Favorites {
    pub entry: Vec<Favorite>,
    pub exit: Vec<Favorite>,
}

/// The hop a favorite applies to. Favorites are independent per hop.
#[derive(Debug, Clone, Copy, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "tauri.ts")]
pub enum FavoriteHop {
    Entry,
    Exit,
}

impl From<&FavoriteSelector> for Favorite {
    fn from(selector: &FavoriteSelector) -> Self {
        match selector {
            FavoriteSelector::Gateway { identity } => Favorite {
                kind: FavoriteKind::Gateway,
                value: identity.to_string(),
            },
            FavoriteSelector::Country {
                two_letter_iso_country_code,
            } => Favorite {
                kind: FavoriteKind::Country,
                value: two_letter_iso_country_code.clone(),
            },
            FavoriteSelector::Region { region } => Favorite {
                kind: FavoriteKind::Region,
                value: region.clone(),
            },
        }
    }
}

impl From<FavoriteSelectors> for Favorites {
    fn from(selectors: FavoriteSelectors) -> Self {
        Favorites {
            entry: selectors.entry.iter().map(Favorite::from).collect(),
            exit: selectors.exit.iter().map(Favorite::from).collect(),
        }
    }
}

impl Favorite {
    fn into_selector(self) -> Result<FavoriteSelector, BackendError> {
        Ok(match self.kind {
            FavoriteKind::Gateway => FavoriteSelector::Gateway {
                identity: NodeIdentity::from_str(&self.value).map_err(|e| {
                    BackendError::internal(&format!("invalid gateway identity: {e}"), None)
                })?,
            },
            FavoriteKind::Country => FavoriteSelector::Country {
                two_letter_iso_country_code: self.value,
            },
            FavoriteKind::Region => FavoriteSelector::Region { region: self.value },
        })
    }
}

#[instrument(skip(favorites))]
#[tauri::command]
pub async fn favorites_get(
    favorites: State<'_, Mutex<FavoritesManager>>,
) -> Result<Favorites, BackendError> {
    Ok(favorites.lock().await.get_favorites().into())
}

#[instrument(skip(favorites))]
#[tauri::command]
pub async fn add_favorite(
    favorites: State<'_, Mutex<FavoritesManager>>,
    hop: FavoriteHop,
    favorite: Favorite,
) -> Result<Favorites, BackendError> {
    let selector = favorite.into_selector()?;
    let mut manager = favorites.lock().await;
    match hop {
        FavoriteHop::Entry => manager.add_favorite_entry(selector).await,
        FavoriteHop::Exit => manager.add_favorite_exit(selector).await,
    }
    .map_err(|e| BackendError::internal(&format!("failed to add favorite: {e}"), None))?;
    Ok(manager.get_favorites().into())
}

#[instrument(skip(favorites))]
#[tauri::command]
pub async fn remove_favorite(
    favorites: State<'_, Mutex<FavoritesManager>>,
    hop: FavoriteHop,
    favorite: Favorite,
) -> Result<Favorites, BackendError> {
    let selector = favorite.into_selector()?;
    let mut manager = favorites.lock().await;
    match hop {
        FavoriteHop::Entry => manager.remove_favorite_entry(selector).await,
        FavoriteHop::Exit => manager.remove_favorite_exit(selector).await,
    }
    .map_err(|e| BackendError::internal(&format!("failed to remove favorite: {e}"), None))?;
    Ok(manager.get_favorites().into())
}
