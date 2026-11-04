// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! The task that owns the daemon connection and turns NetworkManager commands and daemon events
//! into the signals decided by [`crate::state::Machine`].

use std::{
    collections::HashMap,
    net::IpAddr,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
    task::{Context, Poll, Waker},
    time::Duration,
};

use nym_vpn_lib_types::{TunnelEvent, TunnelState};
use nym_vpn_proto::rpc_client::RpcClient;
use tokio::{
    sync::mpsc,
    time::{Instant, sleep_until},
};
use tokio_stream::{Stream, StreamExt};
use zbus::{
    object_server::InterfaceRef,
    zvariant::{OwnedValue, Value},
};

use crate::{
    nm::{
        ACTIVE_CONNECTION_STATE_DEACTIVATING, ActiveConnectionProxy, NetworkManagerProxy, failure,
    },
    plugin::{Command, VpnPlugin, VpnPluginSignals},
    state::{Effect, Machine, Phase},
};

/// How long the plugin stays alive once idle before letting NetworkManager respawn it later.
pub const IDLE_EXIT: Duration = Duration::from_secs(20);
/// How long to wait for the daemon to report a terminal state after `DisconnectTunnel`.
pub const DISCONNECT_WAIT: Duration = Duration::from_secs(15);
/// Upper bound on any single daemon RPC; NetworkManager gives our method handlers 10 s in total.
pub const RPC_TIMEOUT: Duration = Duration::from_secs(5);

pub type EventStream = Pin<Box<dyn Stream<Item = TunnelEvent> + Send>>;
pub type DeactivationStream = Pin<Box<dyn Stream<Item = ()> + Send>>;

/// The daemon as the service sees it. Implemented for the gRPC client and by test fakes.
pub trait Daemon: Send {
    /// (Re)open the connection to the daemon.
    fn open(&mut self) -> impl Future<Output = Result<(), String>> + Send;
    /// Subscribe to tunnel events. Must be called before `tunnel_state` so no transition is lost.
    fn subscribe(&mut self) -> impl Future<Output = Result<EventStream, String>> + Send;
    fn tunnel_state(&mut self) -> impl Future<Output = Result<TunnelState, String>> + Send;
    fn connect(&mut self) -> impl Future<Output = Result<(), String>> + Send;
    fn disconnect(&mut self) -> impl Future<Output = Result<(), String>> + Send;
}

/// Where effects go: the D-Bus interface in production, a vector in tests.
pub trait Sink: Send {
    fn emit(&self, effect: &Effect) -> impl Future<Output = ()> + Send;
}

/// NetworkManager as the service sees it. Implemented over D-Bus and by test fakes.
pub trait Nm: Send {
    /// Watch the active connection of the profile `uuid`. The stream yields when NetworkManager
    /// starts deactivating it, which only happens when the user disconnects it.
    fn watch_deactivation(
        &mut self,
        uuid: &str,
    ) -> impl Future<Output = Result<DeactivationStream, String>> + Send;
}

pub struct Service<D, S, N> {
    machine: Machine,
    state: Arc<AtomicU32>,
    daemon: D,
    sink: S,
    nm: N,
    events: EventStream,
    deactivation: DeactivationStream,
    user_disconnect: bool,
    last_daemon_state: Option<TunnelState>,
    disconnect_deadline: Option<Instant>,
    idle_since: Instant,
}

impl<D: Daemon, S: Sink, N: Nm> Service<D, S, N> {
    pub fn new(daemon: D, sink: S, nm: N, state: Arc<AtomicU32>) -> Self {
        Self {
            machine: Machine::new(),
            state,
            daemon,
            sink,
            nm,
            events: no_stream(),
            deactivation: no_stream(),
            user_disconnect: false,
            last_daemon_state: None,
            disconnect_deadline: None,
            idle_since: Instant::now(),
        }
    }

    fn is_idle(&self) -> bool {
        matches!(self.machine.phase(), Phase::Idle | Phase::Stopped)
    }

    async fn apply(&mut self, effects: Vec<Effect>) {
        for effect in effects {
            if let Effect::StateChanged(state) = effect {
                self.state.store(state, Ordering::SeqCst);
            }
            tracing::debug!("emit {effect:?}");
            self.sink.emit(&effect).await;
        }
        if self.is_idle() {
            self.events = no_stream();
            self.deactivation = no_stream();
            self.user_disconnect = false;
            self.disconnect_deadline = None;
            self.idle_since = Instant::now();
        }
    }

    async fn fail_connect(&mut self, reason: String) -> Result<(), String> {
        tracing::error!("connect failed: {reason}");
        self.apply(vec![Effect::Failure(failure::CONNECT_FAILED)])
            .await;
        let effects = self.machine.finish_stop();
        self.apply(effects).await;
        Err(reason)
    }

    async fn watch_deactivation(&mut self, uuid: Option<String>) {
        self.user_disconnect = false;
        self.deactivation = match uuid {
            Some(uuid) => match rpc(
                "watch NetworkManager connection",
                self.nm.watch_deactivation(&uuid),
            )
            .await
            {
                Ok(stream) => stream,
                Err(err) => {
                    tracing::warn!(
                        "Cannot watch NetworkManager connection {uuid}, Disconnect will leave the daemon running: {err}"
                    );
                    no_stream()
                }
            },
            None => {
                tracing::warn!(
                    "Connect without a connection UUID, Disconnect will leave the daemon running"
                );
                no_stream()
            }
        };
    }

    /// Pick up a deactivation NetworkManager signalled before its `Disconnect` call. zbus queues
    /// the signal before it dispatches the later method call, so it is ready to be polled here.
    fn drain_deactivation(&mut self) {
        let mut cx = Context::from_waker(Waker::noop());
        loop {
            match self.deactivation.as_mut().poll_next(&mut cx) {
                Poll::Ready(Some(())) => self.user_disconnect = true,
                Poll::Ready(None) => {
                    self.deactivation = no_stream();
                    break;
                }
                Poll::Pending => break,
            }
        }
    }

    #[tracing::instrument(skip(self))]
    async fn handle_connect(&mut self, uuid: Option<String>) -> Result<(), String> {
        let effects = self.machine.begin_connect();
        self.apply(effects).await;
        self.watch_deactivation(uuid).await;

        if let Err(err) = rpc("open", self.daemon.open()).await {
            return self
                .fail_connect(format!("cannot reach nym-vpnd: {err}"))
                .await;
        }
        match rpc("ListenToEvents", self.daemon.subscribe()).await {
            Ok(events) => self.events = events,
            Err(err) => {
                return self
                    .fail_connect(format!("cannot subscribe to events: {err}"))
                    .await;
            }
        }
        let state = match rpc("GetTunnelState", self.daemon.tunnel_state()).await {
            Ok(state) => state,
            Err(err) => {
                return self
                    .fail_connect(format!("cannot read tunnel state: {err}"))
                    .await;
            }
        };
        tracing::info!("Tunnel state at connect: {state}");
        match &state {
            TunnelState::Connected { .. } | TunnelState::Offline { reconnect: true } => {
                let effects = self.machine.on_daemon_state(&state);
                self.apply(effects).await;
            }
            TunnelState::Connecting { .. } => {}
            _ => {
                if let Err(err) = rpc("ConnectTunnel", self.daemon.connect()).await {
                    // Keep STARTING: NetworkManager's own connect timeout decides the outcome.
                    tracing::warn!("ConnectTunnel failed: {err}");
                }
            }
        }
        self.last_daemon_state = Some(state.clone());

        match self.machine.phase() {
            Phase::Starting | Phase::Started => Ok(()),
            _ => Err(format!("Daemon cannot connect right now: {state}")),
        }
    }

    #[tracing::instrument(skip(self))]
    async fn handle_disconnect(&mut self) {
        if self.is_idle() {
            // NetworkManager calls Disconnect on every transition to FAILED/DISCONNECTED, including
            // right after our own STOPPED. Nothing is ours to tear down then.
            tracing::debug!("Disconnect while {:?}: nothing to do", self.machine.phase());
            return;
        }

        let daemon_state = match rpc("GetTunnelState", self.daemon.tunnel_state()).await {
            Ok(state) => Some(state),
            Err(err) => {
                tracing::warn!("cannot read tunnel state on disconnect: {err}");
                self.last_daemon_state.clone()
            }
        };

        let Some(daemon_state) = daemon_state else {
            let effects = self.machine.finish_stop();
            self.apply(effects).await;
            return;
        };
        self.drain_deactivation();

        if self.user_disconnect {
            let effects = self.machine.begin_disconnect();
            self.apply(effects).await;
            match rpc("DisconnectTunnel", self.daemon.disconnect()).await {
                Ok(()) => self.disconnect_deadline = Some(Instant::now() + DISCONNECT_WAIT),
                Err(err) => {
                    tracing::warn!("DisconnectTunnel failed: {err}");
                    let effects = self.machine.finish_stop();
                    self.apply(effects).await;
                }
            }
        } else {
            tracing::info!(
                "NetworkManager stopped the VPN without a user request, leaving the daemon {daemon_state}"
            );
            let effects = self.machine.finish_stop();
            self.apply(effects).await;
        }
    }

    async fn handle_event(&mut self, event: Option<TunnelEvent>) {
        match event {
            Some(TunnelEvent::NewState(state)) => {
                tracing::info!("New tunnel state: {state}");
                let effects = self.machine.on_daemon_state(&state);
                self.last_daemon_state = Some(state);
                self.apply(effects).await;
            }
            Some(_) => {}
            None => {
                tracing::warn!("Tunnel event stream closed");
                self.events = no_stream();
                let effects = self.machine.on_stream_closed();
                self.apply(effects).await;
            }
        }
    }

    /// Serve commands until the idle timer fires or the command channel closes.
    pub async fn run(mut self, mut rx: mpsc::Receiver<Command>) -> anyhow::Result<()> {
        loop {
            let idle = self.is_idle();
            let idle_at = self.idle_since + IDLE_EXIT;
            let deadline = self.disconnect_deadline;
            tokio::select! {
                cmd = rx.recv() => match cmd {
                    Some(Command::Connect { uuid, reply }) => {
                        let result = self.handle_connect(uuid).await;
                        let _ = reply.send(result);
                    }
                    Some(Command::Disconnect(reply)) => {
                        self.handle_disconnect().await;
                        let _ = reply.send(());
                    }
                    None => return Ok(()),
                },
                event = self.events.next() => {
                    self.handle_event(event).await;
                }
                deactivated = self.deactivation.next() => match deactivated {
                    Some(()) => self.user_disconnect = true,
                    None => self.deactivation = no_stream(),
                },

                _ = sleep_until(deadline.unwrap_or_else(Instant::now)), if deadline.is_some() => {
                    tracing::warn!("Daemon did not reach a terminal state within {DISCONNECT_WAIT:?}");
                    self.disconnect_deadline = None;
                    let effects = self.machine.finish_stop();
                    self.apply(effects).await;
                }
                _ = sleep_until(idle_at), if idle => {
                    tracing::info!("idle for {IDLE_EXIT:?}, exiting");
                    return Ok(());
                }
            }
        }
    }
}

/// `Daemon` over the gRPC socket.
#[derive(Default)]
pub struct GrpcDaemon {
    client: Option<RpcClient>,
}

impl GrpcDaemon {
    fn client(&mut self) -> Result<&mut RpcClient, String> {
        self.client
            .as_mut()
            .ok_or_else(|| "not connected to nym-vpnd".to_string())
    }
}

impl Daemon for GrpcDaemon {
    async fn open(&mut self) -> Result<(), String> {
        self.client = Some(RpcClient::new().await.map_err(|e| e.to_string())?);
        Ok(())
    }

    async fn subscribe(&mut self) -> Result<EventStream, String> {
        let stream = self
            .client()?
            .listen_to_events()
            .await
            .map_err(|e| e.to_string())?;
        // A transport error ends the stream; the service treats that like a daemon restart.
        Ok(Box::pin(
            stream
                .take_while(|item| item.is_ok())
                .filter_map(Result::ok),
        ))
    }

    async fn tunnel_state(&mut self) -> Result<TunnelState, String> {
        self.client()?
            .get_tunnel_state()
            .await
            .map_err(|e| e.to_string())
    }

    async fn connect(&mut self) -> Result<(), String> {
        self.client()?
            .connect_tunnel()
            .await
            .map(|accepted| tracing::debug!("ConnectTunnel accepted: {accepted}"))
            .map_err(|e| e.to_string())
    }

    async fn disconnect(&mut self) -> Result<(), String> {
        self.client()?
            .disconnect_tunnel()
            .await
            .map(|accepted| tracing::debug!("DisconnectTunnel accepted: {accepted}"))
            .map_err(|e| e.to_string())
    }
}

impl Sink for InterfaceRef<VpnPlugin> {
    async fn emit(&self, effect: &Effect) {
        let result = match effect {
            Effect::StateChanged(state) => self.service_state_changed(*state).await,
            Effect::Failure(reason) => self.failure(*reason).await,
            Effect::Config(gateway) => self.config(config_dict(*gateway)).await,
        };
        if let Err(err) = result {
            tracing::error!("failed to emit {effect:?}: {err}");
        }
    }
}

/// `Nm` over the system bus.
pub struct DbusNm {
    connection: zbus::Connection,
}

impl DbusNm {
    pub fn new(connection: zbus::Connection) -> Self {
        Self { connection }
    }

    async fn find_active_connection(
        &self,
        uuid: &str,
    ) -> zbus::Result<Option<ActiveConnectionProxy<'static>>> {
        let network_manager = NetworkManagerProxy::builder(&self.connection)
            .cache_properties(zbus::proxy::CacheProperties::No)
            .build()
            .await?;
        for path in network_manager.active_connections().await? {
            let active = ActiveConnectionProxy::builder(&self.connection)
                .path(path)?
                .cache_properties(zbus::proxy::CacheProperties::No)
                .build()
                .await?;
            if active.uuid().await.is_ok_and(|candidate| candidate == uuid) {
                return Ok(Some(active));
            }
        }
        Ok(None)
    }
}

impl Nm for DbusNm {
    async fn watch_deactivation(&mut self, uuid: &str) -> Result<DeactivationStream, String> {
        let active = self
            .find_active_connection(uuid)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("no active connection with UUID {uuid}"))?;
        let changes = active
            .receive_state_changed()
            .await
            .map_err(|e| e.to_string())?;
        Ok(Box::pin(changes.filter_map(|signal| {
            let args = signal.args().ok()?;
            (args.state == ACTIVE_CONNECTION_STATE_DEACTIVATING).then(|| {
                tracing::info!(
                    "NetworkManager is deactivating the VPN on request (reason {})",
                    args.reason
                );
            })
        })))
    }
}

/// `Config["gateway"]` as NetworkManager reads it: IPv4 as a `u32` whose in-memory bytes are the
/// address in network order (`in_addr` read as a host `u32`), IPv6 as the 16 address bytes (`ay`).
fn nm_gateway(addr: IpAddr) -> OwnedValue {
    match addr {
        IpAddr::V4(v4) => OwnedValue::from(u32::from_ne_bytes(v4.octets())),
        IpAddr::V6(v6) => OwnedValue::try_from(Value::from(v6.octets().to_vec()))
            .expect("byte arrays never contain file descriptors"),
    }
}

/// The `Config` signal payload: gateway only, no tun device, no IP families.
fn config_dict(gateway: IpAddr) -> HashMap<String, OwnedValue> {
    HashMap::from([
        ("gateway".to_string(), nm_gateway(gateway)),
        ("has-ip4".to_string(), OwnedValue::from(false)),
        ("has-ip6".to_string(), OwnedValue::from(false)),
        ("can-persist".to_string(), OwnedValue::from(false)),
    ])
}

/// A stream that never yields: stands in for a subscription we don't have. Replacing a real
/// stream with it drops that stream, which ends the subscription.
fn no_stream<T: Send + 'static>() -> Pin<Box<dyn Stream<Item = T> + Send>> {
    Box::pin(tokio_stream::pending())
}

/// Bound a daemon RPC so a wedged daemon cannot pin the plugin (and NetworkManager's 10 s method
/// timeout) in STARTING forever.
async fn rpc<T>(what: &str, fut: impl Future<Output = Result<T, String>>) -> Result<T, String> {
    match tokio::time::timeout(RPC_TIMEOUT, fut).await {
        Ok(result) => result,
        Err(_) => Err(format!("{what} timed out after {RPC_TIMEOUT:?}")),
    }
}
