// Copyright 2016-2024 Mullvad VPN AB. All Rights Reserved.
// Copyright 2024 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod network_manager;
mod resolvconf;
mod static_resolv_conf;
mod systemd_resolved;

use std::{env, fmt, net::IpAddr};

use nym_routing::RouteManagerHandle;

use self::{
    network_manager::NetworkManager, resolvconf::Resolvconf, static_resolv_conf::StaticResolvConf,
    systemd_resolved::SystemdResolved,
};
use super::ResolvedDnsConfig;

pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can happen in the Linux DNS monitor
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Error in systemd-resolved DNS monitor
    #[error("Error in systemd-resolved DNS monitor")]
    SystemdResolved(#[from] systemd_resolved::Error),

    /// Error in NetworkManager DNS monitor
    #[error("Error in NetworkManager DNS monitor")]
    NetworkManager(#[from] network_manager::Error),

    /// Error in resolvconf DNS monitor
    #[error("Error in resolvconf DNS monitor")]
    Resolvconf(#[from] resolvconf::Error),

    /// Error in static /etc/resolv.conf DNS monitor
    #[error("Error in static /etc/resolv.conf DNS monitor")]
    StaticResolvConf(#[from] static_resolv_conf::Error),

    /// No suitable DNS monitor implementation detected
    #[error("No suitable DNS monitor implementation detected")]
    NoDnsMonitor,
}

pub struct DnsMonitor {
    route_manager: RouteManagerHandle,
    inner: Option<DnsMonitorHolder>,
}

impl super::DnsMonitorT for DnsMonitor {
    type Error = Error;

    fn new(route_manager: RouteManagerHandle) -> Result<Self> {
        Ok(DnsMonitor {
            route_manager,
            inner: None,
        })
    }

    async fn set(&mut self, interface: &str, config: ResolvedDnsConfig) -> Result<()> {
        let servers = config.tunnel_config();
        self.reset().await?;
        // Creating a new DNS monitor for each set, in case the system changed how it manages DNS.
        let mut inner = DnsMonitorHolder::new()?;
        if !servers.is_empty() {
            inner.set(&self.route_manager, interface, servers).await?;
            self.inner = Some(inner);
        }
        Ok(())
    }

    async fn reset(&mut self) -> Result<()> {
        if let Some(mut inner) = self.inner.take() {
            inner.reset().await?;
        }
        Ok(())
    }
}

pub enum DnsMonitorHolder {
    SystemdResolved(SystemdResolved),
    NetworkManager(NetworkManager),
    Resolvconf(Resolvconf),
    StaticResolvConf(StaticResolvConf),
}

impl fmt::Display for DnsMonitorHolder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::DnsMonitorHolder::*;
        let name = match self {
            Resolvconf(..) => "resolvconf",
            StaticResolvConf(..) => "/etc/resolv.conf",
            SystemdResolved(..) => "systemd-resolved",
            NetworkManager(..) => "network manager",
        };
        f.write_str(name)
    }
}

impl DnsMonitorHolder {
    fn new() -> Result<Self> {
        let dns_module = env::var_os("NYM_DNS_MODULE");

        let manager = match dns_module.as_ref().and_then(|value| value.to_str()) {
            Some("static-file") => DnsMonitorHolder::StaticResolvConf(StaticResolvConf::new()?),
            Some("resolvconf") => DnsMonitorHolder::Resolvconf(Resolvconf::new()?),
            Some("systemd") => DnsMonitorHolder::SystemdResolved(SystemdResolved::new()?),
            Some("network-manager") => DnsMonitorHolder::NetworkManager(NetworkManager::new()?),
            Some(_) | None => Self::with_detected_dns_manager()?,
        };
        tracing::debug!("Managing DNS via {}", manager);
        Ok(manager)
    }

    fn with_detected_dns_manager() -> Result<Self> {
        SystemdResolved::new()
            .map(DnsMonitorHolder::SystemdResolved)
            .or_else(|err| {
                match err {
                    systemd_resolved::Error::SystemdResolvedError(
                        systemd_resolved::SystemdDbusError::NoSystemdResolved(_),
                    ) => (),
                    other_error => {
                        tracing::debug!("NetworkManager is being used because {}", other_error)
                    }
                }
                NetworkManager::new().map(DnsMonitorHolder::NetworkManager)
            })
            .or_else(|_| Resolvconf::new().map(DnsMonitorHolder::Resolvconf))
            .or_else(|_| StaticResolvConf::new().map(DnsMonitorHolder::StaticResolvConf))
            .map_err(|_| Error::NoDnsMonitor)
    }

    async fn set(
        &mut self,
        route_manager: &RouteManagerHandle,
        interface: &str,
        servers: &[IpAddr],
    ) -> Result<()> {
        use self::DnsMonitorHolder::*;
        match self {
            Resolvconf(ref mut resolvconf) => resolvconf.set_dns(interface, servers)?,
            StaticResolvConf(ref mut static_resolv_conf) => {
                static_resolv_conf.set_dns(servers.to_vec()).await?
            }
            SystemdResolved(ref mut systemd_resolved) => {
                systemd_resolved
                    .set_dns(route_manager.clone(), interface, servers)
                    .await?
            }
            NetworkManager(ref mut network_manager) => {
                network_manager.set_dns(interface, servers)?
            }
        }
        Ok(())
    }

    async fn reset(&mut self) -> Result<()> {
        use self::DnsMonitorHolder::*;
        match self {
            Resolvconf(ref mut resolvconf) => resolvconf.reset()?,
            StaticResolvConf(ref mut static_resolv_conf) => static_resolv_conf.reset().await?,
            SystemdResolved(ref mut systemd_resolved) => systemd_resolved.reset().await?,
            NetworkManager(ref mut network_manager) => network_manager.reset()?,
        }
        Ok(())
    }
}

/// Returns true if DnsMonitor will use NetworkManager to manage DNS.
pub fn will_use_nm() -> bool {
    crate::imp::SystemdResolved::new().is_err() && crate::imp::NetworkManager::new().is_ok()
}
