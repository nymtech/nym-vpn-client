// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::IpAddr;

use super::{Error, Result};

pub struct DefaultInterface {
    inner: netdev::Interface,
}

impl DefaultInterface {
    pub fn current() -> Result<Self> {
        Ok(Self {
            inner: netdev::interface::get_default_interface()
                .map_err(Error::GetDefaultInterface)?,
        })
    }

    pub fn interface_name(&self) -> &str {
        &self.inner.name
    }

    pub fn gateway_ip(&self) -> Option<IpAddr> {
        self.inner
            .gateway
            .as_ref()?
            .ipv4
            .first()
            .map(|addr| IpAddr::from(*addr))
            .or_else(|| {
                self.inner
                    .gateway
                    .as_ref()?
                    .ipv6
                    .first()
                    .map(|addr| IpAddr::from(*addr))
            })
    }
}
