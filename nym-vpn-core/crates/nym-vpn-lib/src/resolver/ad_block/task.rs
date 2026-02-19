// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::{AdBlockError, AdBlocker, Result};
use crate::resolver::ResolverMessage;
use std::path::{Path, PathBuf};
use tokio::sync::{mpsc, oneshot, Mutex};

pub struct AdBlockerTask {
    data_dir: PathBuf,
    adblocker: Mutex<Option<AdBlocker>>,
}

impl AdBlockerTask {
    pub async fn spawn(data_dir: &Path) -> Result<Self> {
        Ok(Self {
            data_dir: data_dir.to_path_buf(),
            adblocker: Mutex::new(None),
        })
    }
}

enum AdBlockerTaskMessage {
    /// Enable Ad-blocker.
    EnableAdBlocker {
        /// Response channel when resolvers have been updated
        response_tx: oneshot::Sender<()>,
    },

    /// Disable Ad-blocker.
    DisableAdBlocker {
        /// Response channel when resolvers have been updated
        response_tx: oneshot::Sender<()>,
    },

    /// Ad-blocker initialized in the background
    AdBlockerInitted {
        result: Result<AdBlocker, AdBlockError>,
        retry_count: usize,
    },

    /// Ad-blocker updated in the background
    /// (it may not have actually updated if the data files didn't change)
    AdBlockerUpdated {
        result: Result<Option<AdBlocker>, AdBlockError>,
    },
}

/// A handle to control the Ad-blocker task.
///
/// When all resolver handles are dropped, the resolver will stop.
#[derive(Clone)]
pub struct AdBlockerTaskHandle {
    tx: mpsc::UnboundedSender<AdBlockerTaskMessage>,
}

impl AdBlockerTaskHandle {
    fn new(tx: mpsc::UnboundedSender<AdBlockerTaskMessage>) -> Self {
        Self { tx }
    }

    /// Enable Ad-blocker.
    pub async fn enable_ad_blocker(&self) {
        let (response_tx, response_rx) = oneshot::channel();
        if self
            .tx
            .send(AdBlockerTaskMessage::EnableAdBlocker { response_tx })
            .is_ok()
        {
            response_rx.await.ok();
        }
    }

    /// Disable Ad-blocker.
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub async fn disable_ad_blocker(&self) {
        let (response_tx, response_rx) = oneshot::channel();
        if self
            .tx
            .send(AdBlockerTaskMessage::DisableAdBlocker { response_tx })
            .is_ok()
        {
            response_rx.await.ok();
        }
    }
}
