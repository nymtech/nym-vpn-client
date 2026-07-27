// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use futures::{FutureExt, TryFutureExt, future::BoxFuture};
use tokio::sync::Mutex;
use tokio_util::sync::{CancellationToken, DropGuard};

#[cfg(test)]
use crate::adblocker::state::PrimitiveState;
use crate::{
    adblocker::{
        Result,
        engines::{AdBlockEngine, AdBlockEngineWrap},
        file_manager::{SOURCES, init_files},
        state::{ObservableState, State},
    },
    dns_filter::DnsFilter,
};

#[cfg(not(target_os = "ios"))]
use crate::adblocker::engines::BraveAdblockEngine;
#[cfg(target_os = "ios")]
use crate::adblocker::engines::SimpleAdBlockEngine;

use nym_file_updater::{FileUpdaterError, FileUpdaterHandle, UpdateOutcome};
use tokio::sync::mpsc;
use url::Url;

const ADBLOCK_INITIAL_UPDATE_DELAY: Duration = Duration::from_mins(5);
const ADBLOCK_UPDATE_INTERVAL: Duration = Duration::from_hours(8);

type AdBlockEngineRef = Arc<AdBlockEngineWrap>;
type UpdateReceiver = mpsc::UnboundedReceiver<Result<UpdateOutcome, FileUpdaterError>>;

pub struct AdBlocker {
    state: Arc<Mutex<ObservableState>>,
    engine: AdBlockEngineRef,
    cache_dir: PathBuf,
    file_updater_handle: FileUpdaterHandle,
    shutdown_token: CancellationToken,
    _shutdown_drop_guard: DropGuard,
}

impl AdBlocker {
    pub fn new(cache_dir: PathBuf, file_updater_handle: FileUpdaterHandle) -> Self {
        #[cfg(not(target_os = "ios"))]
        let engine = AdBlockEngineWrap::Brave(BraveAdblockEngine::default());
        #[cfg(target_os = "ios")]
        let engine =
            AdBlockEngineWrap::Simple(SimpleAdBlockEngine::new(cache_dir.join("adblock.db")));

        Self::create(
            cache_dir,
            ObservableState::default(),
            Arc::new(engine),
            file_updater_handle,
        )
    }

    fn create(
        cache_dir: PathBuf,
        initial_state: ObservableState,
        engine: AdBlockEngineRef,
        file_updater_handle: FileUpdaterHandle,
    ) -> Self {
        assert!(matches!(initial_state.get(), State::Disabled));

        let shutdown_token = CancellationToken::new();
        let state = Arc::new(Mutex::new(initial_state));

        Self {
            state,
            engine,
            cache_dir,
            file_updater_handle,
            shutdown_token: shutdown_token.clone(),
            _shutdown_drop_guard: shutdown_token.drop_guard(),
        }
    }

    /// Shutdown the ad-blocker waiting for background tasks to complete.
    pub async fn stop(self) {
        tracing::debug!("Stopping ad-blocker");
        self.shutdown_token.cancel();
        self.disable().await;
    }

    pub async fn enable(&self) {
        tracing::trace!("Enable ad-blocker");

        let mut state = self.state.lock().await;

        if let State::Disabled = state.get() {
            tracing::debug!("Ad-blocker initializing");

            let cancel_token = self.shutdown_token.child_token();
            let task = tokio::spawn(handle_init(
                self.state.clone(),
                self.engine.clone(),
                self.cache_dir.clone(),
                self.file_updater_handle.clone(),
                cancel_token.child_token(),
            ));

            state.replace(State::Initializing { task, cancel_token });
        }
    }

    pub async fn disable(&self) {
        tracing::trace!("Disable ad-blocker");

        let mut state = self.state.lock().await;
        let old_state = state.replace(State::Disabled);

        match old_state {
            State::Enabled {
                task, cancel_token, ..
            }
            | State::Updating {
                task, cancel_token, ..
            } => {
                unload_filters(&self.engine).await;
                cancel_token.cancel();

                drop(state);
                let _ = task.await;
            }
            State::Initializing {
                task, cancel_token, ..
            } => {
                cancel_token.cancel();
                drop(state);
                let _ = task.await;
            }
            State::Disabled => {}
        }

        tracing::debug!("Ad-blocker disabled");
    }

    pub fn get_dns_filter(&self) -> DnsFilter {
        self.engine.clone()
    }
}

async fn handle_init(
    state: Arc<Mutex<ObservableState>>,
    engine: AdBlockEngineRef,
    cache_dir: PathBuf,
    file_updater_handle: FileUpdaterHandle,
    cancel_token: CancellationToken,
) {
    if let Err(err) = init_files(&cache_dir, false).await {
        tracing::error!("Failed to initialize ad-blocker: {err}");
        let mut state_guard = state.lock().await;
        let State::Initializing { .. } = state_guard.get() else {
            return;
        };
        state_guard.replace(State::Disabled);
        return;
    }

    let mut state_guard = state.lock().await;

    if cancel_token.is_cancelled() {
        return;
    }

    let State::Initializing { .. } = state_guard.get() else {
        return;
    };

    tracing::debug!("Ad-blocker was initialized successfully");

    let res = reload_filters(&engine, &cache_dir)
        .or_else(|err| {
            tracing::error!("Failed to load filter set: {err}");
            tracing::debug!("Retrying ad-blocker initialization with builtin data");

            init_files(&cache_dir, true)
                .inspect_err(|err| {
                    tracing::error!("Failed to re-initialize ad-blocker: {err}");
                    tracing::error!(
                        "Ad-blocker initialization has failed twice, so will remain disabled!"
                    );
                })
                .and_then(|_| {
                    reload_filters(&engine, &cache_dir).inspect_err(|err| {
                        tracing::error!(
                            "Failed to load filter set after force initialization: {err}"
                        );
                        tracing::error!(
                            "Ad-blocker has failed twice to reload filters, so will remain disabled!"
                        );
                    })
                })
        })
        .await;

    let new_state = match res {
        Ok(()) => {
            // Register each source with the updater for periodic updates.
            let mut receivers = Vec::new();
            for source in SOURCES.iter() {
                let Ok(url) = source.url.parse::<Url>() else {
                    tracing::error!("Invalid ad-blocker source URL: {}", source.url);
                    continue;
                };
                let dest_path = cache_dir.join(source.file_name);
                match file_updater_handle
                    .register(
                        url,
                        dest_path,
                        ADBLOCK_INITIAL_UPDATE_DELAY,
                        ADBLOCK_UPDATE_INTERVAL,
                    )
                    .await
                {
                    Ok(rx) => receivers.push(rx),
                    Err(err) => {
                        tracing::error!(
                            "Failed to register ad-blocker source {} with updater: {err}",
                            source.file_name
                        );
                    }
                }
            }

            let task = tokio::spawn(wait_for_update(
                state.clone(),
                engine,
                cache_dir,
                receivers,
                cancel_token.child_token(),
            ));

            State::Enabled { task, cancel_token }
        }
        Err(_err) => State::Disabled,
    };

    state_guard.replace(new_state);
}

/// Wait for any source to report `UpdateOutcome::Updated` then hand off to
/// [`handle_filter_reload`], which reloads the engine and comes back here.
fn wait_for_update(
    state: Arc<Mutex<ObservableState>>,
    engine: AdBlockEngineRef,
    cache_dir: PathBuf,
    mut receivers: Vec<UpdateReceiver>,
    cancel_token: CancellationToken,
) -> BoxFuture<'static, ()> {
    async move {
        loop {
            let outcome = recv_any_update(&mut receivers, &cancel_token).await;

            match outcome {
                Some(Ok(UpdateOutcome::Updated)) => {
                    let mut state_guard = state.lock().await;
                    let State::Enabled { .. } = state_guard.get() else {
                        return;
                    };

                    let task = tokio::spawn(handle_filter_reload(
                        state.clone(),
                        engine.clone(),
                        cache_dir.clone(),
                        receivers,
                        cancel_token.child_token(),
                    ));
                    state_guard.replace(State::Updating { task, cancel_token });
                    return;
                }
                Some(Ok(UpdateOutcome::NotModified)) => {
                    // Nothing to do — file unchanged.
                }
                Some(Err(err)) => {
                    tracing::error!("Ad-blocker updater error: {err}");
                }
                None => {
                    // All receivers closed (updater shut down or cancelled).
                    return;
                }
            }
        }
    }
    .boxed()
}

/// Reload filters from disk and transition back to [`State::Enabled`].
fn handle_filter_reload(
    state: Arc<Mutex<ObservableState>>,
    engine: AdBlockEngineRef,
    cache_dir: PathBuf,
    receivers: Vec<UpdateReceiver>,
    cancel_token: CancellationToken,
) -> BoxFuture<'static, ()> {
    async move {
        if let Err(err) = reload_filters(&engine, &cache_dir).await {
            tracing::error!("Failed to reload ad-blocker filters: {err}");
            // Continue anyway — keep running with stale rules rather than disabling.
        } else {
            tracing::debug!("Ad-blocker filters reloaded successfully");
        }

        let mut state_guard = state.lock().await;
        let State::Updating { .. } = state_guard.get() else {
            return;
        };

        let task = tokio::spawn(wait_for_update(
            state.clone(),
            engine,
            cache_dir,
            receivers,
            cancel_token.child_token(),
        ));
        state_guard.replace(State::Enabled { task, cancel_token });
    }
    .boxed()
}

/// Poll all receivers concurrently; return the first message that arrives.
/// Returns `None` when all receivers are closed or the token is cancelled.
async fn recv_any_update(
    receivers: &mut Vec<UpdateReceiver>,
    cancel_token: &CancellationToken,
) -> Option<Result<UpdateOutcome, FileUpdaterError>> {
    loop {
        if receivers.is_empty() {
            return None;
        }

        // Build a vec of recv() futures and race them.
        // We use `futures::future::select_all` to pick the first ready one.
        let futs: Vec<_> = receivers.iter_mut().map(|rx| Box::pin(rx.recv())).collect();
        let result = tokio::select! {
            _ = cancel_token.cancelled() => return None,
            (outcome, _idx, _rest) = futures::future::select_all(futs) => outcome,
        };

        match result {
            Some(msg) => return Some(msg),
            None => {
                // A receiver closed — remove it and keep waiting on the rest.
                receivers.retain_mut(|rx| !rx.is_closed());
                if receivers.is_empty() {
                    return None;
                }
            }
        }
    }
}

async fn reload_filters(adblocker: &AdBlockEngineRef, cache_dir: &Path) -> Result<()> {
    adblocker.load_filters(cache_dir).await?;

    #[cfg(not(any(target_os = "android", target_os = "ios", test)))]
    crate::resolver::flush_system_cache().await;

    Ok(())
}

async fn unload_filters(adblocker: &AdBlockEngineRef) {
    adblocker.unload_filters().await;

    #[cfg(not(any(target_os = "android", target_os = "ios", test)))]
    crate::resolver::flush_system_cache().await;
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use super::*;
    use crate::adblocker::engines::MockEngine;

    #[tokio::test]
    #[traced_test]
    async fn test_state_transitions() {
        let temp_dir = tempfile::tempdir().unwrap();
        let engine = MockEngine::default();

        let (state_tx, mut state_rx) = tokio::sync::mpsc::unbounded_channel();

        let adblocker = AdBlocker::create(
            temp_dir.path().to_path_buf(),
            ObservableState::new_with_observer(state_tx),
            Arc::new(AdBlockEngineWrap::Mock(engine)),
            FileUpdaterHandle::new_test(),
        );
        adblocker.enable().await;

        assert!(matches!(
            wait_state(&mut state_rx).await.unwrap(),
            PrimitiveState::Initializing
        ));
        assert!(matches!(
            wait_state(&mut state_rx).await.unwrap(),
            PrimitiveState::Enabled
        ));

        adblocker.disable().await;
        assert!(matches!(
            wait_state(&mut state_rx).await.unwrap(),
            PrimitiveState::Disabled
        ));
    }

    #[tokio::test]
    #[traced_test]
    async fn test_reset_store_on_load_filters_failure() {
        let temp_dir = tempfile::tempdir().unwrap();
        let (engine, promise) = MockEngine::fail_once();

        let (state_tx, mut state_rx) = tokio::sync::mpsc::unbounded_channel();

        let adblocker = AdBlocker::create(
            temp_dir.path().to_path_buf(),
            ObservableState::new_with_observer(state_tx),
            Arc::new(AdBlockEngineWrap::Mock(engine)),
            FileUpdaterHandle::new_test(),
        );
        adblocker.enable().await;

        assert!(matches!(
            wait_state(&mut state_rx).await.unwrap(),
            PrimitiveState::Initializing
        ));
        assert!(matches!(
            wait_state(&mut state_rx).await.unwrap(),
            PrimitiveState::Enabled
        ));

        assert!(promise.is_fulfilled());
    }

    async fn wait_state(
        state_rx: &mut tokio::sync::mpsc::UnboundedReceiver<PrimitiveState>,
    ) -> Option<PrimitiveState> {
        tokio::time::timeout(Duration::from_secs(1), state_rx.recv())
            .await
            .ok()?
    }
}
