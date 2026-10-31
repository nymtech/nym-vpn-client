// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Shows the tunnel as an active VPN in NetworkManager, whichever client connected it.
//!
//! Whenever the tunnel comes up, or a profile is added to NetworkManager while the tunnel is up, the
//! NymVPN connection is activated in NetworkManager unless it is active already. NetworkManager then starts the nm-nymvpn-service plugin, which follows the
//! tunnel from there. Nothing happens when NetworkManager is not running or has no NymVPN profile.

use std::{collections::HashMap, future::Future, time::Duration};

use futures::{StreamExt, stream::BoxStream};
use nym_common::ErrorExt;
use nym_vpn_lib_types::TunnelState;
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;
use zbus::{
    Connection,
    proxy::CacheProperties,
    zvariant::{ObjectPath, OwnedObjectPath, OwnedValue, Value},
};

const SERVICE_TYPE: &str = "org.freedesktop.NetworkManager.nymvpn";

/// First NetworkManager version with `auto-route-ext-gw`.
const MIN_VERSION_AUTO_ROUTE_EXT_GW: (u32, u32) = (1, 42);

/// `NM_SETTINGS_UPDATE2_FLAG_IN_MEMORY`. Keeps the change out of /etc, where a copy of the packaged
/// profile would shadow the one in /usr/lib and hide its future updates.
const UPDATE2_FLAG_IN_MEMORY: u32 = 0x2;

/// `NM_TERNARY_FALSE`.
const TERNARY_FALSE: i32 = 0;

/// Upper bound for a whole activation round trip with NetworkManager.
const NM_TIMEOUT: Duration = Duration::from_secs(5);

/// Activate the NymVPN connection each time the tunnel comes up or a profile is added while it is
/// up, until `shutdown` is cancelled.
pub async fn run(mut tunnel_state_rx: watch::Receiver<TunnelState>, shutdown: CancellationToken) {
    let mut new_profiles = watch_new_profiles().await;

    loop {
        tokio::select! {
            changed = tunnel_state_rx.changed() => {
                if changed.is_err() {
                    break;
                }
            }
            Some(()) = new_profiles.next() => {}
            _ = shutdown.cancelled() => break,
        }

        let is_tunnel_up = matches!(
            *tunnel_state_rx.borrow(),
            TunnelState::Connected { .. } | TunnelState::Connecting { .. }
        );
        if is_tunnel_up {
            activate_connection().await;
        }
    }
}

/// Yields each time a profile is added to NetworkManager. Never yields when the system bus is not
/// available.
async fn watch_new_profiles() -> BoxStream<'static, ()> {
    match with_timeout(subscribe_new_profiles()).await {
        Ok(stream) => stream,
        Err(err) => {
            tracing::debug!(
                "{}",
                err.display_chain_with_msg("Cannot watch NetworkManager for new profiles")
            );
            futures::stream::pending().boxed()
        }
    }
}

async fn subscribe_new_profiles() -> zbus::Result<BoxStream<'static, ()>> {
    let connection = Connection::system().await?;
    let settings = SettingsProxy::builder(&connection)
        .cache_properties(CacheProperties::No)
        .build()
        .await?;
    let new_connections = settings.receive_new_connection().await?;
    Ok(new_connections.map(|_| ()).boxed())
}

async fn activate_connection() {
    match with_timeout(activate_vpn_connection(SERVICE_TYPE)).await {
        Ok(Some(uuid)) => tracing::info!("Activated NetworkManager connection {uuid}"),
        Ok(None) => {}
        Err(err) => tracing::debug!(
            "{}",
            err.display_chain_with_msg("Cannot activate the NetworkManager connection")
        ),
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

    let network_manager = network_manager(&connection).await?;
    if supports_auto_route_ext_gw(&network_manager).await {
        // Still activate on failure: NetworkManager showing the VPN matters more than one route.
        if let Err(err) = disable_gateway_route(&connection, profile).await {
            tracing::warn!(
                "{}",
                err.display_chain_with_msg(
                    "Cannot stop NetworkManager from routing to the VPN gateway"
                )
            );
        }
    }

    let any = ObjectPath::from_static_str_unchecked("/");
    network_manager
        .activate_connection(&profile.path, &any, &any)
        .await?;
    Ok(Some(profile.uuid.clone()))
}

/// Whether NetworkManager knows `auto-route-ext-gw`. Older versions may reject a profile that
/// sets it.
async fn supports_auto_route_ext_gw(network_manager: &NetworkManagerProxy<'_>) -> bool {
    let Ok(version) = network_manager.version().await else {
        return false;
    };
    let mut parts = version.split('.').map(str::parse::<u32>);
    match (parts.next(), parts.next()) {
        (Some(Ok(major)), Some(Ok(minor))) => (major, minor) >= MIN_VERSION_AUTO_ROUTE_EXT_GW,
        _ => false,
    }
}

/// Stop NetworkManager from adding a route to the VPN gateway through the physical interface when
/// `profile` is activated. The daemon does all routing itself.
///
/// The change is kept in memory only, so it is made again after a reboot. Secrets are not carried
/// over, which is fine as NymVPN profiles have none.
async fn disable_gateway_route(connection: &Connection, profile: &VpnProfile) -> zbus::Result<()> {
    let is_disabled = |section: &str| {
        profile
            .settings
            .get(section)
            .and_then(|section| section.get("auto-route-ext-gw"))
            .and_then(|value| value.downcast_ref::<i32>().ok())
            == Some(TERNARY_FALSE)
    };
    if is_disabled("ipv4") && is_disabled("ipv6") {
        return Ok(());
    }

    let mut settings = HashMap::<&str, HashMap<&str, Value<'_>>>::new();
    for (section, properties) in &profile.settings {
        let section = settings.entry(section.as_str()).or_default();
        for (key, value) in properties {
            section.insert(key.as_str(), Value::from(value.try_clone()?));
        }
    }
    for section in ["ipv4", "ipv6"] {
        settings
            .entry(section)
            .or_default()
            .insert("auto-route-ext-gw", Value::from(TERNARY_FALSE));
    }

    SettingsConnectionProxy::builder(connection)
        .path(&profile.path)?
        .cache_properties(CacheProperties::No)
        .build()
        .await?
        .update2(settings, UPDATE2_FLAG_IN_MEMORY, HashMap::new())
        .await?;
    tracing::info!(
        "Disabled the NetworkManager route to the VPN gateway for {}",
        profile.uuid
    );
    Ok(())
}

/// A VPN profile in NetworkManager.
struct VpnProfile {
    path: OwnedObjectPath,
    uuid: String,
    /// When the profile was last activated, in seconds since the epoch; 0 if never.
    last_used: u64,
    settings: ProfileSettings,
}

/// Profile settings as `GetSettings` returns them: properties by section.
type ProfileSettings = HashMap<String, HashMap<String, OwnedValue>>;

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
            settings,
        });
    }
    Ok(profiles)
}

/// Whether any of `profiles` is currently active, in any state. A profile that is still being torn
/// down counts too: it may be the user disconnecting in NetworkManager, and activating it again
/// would bring back the tunnel they just turned off.
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

    #[zbus(property)]
    fn version(&self) -> zbus::Result<String>;
}

#[zbus::proxy(
    interface = "org.freedesktop.NetworkManager.Settings",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager/Settings"
)]
trait Settings {
    fn list_connections(&self) -> zbus::Result<Vec<OwnedObjectPath>>;

    #[zbus(signal)]
    fn new_connection(&self, connection: OwnedObjectPath) -> zbus::Result<()>;
}

#[zbus::proxy(
    interface = "org.freedesktop.NetworkManager.Settings.Connection",
    default_service = "org.freedesktop.NetworkManager"
)]
trait SettingsConnection {
    fn get_settings(&self) -> zbus::Result<ProfileSettings>;

    fn update2(
        &self,
        settings: HashMap<&str, HashMap<&str, Value<'_>>>,
        flags: u32,
        args: HashMap<&str, Value<'_>>,
    ) -> zbus::Result<HashMap<String, OwnedValue>>;
}

#[zbus::proxy(
    interface = "org.freedesktop.NetworkManager.Connection.Active",
    default_service = "org.freedesktop.NetworkManager"
)]
trait ActiveConnection {
    #[zbus(property)]
    fn uuid(&self) -> zbus::Result<String>;
}
