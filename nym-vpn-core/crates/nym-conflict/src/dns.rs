// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Detects whether something on the system is intercepting or rerouting DNS
//! queries before they reach NymVPN's own resolver.

use std::net::Ipv4Addr;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use std::{net::IpAddr, time::Duration};

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use crate::Conflict;

/// Canary domain resolved by [`detect`] to detect DNS interception. Callers
/// that also run NymVPN's own DNS resolver (see `nym-vpn-lib`'s `resolver`
/// module) must answer this domain with [`PROBE_ADDR`], regardless of
/// ad-block/filter configuration.
pub const PROBE_DOMAIN: &str = "nym-conflict-probe.invalid.";

/// The address NymVPN's own DNS resolver answers [`PROBE_DOMAIN`] with.
/// Taken from the IPv4 documentation range (RFC 5737 TEST-NET-1) so it can
/// never be a real, independently-routable answer.
pub const PROBE_ADDR: Ipv4Addr = Ipv4Addr::new(192, 0, 2, 53);

#[cfg(not(any(target_os = "android", target_os = "ios")))]
const PROBE_TIMEOUT: Duration = Duration::from_secs(2);

/// Scan for DNS interception.
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub(crate) async fn detect() -> Vec<Conflict> {
    if probe_dns_interception().await {
        vec![Conflict::InterceptedDns]
    } else {
        Vec::new()
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
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

#[cfg(all(test, not(any(target_os = "android", target_os = "ios"))))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn probe_reports_interception_outside_the_tunnel() {
        assert!(probe_dns_interception().await);
    }
}
