// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Detects a competing firewall/NAT rule that would intercept NymVPN's
//! bootstrap DNS traffic before NymVPN's own firewall chain ever evaluates
//! it.
//!
//! Linux-only for now - see [`linux`] for the detection approach and why.
//! No-op on every other platform.

#[cfg(target_os = "linux")]
mod linux;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use crate::Conflict;

/// Scan for a competing firewall/NAT rule. No-op on platforms other than
/// Linux.
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub(crate) async fn detect() -> Vec<Conflict> {
    #[cfg(target_os = "linux")]
    {
        linux::detect().await
    }
    #[cfg(not(target_os = "linux"))]
    {
        Vec::new()
    }
}
