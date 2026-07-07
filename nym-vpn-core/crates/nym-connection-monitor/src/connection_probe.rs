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

    /// Returns true when the probe could not be sent (e.g. raw socket bind on utun).
    fn is_send_failure(&self) -> bool {
        false
    }
}

/// Convenience wrapper around a boxed probe error.
#[derive(Debug)]
pub struct BoxedProbeError(pub Box<dyn ProbeError>);

impl std::fmt::Display for BoxedProbeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for BoxedProbeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}
