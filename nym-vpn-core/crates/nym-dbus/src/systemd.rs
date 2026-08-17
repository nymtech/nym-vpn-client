// Copyright 2016-2024 Mullvad VPN AB. All Rights Reserved.
// Copyright 2024 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use dbus::blocking::{Proxy, SyncConnection, stdintf::org_freedesktop_dbus::Properties};
use std::{sync::Arc, time::Duration};

type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to create a DBus connection")]
    ConnectError(#[source] dbus::Error),

    #[error("failed to read SystemState property")]
    ReadSystemStateError(#[source] dbus::Error),

    #[error("failed to read Version property")]
    ReadVersionError(#[source] dbus::Error)
}

const SYSTEMD_BUS: &str = "org.freedesktop.systemd1";
const SYSTEMD_PATH: &str = "/org/freedesktop/systemd1";
const MANAGER_INTERFACE: &str = "org.freedesktop.systemd1.Manager";
const SYSTEM_STATE: &str = "SystemState";
const SYSTEM_STATE_STARTING: &str = "starting";
const SYSTEM_STATE_INITIALIZING: &str = "initializing";
const SYSTEM_STATE_RUNNING: &str = "running";
const SYSTEM_STATE_DEGRADED: &str = "degraded";
const VERSION: &str = "Version";

const RPC_TIMEOUT: Duration = Duration::from_secs(1);

pub struct Systemd {
    pub dbus_connection: Arc<SyncConnection>,
}

impl Systemd {
    pub fn new() -> Result<Self> {
        Ok(Self {
            dbus_connection: crate::get_connection().map_err(Error::ConnectError)?,
        })
    }

    pub fn version(&self) -> Result<String> {
        self.as_manager_object().get::<String>(MANAGER_INTERFACE, VERSION)
            .map_err(Error::ReadVersionError)
    }

    /// Returns true if the host is not shutting down or entering maintenance mode or some other weird
    /// state.
    pub fn system_is_running(&self) -> Result<bool> {
        self.as_manager_object()
            .get(MANAGER_INTERFACE, SYSTEM_STATE)
            .map(|state: String| {
                ![
                    SYSTEM_STATE_STARTING,
                    SYSTEM_STATE_INITIALIZING,
                    SYSTEM_STATE_RUNNING,
                    SYSTEM_STATE_DEGRADED,
                ]
                .contains(&state.as_str())
            })
            .map_err(Error::ReadSystemStateError)
    }

    fn as_manager_object(&self) -> Proxy<'_, &SyncConnection> {
        Proxy::new(
            SYSTEMD_BUS,
            SYSTEMD_PATH,
            RPC_TIMEOUT,
            &self.dbus_connection,
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_version() {
        let version = Systemd::new()
            .expect("failed to create Systemd")
            .version()
            .expect("failed to get systemd version");
        println!("Version is {version}");
    }
}
