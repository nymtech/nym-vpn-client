// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Module formalizing abstract connection probe interface and errors.

use std::time::Duration;

/// Describes interface for implementing a probe sender.
#[async_trait::async_trait]
pub trait ConnectionProbe {
    /// Send a probe with the given timeout.
    async fn send(&self, timeout: Duration) -> Result<(), BoxedProbeError>;
}

/// Describes probe errors.
pub trait ProbeError: std::error::Error + Send + 'static {
    /// Returns true if the error is a timeout error.
    fn is_timeout(&self) -> bool;
}

/// Type alias for boxed probe error.
pub type BoxedProbeError = Box<dyn ProbeError + Send + 'static>;

// Ensures that passing `&Box<dyn ProbeError>` does not deref into `dyn ProbeError` which does not work with `trace_err_chain!`.
impl std::error::Error for BoxedProbeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.as_ref().source()
    }
}
