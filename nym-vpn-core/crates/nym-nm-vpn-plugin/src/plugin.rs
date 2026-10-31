// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! The `org.freedesktop.NetworkManager.VPN.Plugin` D-Bus object.
//!
//! Method calls are turned into [`Command`]s for the service task; signals are emitted by that
//! task through the generated `VpnPluginSignals` trait on `InterfaceRef<VpnPlugin>`.

use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
};

use tokio::sync::{mpsc, oneshot};
use zbus::{fdo, object_server::SignalEmitter, zvariant::OwnedValue};

use crate::nm::service_state;

/// NetworkManager connection settings (`a{sa{sv}}`).
pub type Settings = HashMap<String, HashMap<String, OwnedValue>>;
/// A generic `a{sv}` dictionary.
pub type Dict = HashMap<String, OwnedValue>;

/// What the D-Bus object asks the service task to do.
pub enum Command {
    /// Start the tunnel. The reply is sent once the daemon accepted or refused the request.
    Connect {
        /// UUID of the NetworkManager connection profile being activated.
        uuid: Option<String>,
        reply: oneshot::Sender<Result<(), String>>,
    },
    /// Stop the tunnel. The reply is sent once the service decided how to handle it.
    Disconnect(oneshot::Sender<()>),
}

pub struct VpnPlugin {
    tx: mpsc::Sender<Command>,
    state: Arc<AtomicU32>,
}

impl VpnPlugin {
    pub fn new(tx: mpsc::Sender<Command>, state: Arc<AtomicU32>) -> Self {
        Self { tx, state }
    }

    fn is_active(&self) -> bool {
        matches!(
            self.state.load(Ordering::SeqCst),
            service_state::STARTING | service_state::STARTED | service_state::STOPPING
        )
    }
}

fn connection_uuid(connection: &Settings) -> Option<String> {
    let value = connection.get("connection")?.get("uuid")?;
    <&str>::try_from(&**value).ok().map(str::to_owned)
}

#[zbus::interface(name = "org.freedesktop.NetworkManager.VPN.Plugin")]
impl VpnPlugin {
    async fn connect(&self, connection: Settings) -> fdo::Result<()> {
        tracing::debug!("Connect: {:?}", connection.keys().collect::<Vec<_>>());
        if self.is_active() {
            return Err(fdo::Error::Failed("connection already active".into()));
        }

        let (reply, result) = oneshot::channel();
        let uuid = connection_uuid(&connection);

        self.tx
            .send(Command::Connect { uuid, reply })
            .await
            .map_err(|_| fdo::Error::Failed("service task is gone".into()))?;

        result
            .await
            .map_err(|_| fdo::Error::Failed("service task dropped the request".into()))?
            .map_err(fdo::Error::Failed)
    }

    async fn connect_interactive(&self, connection: Settings, details: Dict) -> fdo::Result<()> {
        tracing::debug!(
            "ConnectInteractive details: {:?}",
            details.keys().collect::<Vec<_>>()
        );
        self.connect(connection).await
    }

    async fn need_secrets(&self, _settings: Settings) -> fdo::Result<String> {
        // The account lives in the daemon; NetworkManager never needs to supply secrets.
        Ok(String::new())
    }

    async fn disconnect(&self) -> fdo::Result<()> {
        tracing::debug!("Disconnect");
        let (reply, ack) = oneshot::channel();
        self.tx
            .send(Command::Disconnect(reply))
            .await
            .map_err(|_| fdo::Error::Failed("service task is gone".into()))?;
        ack.await
            .map_err(|_| fdo::Error::Failed("service task dropped the request".into()))
    }

    async fn set_config(&self, config: Dict) {
        tracing::debug!("SetConfig ignored: {:?}", config.keys().collect::<Vec<_>>());
    }

    async fn set_ip4_config(&self, config: Dict) {
        tracing::debug!(
            "SetIp4Config ignored: {:?}",
            config.keys().collect::<Vec<_>>()
        );
    }

    async fn set_ip6_config(&self, config: Dict) {
        tracing::debug!(
            "SetIp6Config ignored: {:?}",
            config.keys().collect::<Vec<_>>()
        );
    }

    async fn set_failure(&self, reason: String) {
        tracing::warn!("NetworkManager reported failure: {reason}");
    }

    async fn new_secrets(&self, _connection: Settings) {
        tracing::debug!("NewSecrets ignored");
    }

    #[zbus(property)]
    fn state(&self) -> u32 {
        self.state.load(Ordering::SeqCst)
    }

    /// `StateChanged` is also the name zbus gives the `State` property-change helper, hence the
    /// explicit rename.
    #[zbus(signal, name = "StateChanged")]
    pub async fn service_state_changed(emitter: &SignalEmitter<'_>, state: u32)
    -> zbus::Result<()>;

    #[zbus(signal)]
    pub async fn secrets_required(
        emitter: &SignalEmitter<'_>,
        message: String,
        secrets: Vec<String>,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    pub async fn config(emitter: &SignalEmitter<'_>, config: Dict) -> zbus::Result<()>;

    #[zbus(signal)]
    pub async fn ip4_config(emitter: &SignalEmitter<'_>, ip4config: Dict) -> zbus::Result<()>;

    #[zbus(signal)]
    pub async fn ip6_config(emitter: &SignalEmitter<'_>, ip6config: Dict) -> zbus::Result<()>;

    #[zbus(signal)]
    pub async fn login_banner(emitter: &SignalEmitter<'_>, banner: String) -> zbus::Result<()>;

    #[zbus(signal)]
    pub async fn failure(emitter: &SignalEmitter<'_>, reason: u32) -> zbus::Result<()>;
}
