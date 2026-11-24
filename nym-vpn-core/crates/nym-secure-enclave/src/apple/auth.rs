// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::cell::RefCell;

use objc2::rc::Retained;
use objc2_core_foundation::CFAllocator;
use objc2_local_authentication::LAContext;
use tokio_util::task::LocalPoolHandle;

use crate::{Result, SecureEnclaveError};

use super::{
    local_authentication::{authenticate, non_interactive_context},
    zeroing_allocator::create_zeroing_allocator,
};

thread_local! {
    /// Authentication state owned by the worker thread. `LAContext` cannot be sent between
    /// threads, so it never leaves the thread it was created on.
    static AUTH_STATE: RefCell<AuthState> = RefCell::new(AuthState::new());
}

/// Local authentication shared by the secure enclave keys and the protected keychain items.
///
/// All operations requiring the authentication context run on a single worker thread. Clones
/// share the worker and the authentication state, so that a single [`Self::authorize`] grants
/// access to all of them.
#[derive(Clone)]
pub struct LocalAuthentication {
    pool: LocalPoolHandle,
}

impl Default for LocalAuthentication {
    fn default() -> Self {
        Self {
            pool: LocalPoolHandle::new(1),
        }
    }
}

impl LocalAuthentication {
    /// Authenticate the device owner once, so that items requiring user presence can be used
    /// without prompting until [`Self::deauthorize`] is called. Always prompts, replacing any
    /// previously obtained authentication. The `reason` is shown to the user.
    pub async fn authorize(&self, reason: &str) -> Result<()> {
        let reason = reason.to_owned();
        self.pool
            .spawn_pinned(move || async move { unlock_with_local_authentication(&reason).await })
            .await
            .map_err(|_| worker_panic())?
    }

    /// Revoke the authentication obtained with [`Self::authorize`].
    pub async fn deauthorize(&self) -> Result<()> {
        self.run(|state| state.lock()).await
    }

    /// Run the closure on the worker thread and wait for it to complete.
    pub(crate) async fn run<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut AuthState) -> T + Send + 'static,
        T: Send + 'static,
    {
        self.pool
            .spawn_pinned(move || async move { AUTH_STATE.with_borrow_mut(f) })
            .await
            .map_err(|_| worker_panic())
    }

    /// Run the closure on the worker thread without waiting for it.
    pub(crate) fn spawn<F>(&self, f: F)
    where
        F: FnOnce(&mut AuthState) + Send + 'static,
    {
        let _join_handle = self
            .pool
            .spawn_pinned(move || async move { AUTH_STATE.with_borrow_mut(f) });
    }
}

/// Authentication state of the worker thread.
///
/// Items created with user presence requirement are only accessible with the context evaluated
/// by local authentication. The context is tied to the process and cannot be shared with other
/// processes.
pub struct AuthState {
    /// Context that items are accessed with. When unlocked, it is the context evaluated by local
    /// authentication. When locked, it does not allow user interaction, so that items requiring
    /// user presence fail instead of prompting.
    pub context: Retained<LAContext>,
    /// Incremented whenever the context changes, so that keys bound to an older context are
    /// fetched again.
    pub generation: u64,
    /// Incremented on every `lock()`, so that an `unlock()` in flight does not install its
    /// context after it was locked.
    lock_epoch: u64,
}

impl AuthState {
    /// Runs once on the worker thread, before the first call into the Security framework.
    fn new() -> Self {
        match create_zeroing_allocator() {
            Some(allocator) => CFAllocator::set_default(Some(&allocator)),
            None => tracing::warn!("Failed to create zeroing CFAllocator"),
        }

        Self {
            context: non_interactive_context(),
            generation: 0,
            lock_epoch: 0,
        }
    }

    fn unlock(&mut self, context: Retained<LAContext>) {
        self.set_context(context);
    }

    fn lock(&mut self) {
        self.lock_epoch += 1;
        self.set_context(non_interactive_context());
    }

    fn set_context(&mut self, context: Retained<LAContext>) {
        let old_context = std::mem::replace(&mut self.context, context);
        unsafe { old_context.invalidate() };
        self.generation += 1;
    }
}

/// Authenticate the device owner and unlock, unless locked in the meantime. Must run on the
/// worker thread.
async fn unlock_with_local_authentication(reason: &str) -> Result<()> {
    let lock_epoch = AUTH_STATE.with_borrow(|state| state.lock_epoch);
    let context = authenticate(reason).await?;

    AUTH_STATE.with_borrow_mut(|state| {
        if state.lock_epoch == lock_epoch {
            state.unlock(context);
            Ok(())
        } else {
            unsafe { context.invalidate() };
            Err(SecureEnclaveError::UserCancelled)
        }
    })
}

fn worker_panic() -> SecureEnclaveError {
    SecureEnclaveError::Other("local authentication worker panic".to_owned())
}
