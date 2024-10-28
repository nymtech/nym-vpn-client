// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{fmt, net::IpAddr};

use nym_dns::{DnsConfig, DnsMonitor};

#[cfg(target_os = "linux")]
use super::route_handler::RouteHandler;

pub struct DnsHandler {
    inner: DnsMonitor,
}

impl DnsHandler {
    pub async fn new(#[cfg(target_os = "linux")] route_handler: &RouteHandler) -> Result<Self> {
        Ok(Self {
            inner: DnsMonitor::new(
                #[cfg(target_os = "linux")]
                tokio::runtime::Handle::current(),
                #[cfg(target_os = "linux")]
                route_handler.inner_handle(),
            )?,
        })
    }

    pub fn set(&mut self, interface: &str, servers: &[IpAddr]) -> Result<()> {
        Ok(tokio::task::block_in_place(|| {
            let dns_config = DnsConfig::default().resolve(servers);

            self.inner.set(interface, dns_config)
        })?)
    }

    pub fn reset(&mut self) -> Result<()> {
        Ok(tokio::task::block_in_place(|| self.inner.reset())?)
    }

    pub fn reset_before_interface_removal(&mut self) -> Result<()> {
        Ok(tokio::task::block_in_place(|| {
            self.inner.reset_before_interface_removal()
        })?)
    }
}

#[derive(Debug)]
pub struct Error {
    inner: Box<dyn std::error::Error + 'static>,
}

unsafe impl Send for Error {}
unsafe impl Sync for Error {}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.inner.as_ref())
    }
}

impl From<nym_dns::Error> for Error {
    fn from(value: nym_dns::Error) -> Self {
        Self {
            inner: Box::new(value),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DNS error")
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
