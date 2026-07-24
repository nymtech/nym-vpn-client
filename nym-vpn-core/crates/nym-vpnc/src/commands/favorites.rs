// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::anyhow;
use nym_favorites::FavoritesManager;
use nym_vpn_lib_types::NodeIdentity;

use crate::fs::app_data_dir;

#[derive(Debug, Clone, clap::Subcommand)]
pub enum FavoriteSelector {
    /// Mixnet public ID of the gateway.
    Id {
        /// Mixnet public ID of the gateway.
        #[arg(index = 1)]
        id: String,
    },

    /// Country ISO.
    Country {
        /// Country ISO.
        #[arg(index = 1)]
        country: celes::Country,
    },

    /// Region.
    Region {
        /// Region.
        #[arg(index = 1)]
        region: String,
    },
}

impl TryFrom<FavoriteSelector> for nym_vpn_lib_types::FavoriteSelector {
    type Error = anyhow::Error;

    fn try_from(value: FavoriteSelector) -> Result<Self, Self::Error> {
        let ret = match value {
            FavoriteSelector::Id { id } => Self::Gateway {
                identity: NodeIdentity::from_base58_string(id)
                    .map_err(|_| anyhow!("Failed to parse gateway id"))?,
            },
            FavoriteSelector::Country { country } => Self::Country {
                two_letter_iso_country_code: country.alpha2.to_string(),
            },
            FavoriteSelector::Region { region } => Self::Region { region },
        };
        Ok(ret)
    }
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    /// Get favorite selectors
    Get,

    /// Set a favorite entry selector
    #[command(subcommand)]
    SetEntry(FavoriteSelector),

    /// Set a favorite exit selector
    #[command(subcommand)]
    SetExit(FavoriteSelector),

    /// Remove a favorite entry selector
    #[command(subcommand)]
    RemoveEntry(FavoriteSelector),

    /// Remove a favorite exit selector
    #[command(subcommand)]
    RemoveExit(FavoriteSelector),
}

impl Command {
    pub async fn execute(self) -> anyhow::Result<()> {
        let Some(data_dir) = app_data_dir().await else {
            return Err(anyhow::anyhow!("Could not get app data directory"));
        };
        let mut favorites_manager = FavoritesManager::new(data_dir).await;
        match self {
            Command::Get => {
                let current_favorites = favorites_manager.get_favorites();
                println!("Favorites:\n{current_favorites}");
                return Ok(());
            }
            Command::SetEntry(selector) => {
                favorites_manager
                    .add_favorite_entry(selector.try_into()?)
                    .await?
            }
            Command::SetExit(selector) => {
                favorites_manager
                    .add_favorite_exit(selector.try_into()?)
                    .await?
            }
            Command::RemoveEntry(selector) => {
                favorites_manager
                    .remove_favorite_entry(selector.try_into()?)
                    .await?
            }
            Command::RemoveExit(selector) => {
                favorites_manager
                    .remove_favorite_exit(selector.try_into()?)
                    .await?
            }
        }
        Ok(())
    }
}
