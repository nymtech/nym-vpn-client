// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    path::Path,
    sync::{Arc, atomic::AtomicBool},
};

use tokio::sync::Mutex;

use crate::{
    adblocker::{AdBlockerError, Result, engines::AdBlockEngine},
    resolver::{DnsFilterDecision, DnsFilterT},
};

enum State {
    AlwaysOk,
    FailOnce(FailOncePromise),
}

#[derive(Default, Clone)]
pub struct FailOncePromise {
    state: Arc<AtomicBool>,
}

impl FailOncePromise {
    pub fn is_fulfilled(&self) -> bool {
        self.state.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn fulfill(&self) {
        self.state.store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

pub struct MockEngine {
    state: Arc<Mutex<State>>,
}

impl Default for MockEngine {
    fn default() -> Self {
        Self::new(State::AlwaysOk)
    }
}

impl MockEngine {
    /// Creates a new `MockEngine` that will fail once on `load_filters`.
    pub fn fail_once() -> (Self, FailOncePromise) {
        let promise = FailOncePromise::default();
        (Self::new(State::FailOnce(promise.clone())), promise)
    }

    fn new(state: State) -> Self {
        Self {
            state: Arc::new(Mutex::new(state)),
        }
    }
}

#[async_trait::async_trait]
impl AdBlockEngine for MockEngine {
    async fn load_filters(&self, dir: &Path) -> Result<()> {
        let state_guard = self.state.lock().await;

        match &*state_guard {
            State::AlwaysOk => Ok(()),
            State::FailOnce(promise) => {
                if promise.is_fulfilled() {
                    Ok(())
                } else {
                    promise.fulfill();

                    Err(AdBlockerError::OpenFile {
                        file_path: dir.to_path_buf(),
                        error: std::io::Error::from(std::io::ErrorKind::PermissionDenied),
                    })
                }
            }
        }
    }

    async fn unload_filters(&self) {
        // no-op
    }
}

#[async_trait::async_trait]
impl DnsFilterT for MockEngine {
    async fn should_block(&self, _domain: &str) -> DnsFilterDecision {
        DnsFilterDecision::Pass
    }
}
