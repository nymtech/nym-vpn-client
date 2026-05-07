// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use futures::{FutureExt, TryFutureExt, future::BoxFuture};
use time::OffsetDateTime;
use tokio::sync::Mutex;
use tokio_util::sync::{CancellationToken, DropGuard};

#[cfg(test)]
use crate::adblocker::{engines::MockEngine, file_manager::MockFileManager, state::PrimitiveState};
use crate::{
    adblocker::{
        Result,
        engines::{AdBlockEngine, AdBlockEngineWrap},
        file_manager::{AdBlockFileManager, AdBlockFileManagerWrap, RealFileManager},
        state::{ObservableState, State},
    },
    dns_filter::DnsFilter,
};

#[cfg(not(target_os = "ios"))]
use crate::adblocker::engines::BraveAdblockEngine;
#[cfg(target_os = "ios")]
use crate::adblocker::engines::SimpleAdBlockEngine;

const INITIAL_ADBLOCK_UPDATE_DELAY: Duration = Duration::from_mins(2);
const ADBLOCK_UPDATE_DELAY: Duration = Duration::from_hours(1);

type AdBlockEngineRef = Arc<AdBlockEngineWrap>;
type FileManagerRef = Arc<AdBlockFileManagerWrap>;

pub struct AdBlocker {
    state: Arc<Mutex<ObservableState>>,
    engine: AdBlockEngineRef,
    file_manager: FileManagerRef,
    cache_dir: PathBuf,
    shutdown_token: CancellationToken,
    _shutdown_drop_guard: DropGuard,
}

impl AdBlocker {
    pub fn new(cache_dir: PathBuf, user_agent: String) -> Self {
        #[cfg(not(target_os = "ios"))]
        let engine = AdBlockEngineWrap::Brave(BraveAdblockEngine::default());
        #[cfg(target_os = "ios")]
        let engine =
            AdBlockEngineWrap::Simple(SimpleAdBlockEngine::new(cache_dir.join("adblock.db")));

        let file_manager =
            AdBlockFileManagerWrap::Real(RealFileManager::new(user_agent, cache_dir.clone()));
        let initial_state = ObservableState::default();

        Self::create(
            cache_dir,
            initial_state,
            Arc::new(engine),
            Arc::new(file_manager),
        )
    }

    fn create(
        cache_dir: PathBuf,
        initial_state: ObservableState,
        engine: AdBlockEngineRef,
        file_manager: FileManagerRef,
    ) -> Self {
        assert!(matches!(initial_state.get(), State::Disabled));

        let shutdown_token = CancellationToken::new();
        let state = Arc::new(Mutex::new(initial_state));

        Self {
            state,
            engine,
            file_manager,
            cache_dir,
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
                self.file_manager.clone(),
                self.cache_dir.clone(),
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

                // release the lock before awaiting the task
                drop(state);

                let _ = task.await;
            }
            State::Initializing {
                task, cancel_token, ..
            } => {
                cancel_token.cancel();
                drop(state);

                // release the lock before awaiting the task
                let _ = task.await;

                // no need to unload filters since the engine is not initialized yet
            }
            State::Disabled => {
                // ad-blocker is already disabled
            }
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
    file_manager: FileManagerRef,
    cache_dir: PathBuf,
    cancel_token: CancellationToken,
) {
    if let Err(err) = file_manager.init_files(false).await {
        tracing::error!("Failed to initialize ad-blocker: {err}");

        // Switch back to disabled state in case of error
        let mut state_guard = state.lock().await;
        let State::Initializing { .. } = state_guard.get() else {
            // ad-blocker is already disabled.
            return;
        };
        state_guard.replace(State::Disabled);
        // Explicit return
        return;
    }

    let mut state_guard = state.lock().await;

    // Return early if cancellation was requested
    if cancel_token.is_cancelled() {
        // State transition handled by caller
        return;
    }

    let State::Initializing { .. } = state_guard.get() else {
        // ad-blocker is already disabled.
        return;
    };

    tracing::debug!("Ad-blocker was initialized successfully");

    let res = reload_filters(&engine, &cache_dir)
        .or_else(|err| {
            tracing::error!("Failed to load filter set: {err}");
            tracing::debug!("Retrying ad-blocker initialization with builtin data");

            // If adblocker can't load filters, retry with builtin rules
            file_manager.init_files(true)
                .inspect_err(|err| {
                    tracing::error!("Failed to re-initialize ad-blocker: {err}");
                    tracing::error!(
                        "Ad-blocker initialization has failed twice, so will remain disabled!"
                    );
                })
                .and_then(|_| {
                    reload_filters(&engine, &cache_dir).inspect_err(|err| {
                        tracing::error!("Failed to load filter set after force initialization: {err}");
                        tracing::error!(
                            "Ad-blocker has failed twice to reload filters, so will remain disabled!"
                        );
                    })
                })
        })
        .await;

    let new_state = match res {
        Ok(()) => {
            // Schedule next update if adblocker is working.
            let task = tokio::spawn(schedule_next_update(
                state.clone(),
                engine,
                file_manager,
                cache_dir,
                OffsetDateTime::now_utc(),
                INITIAL_ADBLOCK_UPDATE_DELAY,
                cancel_token.child_token(),
            ));

            State::Enabled { task, cancel_token }
        }
        Err(_err) => State::Disabled,
    };

    state_guard.replace(new_state);
}

fn schedule_next_update(
    state: Arc<Mutex<ObservableState>>,
    engine: AdBlockEngineRef,
    file_manager: FileManagerRef,
    cache_dir: PathBuf,
    current_update_at: OffsetDateTime,
    next_update_after: Duration,
    cancel_token: CancellationToken,
) -> BoxFuture<'static, ()> {
    async move {
        let next_update_due = current_update_at + next_update_after;

        tracing::trace!("Next Ad-blocker update due at {:?}", next_update_due);

        tokio::select! {
            _ = tokio::time::sleep(next_update_after) => {
                let mut state_guard = state.lock().await;

                if let State::Enabled { .. } = state_guard.get() {
                    tracing::debug!("Ad-blocker updating");

                    let task = tokio::spawn(handle_background_update(
                        state.clone(),
                        engine.clone(),
                        file_manager.clone(),
                        cache_dir,
                        next_update_due,
                        cancel_token.child_token(),
                    ));

                    state_guard.replace(State::Updating {
                        task,
                        cancel_token,
                    });
                }
            }
            _ = cancel_token.cancelled() => {
                tracing::debug!("Ad-blocker update cancelled");

                // State transition handled by caller
            }
        }
    }
    .boxed()
}

async fn handle_background_update(
    state: Arc<Mutex<ObservableState>>,
    engine: AdBlockEngineRef,
    file_manager: FileManagerRef,
    cache_dir: PathBuf,
    current_update_at: OffsetDateTime,
    cancel_token: CancellationToken,
) {
    match file_manager.update_files(cancel_token.child_token()).await {
        Ok(is_updated) => {
            if is_updated {
                tracing::debug!("Ad-blocker was updated successfully");
            } else {
                tracing::debug!("Ad-blocker is already up-to-date");
            }
        }
        Err(error) => {
            if error.is_cancelled() {
                // Explicit return. State transition handled by caller
                return;
            } else {
                tracing::error!("Ad-blocker update failed: {error}");
            }
        }
    }

    let mut state_guard = state.lock().await;
    let State::Updating { .. } = state_guard.get() else {
        return;
    };

    if let Err(err) = reload_filters(&engine, &cache_dir).await {
        tracing::error!("Failed to load filter set: {err}");
        // Ignore error and continue with the existing rules
    }

    let task = tokio::spawn(schedule_next_update(
        state.clone(),
        engine,
        file_manager,
        cache_dir,
        current_update_at,
        ADBLOCK_UPDATE_DELAY,
        cancel_token.child_token(),
    ));

    state_guard.replace(State::Enabled { task, cancel_token });
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

    #[tokio::test(start_paused = true)]
    #[traced_test]
    async fn test_state_transitions() {
        let cache_dir = std::env::temp_dir();

        let engine = MockEngine::default();
        let file_manager = MockFileManager;

        let (state_tx, mut state_rx) = tokio::sync::mpsc::unbounded_channel();

        let adblocker = AdBlocker::create(
            cache_dir,
            ObservableState::new_with_observer(state_tx),
            Arc::new(AdBlockEngineWrap::Mock(engine)),
            Arc::new(AdBlockFileManagerWrap::Mock(file_manager)),
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

        tokio::time::advance(INITIAL_ADBLOCK_UPDATE_DELAY).await;

        assert!(matches!(
            wait_state(&mut state_rx).await.unwrap(),
            PrimitiveState::Updating
        ));
        assert!(matches!(
            wait_state(&mut state_rx).await.unwrap(),
            PrimitiveState::Enabled
        ));

        tokio::time::advance(ADBLOCK_UPDATE_DELAY).await;

        assert!(matches!(
            wait_state(&mut state_rx).await.unwrap(),
            PrimitiveState::Updating
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

    #[tokio::test(start_paused = true)]
    #[traced_test]
    async fn test_reset_store_on_load_filters_failure() {
        let cache_dir = std::env::temp_dir();

        let (engine, promise) = MockEngine::fail_once();
        let file_manager = MockFileManager;

        let (state_tx, mut state_rx) = tokio::sync::mpsc::unbounded_channel();

        let adblocker = AdBlocker::create(
            cache_dir,
            ObservableState::new_with_observer(state_tx),
            Arc::new(AdBlockEngineWrap::Mock(engine)),
            Arc::new(AdBlockFileManagerWrap::Mock(file_manager)),
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
