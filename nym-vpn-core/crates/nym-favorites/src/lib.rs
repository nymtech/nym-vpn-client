// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use nym_vpn_lib_types::{FavoriteSelector, FavoriteSelectors};

pub use error::FavoritesError;
pub use gateway_cache::RecentGatewayCache;
use io::{parse_from_file, save_to_file};
pub use recents::RecentsManager;

mod error;
mod gateway_cache;
pub(crate) mod io;
pub mod recents;

const FAVORITES_FILE_NAME: &str = "favorites.json";

pub struct FavoritesManager {
    file_path: PathBuf,
    cache: FavoriteSelectors,
}

impl FavoritesManager {
    pub async fn new(dir_path: PathBuf) -> Self {
        let file_path = dir_path.join(FAVORITES_FILE_NAME);

        let cache = match parse_from_file(&file_path).await {
            Some(cache) => cache,
            None => {
                let cache = FavoriteSelectors {
                    entry: Vec::new(),
                    exit: Vec::new(),
                };
                // flushing errors are logged, but they are not fatal on creating the manager
                let _ = save_to_file(&cache, &file_path).await;
                cache
            }
        };

        Self { file_path, cache }
    }

    fn add_favorite(list: &mut Vec<FavoriteSelector>, selector: FavoriteSelector) {
        if list.iter().find(|s| **s == selector).is_none() {
            list.push(selector);
        }
    }

    fn remove_favorite(list: &mut Vec<FavoriteSelector>, selector: FavoriteSelector) {
        if let Some(idx) = list.iter_mut().position(|s| *s == selector) {
            list.swap_remove(idx);
        }
    }

    pub async fn add_favorite_entry(
        &mut self,
        selector: FavoriteSelector,
    ) -> Result<(), FavoritesError> {
        let list = self.cache.entry.clone();
        Self::add_favorite(&mut list, selector);
        save_to_file(&self.cache, &self.file_path).await?;
        self.cache.entry = list;

        Ok(())
    }

    pub async fn remove_favorite_entry(
        &mut self,
        selector: FavoriteSelector,
    ) -> Result<(), FavoritesError> {
        let list = self.cache.entry.clone();
        Self::remove_favorite(&mut self.cache.entry, selector);
        save_to_file(&self.cache, &self.file_path).await?;
        self.cache.entry = list;

        Ok(())
    }

    pub async fn add_favorite_exit(
        &mut self,
        selector: FavoriteSelector,
    ) -> Result<(), FavoritesError> {
        let list = self.cache.exit.clone();
        Self::add_favorite(&mut self.cache.exit, selector);
        save_to_file(&self.cache, &self.file_path).await?;
        self.cache.exit = list;

        Ok(())
    }

    pub async fn remove_favorite_exit(
        &mut self,
        selector: FavoriteSelector,
    ) -> Result<(), FavoritesError> {
        let list = self.cache.exit.clone();
        Self::remove_favorite(&mut self.cache.exit, selector);
        save_to_file(&self.cache, &self.file_path).await?;
        self.cache.exit = list;

        Ok(())
    }

    pub fn get_favorites(&self) -> FavoriteSelectors {
        self.cache.clone()
    }
}
