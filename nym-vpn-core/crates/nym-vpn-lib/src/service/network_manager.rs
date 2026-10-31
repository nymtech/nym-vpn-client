// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Shows the tunnel as an active VPN in NetworkManager, whichever client connected it.
//!
//! Whenever the tunnel comes up, the NymVPN connection is activated in NetworkManager unless it is
//! active already. NetworkManager then starts the nm-nymvpn-service plugin, which follows the
//! tunnel from there. Nothing happens when NetworkManager is not running or has no NymVPN profile.

use std::{collections::HashMap, future::Future, time::Duration};

use nym_common::ErrorExt;
use nym_vpn_lib_types::TunnelState;
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;
use zbus::{
    Connection,
    proxy::CacheProperties,
    zvariant::{ObjectPath, OwnedObjectPath, OwnedValue},
};

const SERVICE_TYPE: &str = "org.freedesktop.NetworkManager.nymvpn";

/// Upper bound for a whole activation round trip with NetworkManager.
const NM_TIMEOUT: Duration = Duration::from_secs(5);

/// Activate the NymVPN connection each time the tunnel comes up, until `shutdown` is cancelled.
pub async fn run(mut tunnel_state_rx: watch::Receiver<TunnelState>, shutdown: CancellationToken) {
    let mut is_active = false;

    loop {
        tokio::select! {
            changed = tunnel_state_rx.changed() => {
                if changed.is_err() {
                    break;
                }
            }
            _ = shutdown.cancelled() => break,
        }

        let tunnel_state = tunnel_state_rx.borrow().clone();
        match tunnel_state {
            TunnelState::Connected { .. } | TunnelState::Connecting { .. } => {
                if !is_active {
                    is_active = activate_connection().await;
                }
            }
            TunnelState::Disconnecting { .. } => {}
            TunnelState::Disconnected | TunnelState::Offline { .. } | TunnelState::Error(..) => {
                is_active = false
            }
        }
    }
}

async fn activate_connection() -> bool {
    match with_timeout(activate_vpn_connection(SERVICE_TYPE)).await {
        Ok(Some(uuid)) => {
            tracing::info!("Activated NetworkManager connection {uuid}");
            true
        }
        Ok(None) => false,
        Err(err) => {
            tracing::debug!(
                "{}",
                err.display_chain_with_msg("Cannot activate the NetworkManager connection")
            );
            false
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("D-Bus call to NetworkManager failed")]
    Dbus(#[from] zbus::Error),

    #[error("NetworkManager did not respond in time")]
    Timeout,
}

async fn with_timeout<T>(
    future: impl Future<Output = zbus::Result<T>>,
) -> std::result::Result<T, Error> {
    tokio::time::timeout(NM_TIMEOUT, future)
        .await
        .map_err(|_| Error::Timeout)?
        .map_err(Error::Dbus)
}

/// Activate the most recently used profile of the VPN `service_type` on top of the primary
/// connection, unless a profile of that VPN is already active. Returns the UUID of the activated
/// profile.
async fn activate_vpn_connection(service_type: &str) -> zbus::Result<Option<String>> {
    let connection = Connection::system().await?;
    let profiles = vpn_profiles(&connection, service_type).await?;
    let Some(profile) = profiles.iter().max_by_key(|profile| profile.last_used) else {
        return Ok(None);
    };

    if has_active_profile(&connection, &profiles).await? {
        return Ok(None);
    }

    let any = ObjectPath::from_static_str_unchecked("/");
    network_manager(&connection)
        .await?
        .activate_connection(&profile.path, &any, &any)
        .await?;
    Ok(Some(profile.uuid.clone()))
}

/// A VPN profile in NetworkManager.
struct VpnProfile {
    path: OwnedObjectPath,
    uuid: String,
    /// When the profile was last activated, in seconds since the epoch; 0 if never.
    last_used: u64,
}

/// All saved profiles of the VPN `service_type`.
async fn vpn_profiles(
    connection: &Connection,
    service_type: &str,
) -> zbus::Result<Vec<VpnProfile>> {
    let settings = SettingsProxy::builder(connection)
        .cache_properties(CacheProperties::No)
        .build()
        .await?;

    let mut profiles = Vec::new();
    for path in settings.list_connections().await? {
        let profile = SettingsConnectionProxy::builder(connection)
            .path(&path)?
            .cache_properties(CacheProperties::No)
            .build()
            .await?;
        // Profiles can disappear or be unreadable for us; skip those.
        let Ok(settings) = profile.get_settings().await else {
            continue;
        };

        let value = |section: &str, key: &str| settings.get(section)?.get(key);
        let is_service_type = value("vpn", "service-type")
            .and_then(|value| value.downcast_ref::<&str>().ok())
            .is_some_and(|value| value == service_type);
        if !is_service_type {
            continue;
        }
        let Some(uuid) =
            value("connection", "uuid").and_then(|value| value.downcast_ref::<&str>().ok())
        else {
            continue;
        };
        let last_used = value("connection", "timestamp")
            .and_then(|value| value.downcast_ref::<u64>().ok())
            .unwrap_or(0);

        profiles.push(VpnProfile {
            uuid: uuid.to_owned(),
            path,
            last_used,
        });
    }
    Ok(profiles)
}

/// Whether any of `profiles` is currently active.
async fn has_active_profile(
    connection: &Connection,
    profiles: &[VpnProfile],
) -> zbus::Result<bool> {
    for path in network_manager(connection)
        .await?
        .active_connections()
        .await?
    {
        let active = ActiveConnectionProxy::builder(connection)
            .path(&path)?
            .cache_properties(CacheProperties::No)
            .build()
            .await?;
        // Active connections can go away while we iterate; skip those.
        let Ok(uuid) = active.uuid().await else {
            continue;
        };
        if profiles.iter().any(|profile| profile.uuid == uuid) {
            return Ok(true);
        }
    }
    Ok(false)
}

async fn network_manager(connection: &Connection) -> zbus::Result<NetworkManagerProxy<'static>> {
    NetworkManagerProxy::builder(connection)
        .cache_properties(CacheProperties::No)
        .build()
        .await
}

#[zbus::proxy(
    interface = "org.freedesktop.NetworkManager",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager"
)]
trait NetworkManager {
    fn activate_connection(
        &self,
        connection: &ObjectPath<'_>,
        device: &ObjectPath<'_>,
        specific_object: &ObjectPath<'_>,
    ) -> zbus::Result<OwnedObjectPath>;

    #[zbus(property)]
    fn active_connections(&self) -> zbus::Result<Vec<OwnedObjectPath>>;
}

#[zbus::proxy(
    interface = "org.freedesktop.NetworkManager.Settings",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager/Settings"
)]
trait Settings {
    fn list_connections(&self) -> zbus::Result<Vec<OwnedObjectPath>>;
}

#[zbus::proxy(
    interface = "org.freedesktop.NetworkManager.Settings.Connection",
    default_service = "org.freedesktop.NetworkManager"
)]
trait SettingsConnection {
    fn get_settings(&self) -> zbus::Result<HashMap<String, HashMap<String, OwnedValue>>>;
}

#[zbus::proxy(
    interface = "org.freedesktop.NetworkManager.Connection.Active",
    default_service = "org.freedesktop.NetworkManager"
)]
trait ActiveConnection {
    #[zbus(property)]
    fn uuid(&self) -> zbus::Result<String>;
}
