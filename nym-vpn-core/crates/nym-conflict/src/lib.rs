// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Detection of other applications on the system that conflict with
//! NymVPN's own network filtering and routing (ad-blocking, DNS handling,
//! competing VPN tunnels).
//!
//! This crate only *detects* conflicting software - it never changes system
//! configuration or NymVPN's own behavior based on what it finds. Callers are
//! expected to surface findings to the user and otherwise proceed as normal.
//!
//! Detection works by behavior and structure, not by static system state or
//! vendor identity - see the [`dns`] and [`vpn`] modules for the specifics
//! of each check.

mod dns;
mod vpn;

pub use dns::{PROBE_ADDR, PROBE_DOMAIN};

/// A conflict between NymVPN and some other application found on the system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Conflict {
    /// DNS queries are being intercepted or rerouted before reaching
    /// NymVPN's own resolver.
    InterceptedDns,

    /// Another VPN client's tunnel appears to be competing for the default
    /// route alongside NymVPN's own.
    CompetingVpn,
}

/// Scan the local system for conflicting applications.
pub async fn scan() -> Vec<Conflict> {
    let mut conflicts = dns::scan().await;
    conflicts.extend(vpn::scan().await);
    conflicts
}
