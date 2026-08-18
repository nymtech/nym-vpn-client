// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Detects whether something on the system is intercepting or rerouting DNS
//! queries before they reach NymVPN's own resolver.
//!
//! Detection works by behavior, not by static system state: a driver or
//! service being present doesn't mean the feature it implements is actually
//! turned on, and checking for that turned out to produce a WFP footprint
//! that's identical whether a culprit's protection is enabled or disabled.
//! Instead, [`scan`] resolves [`PROBE_DOMAIN`] the same way an ordinary
//! application would (through the OS resolver) and checks whether the answer
//! matches what NymVPN's own DNS resolver would have returned for it. A
//! mismatch - a different address, a timeout, or a failure - means something
//! between the caller and our resolver is intercepting or rerouting DNS
//! traffic. This is deliberately vendor-agnostic: we don't try to guess
//! *what* is intercepting DNS, since that's a coincidence-based attribution
//! (something else being installed doesn't mean it's the actual cause) that
//! costs ongoing per-vendor maintenance for little real benefit.

use std::net::Ipv4Addr;
#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
use std::{net::IpAddr, time::Duration};

#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
use crate::Conflict;

/// Canary domain resolved by [`scan`] to detect DNS interception. Callers
/// that also run NymVPN's own DNS resolver (see `nym-vpn-lib`'s `resolver`
/// module) must answer this domain with [`PROBE_ADDR`], regardless of
/// ad-block/filter configuration.
pub const PROBE_DOMAIN: &str = "nym-conflict-probe.invalid.";

/// The address NymVPN's own DNS resolver answers [`PROBE_DOMAIN`] with.
/// Taken from the IPv4 documentation range (RFC 5737 TEST-NET-1) so it can
/// never be a real, independently-routable answer.
pub const PROBE_ADDR: Ipv4Addr = Ipv4Addr::new(192, 0, 2, 53);

#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
const PROBE_TIMEOUT: Duration = Duration::from_secs(2);

/// Scan for DNS interception.
///
/// This resolves [`PROBE_DOMAIN`] through the OS resolver (mimicking how any
/// other application on the system would perform DNS lookups) and only
/// reports a conflict if that resolution doesn't come back the way NymVPN's
/// own resolver would answer it - so nothing is reported merely because some
/// other DNS-capable software is installed or running.
#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
pub(crate) async fn scan() -> Vec<Conflict> {
    if probe_dns_interception().await {
        vec![Conflict::InterceptedDns]
    } else {
        Vec::new()
    }
}

/// Resolve [`PROBE_DOMAIN`] via the OS resolver and check whether the answer
/// matches [`PROBE_ADDR`]. Returns `true` if it doesn't - i.e. if DNS
/// resolution failed, timed out, or came back with an unexpected address.
#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
async fn probe_dns_interception() -> bool {
    let target = format!("{PROBE_DOMAIN}:0");
    match tokio::time::timeout(PROBE_TIMEOUT, tokio::net::lookup_host(target)).await {
        Ok(Ok(addrs)) => {
            let expected = IpAddr::V4(PROBE_ADDR);
            !addrs.map(|addr| addr.ip()).any(|ip| ip == expected)
        }
        Ok(Err(error)) => {
            tracing::debug!("conflict probe: DNS resolution failed: {error}");
            true
        }
        Err(_) => {
            tracing::debug!("conflict probe: DNS resolution timed out");
            true
        }
    }
}

#[cfg(all(
    test,
    any(target_os = "windows", target_os = "linux", target_os = "macos")
))]
mod tests {
    use super::*;

    /// Outside of NymVPN's own tunnel, nothing on the network can ever
    /// answer [`PROBE_DOMAIN`] with [`PROBE_ADDR`], so the probe must report
    /// interception (rather than silently treating resolution failure as a
    /// pass) - this is the fail-safe this whole mechanism depends on.
    #[tokio::test]
    async fn probe_reports_interception_outside_the_tunnel() {
        assert!(probe_dns_interception().await);
    }
}
