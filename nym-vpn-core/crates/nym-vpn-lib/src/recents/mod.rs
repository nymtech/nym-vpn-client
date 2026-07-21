// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{collections::VecDeque, path::PathBuf, str::FromStr, sync::Arc};

use itertools::Itertools;
use nym_gateway_directory::{GatewayList, GatewayType, NodeIdentity};
use nym_vpn_lib_types::{Gateway, RecentGateways, TunnelType};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

pub use crate::recents::gateway_cache::RecentGatewayCache;

mod gateway_cache;

const MAX_RECENTS: usize = 20;
const RECENTS_FILE_NAME: &str = "recents.json";

#[derive(Clone, Serialize, Deserialize)]
struct InnerRecents {
    entry: VecDeque<String>,
    exit: VecDeque<String>,
}

impl InnerRecents {
    fn truncate(&mut self) {
        if self.entry.len() > MAX_RECENTS {
            let _ = self.entry.split_off(MAX_RECENTS);
        }
        if self.exit.len() > MAX_RECENTS {
            let _ = self.exit.split_off(MAX_RECENTS);
        }
    }
}

impl Default for InnerRecents {
    fn default() -> Self {
        Self {
            entry: VecDeque::with_capacity(MAX_RECENTS),
            exit: VecDeque::with_capacity(MAX_RECENTS),
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
struct Recents {
    mixnet_recents: InnerRecents,
    wireguard_recents: InnerRecents,
}

impl Recents {
    fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    fn truncate(&mut self) {
        self.mixnet_recents.truncate();
        self.wireguard_recents.truncate();
    }
}

/// Manager for storing the recent successful gateway connections to disk and in a memory cache
#[derive(Clone)]
pub struct RecentsManager<C: RecentGatewayCache> {
    file_path: PathBuf,
    gateway_cache: C,
    cache: Arc<RwLock<Recents>>,
}

async fn persisted_recents(file_path: &PathBuf) -> Option<Recents> {
    let bytes = tokio::fs::read(file_path)
        .await
        .inspect_err(|err| tracing::warn!("Could not load disk recents, clearing it out: {err:?}"))
        .ok()?;
    serde_json::from_slice(&bytes)
        .inspect_err(|err| tracing::warn!("Could not decode recents, clearing it out: {err:?}"))
        .ok()
}

async fn flush(file_path: PathBuf, contents: Vec<u8>) {
    if let Err(err) = tokio::fs::write(file_path, contents).await {
        tracing::warn!("Could not flush recents file: {err:?}");
    } else {
        tracing::debug!("Recents file written to disk");
    }
}

impl<C: RecentGatewayCache> RecentsManager<C> {
    pub async fn new(dir_path: PathBuf, gateway_cache: C) -> Self {
        let file_path = dir_path.join(RECENTS_FILE_NAME);
        match persisted_recents(&file_path).await {
            Some(mut cache) => {
                // in case disk got more entries than current maximum, we truncate that to the current max value
                cache.truncate();
                Self {
                    file_path,
                    gateway_cache,
                    cache: Arc::new(RwLock::new(cache)),
                }
            }
            None => {
                let cache = Recents::default();
                if let Ok(contents) = cache.to_bytes() {
                    // write the first empty recents, to have the file created on disk
                    flush(file_path.clone(), contents).await;
                } else {
                    tracing::warn!("Could not serialize empty recents cache");
                };
                Self {
                    file_path,
                    gateway_cache,
                    cache: Arc::new(RwLock::new(cache)),
                }
            }
        }
    }

    fn add_recent_to_queue(queue: &mut VecDeque<String>, recent: String) {
        if let Some((idx, _)) = queue.iter().find_position(|v| **v == recent) {
            if idx == 0 {
                // already the most recent, nothing to do
                return;
            }
            queue.remove(idx);
        } else if queue.len() >= MAX_RECENTS {
            queue.pop_back();
        }
        queue.push_front(recent);
    }

    pub async fn add_recent(&mut self, tunnel_type: TunnelType, entry: String, exit: String) {
        let mut cache: tokio::sync::RwLockWriteGuard<'_, Recents> = self.cache.write().await;
        let inner = match tunnel_type {
            TunnelType::Mixnet => &mut cache.mixnet_recents,
            TunnelType::Wireguard => &mut cache.wireguard_recents,
        };
        Self::add_recent_to_queue(&mut inner.entry, entry);
        Self::add_recent_to_queue(&mut inner.exit, exit);

        let Ok(contents) = cache.to_bytes() else {
            tracing::warn!("Could not serialize recents cache");
            return;
        };
        let file_path = self.file_path.clone();
        // put IO disk operation on a new thread, as it's a best effort sync to disk that's not needed
        // for the hot path of going into connected state
        tokio::spawn(flush(file_path, contents));
    }

    fn get_recent_queue(queue: &VecDeque<String>, gateways: &GatewayList) -> Vec<Gateway> {
        queue
            .iter()
            .filter_map(|gw_str| NodeIdentity::from_str(gw_str).ok())
            .filter_map(|id| gateways.gateway_with_identity(&id))
            .cloned()
            .map(Into::into)
            .collect()
    }

    pub async fn get_recent(
        &self,
        tunnel_type: TunnelType,
    ) -> Result<RecentGateways, crate::gateway_directory::Error> {
        let cache = self.cache.read().await.clone();
        let (inner, entry_gateways, exit_gateways) = match tunnel_type {
            TunnelType::Mixnet => (
                cache.mixnet_recents,
                self.gateway_cache
                    .lookup_gateways(GatewayType::MixnetEntry)
                    .await?,
                self.gateway_cache
                    .lookup_gateways(GatewayType::MixnetExit)
                    .await?,
            ),
            TunnelType::Wireguard => {
                let wg_gateways = self.gateway_cache.lookup_gateways(GatewayType::Wg).await?;
                (cache.wireguard_recents, wg_gateways.clone(), wg_gateways)
            }
        };
        let entry = Self::get_recent_queue(&inner.entry, &entry_gateways);
        let exit = Self::get_recent_queue(&inner.exit, &exit_gateways);
        Ok(RecentGateways { entry, exit })
    }
}
