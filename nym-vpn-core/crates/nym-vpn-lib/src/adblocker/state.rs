// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::ops::Deref;

use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

pub enum State {
    /// Indicates that adblocker is not active
    Disabled,

    /// Initializing filters, adblocker will transition to [`State::Enabled`] on success,
    /// otherwise it will transition to [`State::Disabled`]
    Initializing {
        /// Background task that is initializing filters
        task: JoinHandle<()>,
        /// Cancellation token for the background task
        cancel_token: CancellationToken,
    },

    /// Adblocker is active and filters are loaded
    Enabled {
        /// Background task initiating the next update.
        task: JoinHandle<()>,
        /// Cancellation token for the background task
        cancel_token: CancellationToken,
    },

    /// Adblocker had been enabled and is currently updating filters
    /// It will transition to [`State::Enabled`] on success or failure
    Updating {
        /// Background task that is updating filters
        task: JoinHandle<()>,
        /// Cancellation token for the background task
        cancel_token: CancellationToken,
    },
}

#[cfg(test)]
pub enum PrimitiveState {
    Enabled,
    Disabled,
    Updating,
    Initializing,
}

#[cfg(test)]
impl State {
    pub fn primitive_state(&self) -> PrimitiveState {
        match self {
            State::Enabled { .. } => PrimitiveState::Enabled,
            State::Disabled => PrimitiveState::Disabled,
            State::Updating { .. } => PrimitiveState::Updating,
            State::Initializing { .. } => PrimitiveState::Initializing,
        }
    }
}

/// Wrapper for [`State`] changes to which can be observed when testing.
pub struct ObservableState {
    inner: State,
    #[cfg(test)]
    observer: Option<tokio::sync::mpsc::UnboundedSender<PrimitiveState>>,
}

impl Default for ObservableState {
    fn default() -> Self {
        Self {
            inner: State::Disabled,
            #[cfg(test)]
            observer: None,
        }
    }
}

impl ObservableState {
    #[cfg(test)]
    pub fn new_with_observer(observer: tokio::sync::mpsc::UnboundedSender<PrimitiveState>) -> Self {
        Self {
            inner: State::Disabled,
            #[cfg(test)]
            observer: Some(observer),
        }
    }

    /// Replaces the inner state with a new value, returning the old state.
    pub fn replace(&mut self, inner: State) -> State {
        let old_state = std::mem::replace(&mut self.inner, inner);

        #[cfg(test)]
        if let Some(observer) = &self.observer {
            observer.send(self.inner.primitive_state()).ok();
        }

        old_state
    }

    /// Returns a reference to the inner state.
    pub fn get(&self) -> &State {
        &self.inner
    }
}

impl Deref for ObservableState {
    type Target = State;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
