// Copyright 2016-2024 Mullvad VPN AB. All Rights Reserved.
// Copyright 2024 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::{iphlpapi, netsh, tcpip};
use crate::{DnsMonitorT, ResolvedDnsConfig};
use windows::Win32::System::Rpc::RPC_S_SERVER_UNAVAILABLE;

pub struct DnsMonitor {
    current_monitor: InnerMonitor,
}
enum InnerMonitor {
    Iphlpapi(iphlpapi::DnsMonitor),
    Netsh(netsh::DnsMonitor),
    Tcpip(tcpip::DnsMonitor),
}

impl InnerMonitor {
    async fn set(
        &mut self,
        interface: &str,
        config: ResolvedDnsConfig,
    ) -> Result<(), super::Error> {
        match self {
            InnerMonitor::Iphlpapi(monitor) => monitor.set(interface, config).await?,
            InnerMonitor::Netsh(monitor) => monitor.set(interface, config).await?,
            InnerMonitor::Tcpip(monitor) => monitor.set(interface, config).await?,
        }
        Ok(())
    }

    async fn reset(&mut self) -> Result<(), super::Error> {
        match self {
            InnerMonitor::Iphlpapi(monitor) => monitor.reset().await?,
            InnerMonitor::Netsh(monitor) => monitor.reset().await?,
            InnerMonitor::Tcpip(monitor) => monitor.reset().await?,
        }
        Ok(())
    }

    async fn reset_before_interface_removal(&mut self) -> Result<(), super::Error> {
        match self {
            InnerMonitor::Iphlpapi(monitor) => monitor.reset_before_interface_removal().await?,
            InnerMonitor::Netsh(monitor) => monitor.reset_before_interface_removal().await?,
            InnerMonitor::Tcpip(monitor) => monitor.reset_before_interface_removal().await?,
        }
        Ok(())
    }
}

impl DnsMonitorT for DnsMonitor {
    type Error = super::Error;

    fn new() -> Result<Self, Self::Error> {
        let current_monitor = if iphlpapi::DnsMonitor::is_supported() {
            InnerMonitor::Iphlpapi(iphlpapi::DnsMonitor::new()?)
        } else {
            InnerMonitor::Netsh(netsh::DnsMonitor::new()?)
        };

        Ok(Self { current_monitor })
    }

    async fn set(&mut self, interface: &str, config: ResolvedDnsConfig) -> Result<(), Self::Error> {
        let result = self.current_monitor.set(interface, config.clone()).await;
        if self.fallback_due_to_dnscache(&result) {
            return Box::pin(self.set(interface, config)).await;
        }
        result
    }

    async fn reset(&mut self) -> Result<(), Self::Error> {
        let result = self.current_monitor.reset().await;
        if self.fallback_due_to_dnscache(&result) {
            return Box::pin(self.reset()).await;
        }
        result
    }

    async fn reset_before_interface_removal(&mut self) -> Result<(), Self::Error> {
        let result = self.current_monitor.reset_before_interface_removal().await;
        if self.fallback_due_to_dnscache(&result) {
            return Box::pin(self.reset_before_interface_removal()).await;
        }
        result
    }
}

impl DnsMonitor {
    fn fallback_due_to_dnscache(&mut self, result: &Result<(), super::Error>) -> bool {
        let is_dnscache_error = match result {
            Err(super::Error::Iphlpapi(iphlpapi::Error::SetInterfaceDnsSettings(error))) => {
                error.code() == RPC_S_SERVER_UNAVAILABLE.to_hresult()
            }
            Err(super::Error::Netsh(netsh::Error::Netsh(Some(1)))) => true,
            _ => false,
        };
        if is_dnscache_error {
            tracing::warn!("dnscache is not running? Falling back on tcpip method");

            match tcpip::DnsMonitor::new() {
                Ok(mut tcpip) => {
                    // We need to disable flushing here since it may fail.
                    // Because dnscache is disabled, there's nothing to flush anyhow.
                    tcpip.disable_flushing();
                    self.current_monitor = InnerMonitor::Tcpip(tcpip);
                    true
                }
                Err(error) => {
                    tracing::error!("Failed to init tcpip DNS module: {error}");
                    false
                }
            }
        } else {
            false
        }
    }
}
