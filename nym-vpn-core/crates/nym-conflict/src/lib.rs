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

/// Which conflict check [`detect`] should run. The two checks are only
/// meaningful at different points in a connection attempt, so callers pick
/// the one that matches where they are:
///
/// - [`Self::InterceptedDns`] can only be observed once NymVPN is actually
///   connected and routing traffic through its own resolver.
/// - [`Self::CompetingVpn`] must be checked *before* NymVPN starts a
///   connection attempt, rather than after connecting or after a failure -
///   NymVPN's own attempt to take over the default route can itself force a
///   competing VPN's tunnel to disconnect, destroying the routing evidence
///   for it before NymVPN ever reaches a connected (or failed) state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictCheck {
    /// Check for [`Conflict::InterceptedDns`].
    InterceptedDns,

    /// Check for [`Conflict::CompetingVpn`]. Must only be run before NymVPN
    /// installs any of its own routes for the attempt - see the enum-level
    /// docs.
    CompetingVpn,
}

/// Run a single conflict check against the local system. No-op on mobile
/// platforms: NymVPN's local resolver (the thing [`ConflictCheck::InterceptedDns`]
/// relies on to answer [`PROBE_DOMAIN`]) isn't created there, which would
/// otherwise make the DNS check report a false positive on every run, and a
/// competing VPN tunnel can't coexist with NymVPN's own in the first place.
pub async fn detect(check: ConflictCheck) -> Vec<Conflict> {
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        let _ = check;
        Vec::new()
    }
    #[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
    match check {
        ConflictCheck::InterceptedDns => dns::scan().await,
        ConflictCheck::CompetingVpn => vpn::scan().await,
    }
}
