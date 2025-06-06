// Copyright 2016-2024 Mullvad VPN AB. All Rights Reserved.
// Copyright 2024 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::IpAddr;

use nym_common::trace_err_chain;
use nym_dbus::systemd_resolved::{AsyncHandle, SystemdResolved as DbusInterface};
use nym_routing::RouteManagerHandle;

pub(crate) use nym_dbus::systemd_resolved::Error as SystemdDbusError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("systemd-resolved operation failed")]
    SystemdResolvedError(#[from] SystemdDbusError),

    #[error("unable to translate network interface name \"{0}\" into index")]
    LookupInterfaceIndex(String, #[source] nix::Error),
}

pub struct SystemdResolved {
    pub dbus_interface: AsyncHandle,
    tunnel_index: u32,
}

impl SystemdResolved {
    pub fn new() -> Result<Self> {
        let dbus_interface = DbusInterface::new()?.async_handle();

        let systemd_resolved = SystemdResolved {
            dbus_interface,
            tunnel_index: 0,
        };

        Ok(systemd_resolved)
    }

    pub async fn set_dns(
        &mut self,
        _route_manager: RouteManagerHandle,
        interface_name: &str,
        servers: &[IpAddr],
    ) -> Result<()> {
        let tunnel_index = nix::net::if_::if_nametoindex(interface_name)
            .map_err(|e| Error::LookupInterfaceIndex(interface_name.to_owned(), e))?;
        self.tunnel_index = tunnel_index;

        if let Err(error) = self.dbus_interface.disable_dot(self.tunnel_index).await {
            trace_err_chain!(error, "Failed to disable DoT");
        }

        if let Err(error) = self
            .dbus_interface
            .set_domains(tunnel_index, &[(".", true)])
            .await
        {
            trace_err_chain!(error, "Failed to set search domains");
        }

        let _ = self
            .dbus_interface
            .set_dns(self.tunnel_index, servers.to_vec())
            .await?;

        Ok(())
    }

    pub async fn reset(&mut self) -> Result<()> {
        if let Err(error) = self
            .dbus_interface
            .set_domains(self.tunnel_index, &[])
            .await
        {
            trace_err_chain!(error, "Failed to set search domains");
        }

        let _ = self
            .dbus_interface
            .set_dns(self.tunnel_index, vec![])
            .await?;

        Ok(())
    }
}
