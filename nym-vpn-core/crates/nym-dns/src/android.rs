// Copyright 2016-2024 Mullvad VPN AB. All Rights Reserved.
// Copyright 2024 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::ResolvedDnsConfig;

/// Stub error type for DNS errors on Android.
#[derive(Debug, thiserror::Error)]
#[error("Unknown Android DNS error")]
pub struct Error;

pub struct DnsMonitor;

impl super::DnsMonitorT for DnsMonitor {
    type Error = Error;

    fn new() -> Result<Self, Self::Error> {
        Ok(DnsMonitor)
    }

    fn set(&mut self, _interface: &str, _servers: ResolvedDnsConfig) -> Result<(), Self::Error> {
        Ok(())
    }

    fn reset(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
