use nym_vpn_lib_types as lib;
use tauri::State;
use tracing::{debug, info, instrument};

use crate::commands::gateway::Hop;
use crate::error::BackendError;
use crate::favorites::{Favorite, Favorites, FavoritesState};

fn to_selector(favorite: Favorite) -> Result<lib::FavoriteSelector, BackendError> {
    lib::FavoriteSelector::try_from(favorite)
        .map_err(|e| BackendError::internal_with_detail("invalid favorite selector", e.to_string()))
}

fn unavailable() -> BackendError {
    BackendError::internal("favorites store is unavailable", None)
}

#[instrument(skip(favorites))]
#[tauri::command]
pub async fn get_favorites(
    favorites: State<'_, FavoritesState>,
) -> Result<Favorites, BackendError> {
    let guard = favorites.0.lock().await;
    // A missing store is reported as an empty set rather than an error
    let Some(manager) = guard.as_ref() else {
        info!("favorites store unavailable, reporting empty");
        return Ok(Favorites::default());
    };
    let favorites = Favorites::from(manager.get_favorites());
    debug!(
        "favorites: entry #{} exit #{}",
        favorites.entry.len(),
        favorites.exit.len()
    );
    Ok(favorites)
}

#[instrument(skip(favorites))]
#[tauri::command]
pub async fn add_favorite(
    hop: Hop,
    favorite: Favorite,
    favorites: State<'_, FavoritesState>,
) -> Result<(), BackendError> {
    info!("adding favorite {favorite} for {hop:?}");
    let selector = to_selector(favorite)?;
    let mut guard = favorites.0.lock().await;
    let manager = guard.as_mut().ok_or_else(unavailable)?;

    match hop {
        Hop::Entry => manager.add_favorite_entry(selector).await,
        Hop::Exit => manager.add_favorite_exit(selector).await,
    }
    .map_err(|e| BackendError::internal_with_detail("failed to add favorite", e.to_string()))
}

#[instrument(skip(favorites))]
#[tauri::command]
pub async fn remove_favorite(
    hop: Hop,
    favorite: Favorite,
    favorites: State<'_, FavoritesState>,
) -> Result<(), BackendError> {
    info!("removing favorite {favorite} for {hop:?}");
    let selector = to_selector(favorite)?;
    let mut guard = favorites.0.lock().await;
    let manager = guard.as_mut().ok_or_else(unavailable)?;

    match hop {
        Hop::Entry => manager.remove_favorite_entry(selector).await,
        Hop::Exit => manager.remove_favorite_exit(selector).await,
    }
    .map_err(|e| BackendError::internal_with_detail("failed to remove favorite", e.to_string()))
}
