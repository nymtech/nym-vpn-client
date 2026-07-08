// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Module implementing mock connection probe

use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

use crate::{BoxedProbeError, ConnectionProbe, ProbeError};

/// Defines the mock probe outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Probe timeout
    Timeout,

    /// Successful probe after a certain delay
    Succeed { after: Duration },

    /// Failure to send the probe (i.e socket error)
    SendFailure,
}

/// Mock probe implementation for testing purposes.
/// It enables the simulation of various probe outcomes.
pub struct MockProbe {
    outcome_index: AtomicUsize,
    outcomes: Vec<Outcome>,
    subsequent_outcome: Outcome,
}

impl MockProbe {
    /// Creates a new mock probe with the given outcomes.
    /// Subsequent outcomes are repeated indefinitely once all outcomes are exhausted.
    pub fn new(outcomes: Vec<Outcome>, subsequent_outcome: Outcome) -> Self {
        Self {
            outcome_index: AtomicUsize::new(0),
            outcomes,
            subsequent_outcome,
        }
    }

    /// Creates a new mock probe with a single outcome that is repeated indefinitely.
    pub fn repeating(outcome: Outcome) -> Self {
        Self {
            outcome_index: AtomicUsize::new(0),
            outcomes: vec![],
            subsequent_outcome: outcome,
        }
    }
}

#[async_trait::async_trait]
impl ConnectionProbe for MockProbe {
    async fn send(&self, timeout: Duration) -> Result<(), BoxedProbeError> {
        let index = self.outcome_index.fetch_add(1, Ordering::Relaxed);

        let outcome = if index < self.outcomes.len() {
            &self.outcomes[index]
        } else {
            &self.subsequent_outcome
        };

        match outcome {
            Outcome::Timeout => {
                tokio::time::advance(timeout).await;
                Err(BoxedProbeError::from(MockProbeError::Timeout))
            }
            Outcome::Succeed { after } => {
                // Don't advance time past timeout to uphold the trait correctness.
                let advance_by = std::cmp::min(after, &timeout);
                tokio::time::advance(*advance_by).await;
                Ok(())
            }
            Outcome::SendFailure => Err(BoxedProbeError::from(MockProbeError::SendFailure)),
        }
    }
}

#[derive(Debug, Copy, Clone, thiserror::Error)]
pub enum MockProbeError {
    #[error("timeout")]
    Timeout,

    #[error("socket error")]
    SendFailure,
}

impl ProbeError for MockProbeError {
    fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout)
    }

    fn is_send_failure(&self) -> bool {
        matches!(self, Self::SendFailure)
    }
}

impl From<MockProbeError> for BoxedProbeError {
    fn from(error: MockProbeError) -> Self {
        BoxedProbeError(Box::new(error))
    }
}
