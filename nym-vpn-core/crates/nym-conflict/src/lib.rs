// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Detection of other applications on the system that conflict with
//! NymVPN's own network filtering and routing (ad-blocking, DNS handling,
//! competing VPN tunnels).

mod dns;
mod firewall;
mod vpn;

pub use dns::{PROBE_ADDR, PROBE_DOMAIN};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Conflict {
    InterceptedDns,
    CompetingVpn,
    CompetingFirewall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictCheck {
    InterceptedDns,
    CompetingVpn,
    CompetingFirewall,
}

pub async fn detect(check: ConflictCheck) -> Vec<Conflict> {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = check;
        Vec::new()
    }
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    match check {
        ConflictCheck::InterceptedDns => dns::detect().await,
        ConflictCheck::CompetingVpn => vpn::detect().await,
        ConflictCheck::CompetingFirewall => firewall::detect().await,
    }
}
