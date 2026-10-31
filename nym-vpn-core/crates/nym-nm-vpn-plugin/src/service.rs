// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! The task that owns the daemon connection and turns NetworkManager commands and daemon events
//! into the signals decided by [`crate::state::Machine`].

use std::{
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
use zbus::object_server::InterfaceRef;

use crate::{
    nm::{
        ACTIVE_CONNECTION_STATE_DEACTIVATING, ActiveConnectionProxy, NetworkManagerProxy,
        SettingsProxy, failure,
    },
    plugin::{Command, VpnPlugin, VpnPluginSignals},
    state::{
        DisconnectDecision, Effect, Machine, Phase, config_dict, disconnect_decision, is_terminal,
    },
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
    /// Ask NetworkManager to activate the profile `uuid`.
    fn activate(&mut self, uuid: &str) -> impl Future<Output = Result<(), String>> + Send;
}

pub struct Service<D, S, N> {
    machine: Machine,
    state: Arc<AtomicU32>,
    daemon: D,
    sink: S,
    nm: N,
    events: Option<EventStream>,
    deactivation: Option<DeactivationStream>,
    user_disconnect: bool,
    /// UUID of the profile NetworkManager activated last.
    uuid: Option<String>,
    /// Set after NetworkManager tore the VPN down on its own while the daemon stayed up: the
    /// profile to activate again once the daemon reconnects.
    reactivate: Option<String>,
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
            events: None,
            deactivation: None,
            user_disconnect: false,
            uuid: None,
            reactivate: None,
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
            if self.reactivate.is_none() {
                self.events = None;
            }
            self.deactivation = None;
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
                Ok(stream) => Some(stream),
                Err(err) => {
                    tracing::warn!(
                        "cannot watch NetworkManager connection {uuid}, Disconnect will leave the daemon running: {err}"
                    );
                    None
                }
            },
            None => {
                tracing::warn!(
                    "Connect without a connection UUID, Disconnect will leave the daemon running"
                );
                None
            }
        };
    }

    /// Pick up a deactivation NetworkManager signalled before its `Disconnect` call. zbus queues
    /// the signal before it dispatches the later method call, so it is ready to be polled here.
    fn drain_deactivation(&mut self) {
        let Some(stream) = self.deactivation.as_mut() else {
            return;
        };
        let mut cx = Context::from_waker(Waker::noop());
        let mut deactivated = false;
        let mut closed = false;
        loop {
            match stream.as_mut().poll_next(&mut cx) {
                Poll::Ready(Some(())) => deactivated = true,
                Poll::Ready(None) => {
                    closed = true;
                    break;
                }
                Poll::Pending => break,
            }
        }
        self.user_disconnect |= deactivated;
        if closed {
            self.deactivation = None;
        }
    }

    async fn handle_connect(&mut self, uuid: Option<String>) -> Result<(), String> {
        self.reactivate = None;
        self.uuid = uuid.clone();
        let effects = self.machine.begin_connect();
        self.apply(effects).await;
        self.watch_deactivation(uuid).await;

        if let Err(err) = rpc("open", self.daemon.open()).await {
            return self
                .fail_connect(format!("cannot reach nym-vpnd: {err}"))
                .await;
        }
        match rpc("ListenToEvents", self.daemon.subscribe()).await {
            Ok(events) => self.events = Some(events),
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
        tracing::info!("daemon state at connect: {state}");
        // The current state is a snapshot from before our request, so a terminal state here is
        // the normal starting point. A daemon that is connected or waiting to reconnect is
        // already set to connect.
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
            _ => Err(format!("daemon cannot connect right now: {state}")),
        }
    }

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
        match disconnect_decision(&daemon_state, self.user_disconnect) {
            DisconnectDecision::AlreadyStopped => {
                tracing::info!("daemon already {daemon_state}; reporting stopped");
                let effects = self.machine.finish_stop();
                self.apply(effects).await;
            }
            DisconnectDecision::LeaveDaemon => {
                tracing::info!(
                    "NetworkManager stopped the VPN without a user request, leaving the daemon {daemon_state}"
                );
                self.reactivate = self.uuid.clone();
                let effects = self.machine.finish_stop();
                self.apply(effects).await;
            }
            DisconnectDecision::DisconnectDaemon => {
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
            }
        }
    }

    async fn handle_event(&mut self, event: Option<TunnelEvent>) {
        match event {
            Some(TunnelEvent::NewState(state)) => {
                tracing::info!("daemon state: {state}");
                let effects = self.machine.on_daemon_state(&state);
                self.apply(effects).await;
                self.follow_recovery(&state).await;
                self.last_daemon_state = Some(state);
            }
            Some(_) => {}
            None => {
                tracing::warn!("daemon event stream closed");
                self.events = None;
                if self.reactivate.take().is_some() {
                    self.idle_since = Instant::now();
                }
                let effects = self.machine.on_stream_closed();
                self.apply(effects).await;
            }
        }
    }

    /// Activate the NetworkManager profile again once the daemon has reconnected on its own, or
    /// stop waiting when it went down for good.
    async fn follow_recovery(&mut self, state: &TunnelState) {
        let connected = matches!(state, TunnelState::Connected { .. });
        if !connected && !is_terminal(state) {
            return;
        }
        let Some(uuid) = self.reactivate.take() else {
            return;
        };
        self.events = None;
        self.idle_since = Instant::now();
        if connected {
            tracing::info!("daemon reconnected, activating NetworkManager connection {uuid}");
            if let Err(err) = rpc("ActivateConnection", self.nm.activate(&uuid)).await {
                tracing::warn!("cannot activate NetworkManager connection {uuid}: {err}");
            }
        } else {
            tracing::info!("daemon {state}, not activating NetworkManager connection {uuid}");
        }
    }

    /// Serve commands until the idle timer fires or the command channel closes.
    pub async fn run(mut self, mut rx: mpsc::Receiver<Command>) -> anyhow::Result<()> {
        loop {
            let idle = self.is_idle() && self.reactivate.is_none();
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
                event = next_item(&mut self.events), if self.events.is_some() => {
                    self.handle_event(event).await;
                }
                deactivated = next_item(&mut self.deactivation), if self.deactivation.is_some() => {
                    match deactivated {
                        Some(()) => self.user_disconnect = true,
                        None => self.deactivation = None,
                    }
                }
                _ = sleep_until(deadline.unwrap_or_else(Instant::now)), if deadline.is_some() => {
                    tracing::warn!("daemon did not reach a terminal state within {DISCONNECT_WAIT:?}");
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

/// Bound a daemon RPC so a wedged daemon cannot pin the plugin (and NetworkManager's 10 s method
/// timeout) in STARTING forever.
async fn rpc<T>(what: &str, fut: impl Future<Output = Result<T, String>>) -> Result<T, String> {
    match tokio::time::timeout(RPC_TIMEOUT, fut).await {
        Ok(result) => result,
        Err(_) => Err(format!("{what} timed out after {RPC_TIMEOUT:?}")),
    }
}

async fn next_item<T>(stream: &mut Option<Pin<Box<dyn Stream<Item = T> + Send>>>) -> Option<T> {
    match stream {
        Some(stream) => stream.next().await,
        None => std::future::pending().await,
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

    async fn activate_profile(&self, uuid: &str) -> zbus::Result<()> {
        let profile = SettingsProxy::builder(&self.connection)
            .cache_properties(zbus::proxy::CacheProperties::No)
            .build()
            .await?
            .get_connection_by_uuid(uuid)
            .await?;
        let any = zbus::zvariant::ObjectPath::from_static_str_unchecked("/");
        NetworkManagerProxy::builder(&self.connection)
            .cache_properties(zbus::proxy::CacheProperties::No)
            .build()
            .await?
            .activate_connection(&profile, &any, &any)
            .await?;
        Ok(())
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

    async fn activate(&mut self, uuid: &str) -> Result<(), String> {
        self.activate_profile(uuid).await.map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicU32, AtomicUsize, Ordering},
    };

    use nym_vpn_lib_types::{TunnelEvent, TunnelState};
    use tokio::sync::{mpsc, oneshot};

    use super::*;
    use crate::{
        nm::{failure, service_state},
        plugin::Command,
        state::{Effect, fixtures::*},
    };

    #[derive(Clone)]
    struct FakeDaemon {
        reachable: bool,
        hang: bool,
        state: Arc<Mutex<TunnelState>>,
        events: Arc<Mutex<Option<mpsc::Receiver<TunnelEvent>>>>,
        connect_calls: Arc<AtomicUsize>,
        disconnect_calls: Arc<AtomicUsize>,
    }

    impl Daemon for FakeDaemon {
        async fn open(&mut self) -> Result<(), String> {
            if self.reachable {
                Ok(())
            } else {
                Err("socket missing".into())
            }
        }

        async fn subscribe(&mut self) -> Result<EventStream, String> {
            let rx = self
                .events
                .lock()
                .unwrap()
                .take()
                .expect("subscribed twice");
            Ok(Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx)))
        }

        async fn tunnel_state(&mut self) -> Result<TunnelState, String> {
            if self.hang {
                std::future::pending::<()>().await;
            }
            Ok(self.state.lock().unwrap().clone())
        }

        async fn connect(&mut self) -> Result<(), String> {
            self.connect_calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        async fn disconnect(&mut self) -> Result<(), String> {
            self.disconnect_calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[derive(Clone, Default)]
    struct FakeSink(Arc<Mutex<Vec<Effect>>>);

    impl Sink for FakeSink {
        async fn emit(&self, effect: &Effect) {
            self.0.lock().unwrap().push(effect.clone());
        }
    }

    #[derive(Clone)]
    struct FakeNm {
        deactivation: Arc<Mutex<Option<mpsc::Receiver<()>>>>,
        activations: Arc<Mutex<Vec<String>>>,
    }

    impl Nm for FakeNm {
        async fn watch_deactivation(&mut self, _uuid: &str) -> Result<DeactivationStream, String> {
            let rx = self
                .deactivation
                .lock()
                .unwrap()
                .take()
                .ok_or("watched twice")?;
            Ok(Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx)))
        }

        async fn activate(&mut self, uuid: &str) -> Result<(), String> {
            self.activations.lock().unwrap().push(uuid.to_owned());
            Ok(())
        }
    }

    struct Harness {
        tx: mpsc::Sender<Command>,
        events: Option<mpsc::Sender<TunnelEvent>>,
        deactivate: mpsc::Sender<()>,
        activations: Arc<Mutex<Vec<String>>>,
        sink: FakeSink,
        daemon: FakeDaemon,
        state: Arc<AtomicU32>,
        task: tokio::task::JoinHandle<anyhow::Result<()>>,
    }

    impl Harness {
        fn start(reachable: bool, initial: TunnelState) -> Self {
            Self::start_with(reachable, false, initial)
        }

        fn start_with(reachable: bool, hang: bool, initial: TunnelState) -> Self {
            let (events, events_rx) = mpsc::channel(8);
            let (deactivate, deactivation_rx) = mpsc::channel(1);
            let activations = Arc::<Mutex<Vec<String>>>::default();
            let nm = FakeNm {
                deactivation: Arc::new(Mutex::new(Some(deactivation_rx))),
                activations: activations.clone(),
            };
            let daemon = FakeDaemon {
                reachable,
                hang,
                state: Arc::new(Mutex::new(initial)),
                events: Arc::new(Mutex::new(Some(events_rx))),
                connect_calls: Default::default(),
                disconnect_calls: Default::default(),
            };
            let sink = FakeSink::default();
            let state = Arc::new(AtomicU32::new(service_state::INIT));
            let (tx, rx) = mpsc::channel(4);
            let service = Service::new(daemon.clone(), sink.clone(), nm, state.clone());
            let task = tokio::spawn(service.run(rx));
            Self {
                tx,
                events: Some(events),
                deactivate,
                activations,
                sink,
                daemon,
                state,
                task,
            }
        }

        async fn connect(&self) -> Result<(), String> {
            let (reply, result) = oneshot::channel();
            let uuid = Some("4b1c".to_owned());
            self.tx
                .send(Command::Connect { uuid, reply })
                .await
                .unwrap();
            result.await.unwrap()
        }

        /// NetworkManager stopping the VPN on its own (network loss, sleep, timeout, failure).
        async fn disconnect(&self) {
            let (reply, ack) = oneshot::channel();
            self.tx.send(Command::Disconnect(reply)).await.unwrap();
            ack.await.unwrap();
        }

        /// The user disconnecting the VPN: NetworkManager signals the deactivation, then calls
        /// `Disconnect` right away.
        async fn user_disconnect(&self) {
            self.deactivate.send(()).await.unwrap();
            self.disconnect().await;
        }

        async fn send(&self, state: TunnelState) {
            self.events
                .as_ref()
                .unwrap()
                .send(TunnelEvent::NewState(state))
                .await
                .unwrap();
        }

        fn close_events(&mut self) {
            self.events.take();
        }

        async fn settle(&self) {
            for _ in 0..20 {
                tokio::task::yield_now().await;
            }
        }

        fn effects(&self) -> Vec<Effect> {
            self.sink.0.lock().unwrap().clone()
        }

        fn activations(&self) -> Vec<String> {
            self.activations.lock().unwrap().clone()
        }
    }

    #[tokio::test]
    async fn connect_failure_when_daemon_unreachable() {
        let h = Harness::start(false, TunnelState::Disconnected);
        assert!(h.connect().await.is_err());
        assert_eq!(
            h.effects(),
            vec![
                Effect::StateChanged(service_state::STARTING),
                Effect::Failure(failure::CONNECT_FAILED),
                Effect::StateChanged(service_state::STOPPED),
            ]
        );
        assert_eq!(h.state.load(Ordering::SeqCst), service_state::STOPPED);
    }

    #[tokio::test]
    async fn connect_skips_connect_tunnel_when_already_connected() {
        let h = Harness::start(true, connected(V4, None));
        assert!(h.connect().await.is_ok());
        assert_eq!(
            h.effects(),
            vec![
                Effect::StateChanged(service_state::STARTING),
                Effect::Config(GW),
                Effect::StateChanged(service_state::STARTED),
            ]
        );
        assert_eq!(h.daemon.connect_calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn connect_then_connected_event_reaches_started() {
        let h = Harness::start(true, TunnelState::Disconnected);
        assert!(h.connect().await.is_ok());
        assert_eq!(h.daemon.connect_calls.load(Ordering::SeqCst), 1);
        assert_eq!(
            h.effects(),
            vec![Effect::StateChanged(service_state::STARTING)]
        );
        h.send(connecting()).await;
        h.send(connected(V4, None)).await;
        h.settle().await;
        assert_eq!(
            h.effects(),
            vec![
                Effect::StateChanged(service_state::STARTING),
                Effect::Config(GW),
                Effect::StateChanged(service_state::STARTED),
            ]
        );
        assert_eq!(h.state.load(Ordering::SeqCst), service_state::STARTED);
    }

    #[tokio::test]
    async fn stream_closed_while_started_emits_stopped() {
        let mut h = Harness::start(true, connected(V4, None));
        h.connect().await.unwrap();
        h.close_events();
        h.settle().await;
        assert_eq!(
            h.effects().last(),
            Some(&Effect::StateChanged(service_state::STOPPED))
        );
    }

    #[tokio::test]
    async fn disconnect_without_user_request_leaves_daemon() {
        let h = Harness::start(true, connected(V4, None));
        h.connect().await.unwrap();
        h.disconnect().await;
        h.settle().await;
        assert_eq!(h.daemon.disconnect_calls.load(Ordering::SeqCst), 0);
        assert_eq!(
            h.effects()[3..],
            [Effect::StateChanged(service_state::STOPPED)]
        );
    }

    #[tokio::test]
    async fn user_disconnect_waits_for_terminal_event() {
        let h = Harness::start(true, connected(V4, None));
        h.connect().await.unwrap();
        h.user_disconnect().await;
        h.settle().await;
        assert_eq!(h.daemon.disconnect_calls.load(Ordering::SeqCst), 1);
        assert_eq!(
            h.effects()[3..],
            [Effect::StateChanged(service_state::STOPPING)]
        );
        h.send(TunnelState::Disconnected).await;
        h.settle().await;
        assert_eq!(
            h.effects()[3..],
            [
                Effect::StateChanged(service_state::STOPPING),
                Effect::StateChanged(service_state::STOPPED)
            ]
        );
        assert!(h.activations().is_empty());
    }

    #[tokio::test]
    async fn disconnect_when_daemon_already_down_reports_stopped_without_rpc() {
        let h = Harness::start(true, connected(V4, None));
        h.connect().await.unwrap();
        *h.daemon.state.lock().unwrap() = TunnelState::Disconnected;
        h.user_disconnect().await;
        h.settle().await;
        assert_eq!(h.daemon.disconnect_calls.load(Ordering::SeqCst), 0);
        assert_eq!(
            h.effects()[3..],
            [Effect::StateChanged(service_state::STOPPED)]
        );
    }

    #[tokio::test]
    async fn offline_recovery_keeps_started_and_event_stream() {
        let h = Harness::start(true, connected(V4, None));
        h.connect().await.unwrap();
        h.send(offline(true)).await;
        h.send(connecting()).await;
        h.send(connected(V4, None)).await;
        h.settle().await;
        assert_eq!(h.effects().len(), 3);
        assert_eq!(h.state.load(Ordering::SeqCst), service_state::STARTED);
        h.send(TunnelState::Disconnected).await;
        h.settle().await;
        assert_eq!(
            h.effects()[3..],
            [Effect::StateChanged(service_state::STOPPED)]
        );
    }

    #[tokio::test]
    async fn user_disconnect_during_offline_recovery_disconnects_daemon() {
        let h = Harness::start(true, connected(V4, None));
        h.connect().await.unwrap();
        h.send(offline(true)).await;
        h.settle().await;
        *h.daemon.state.lock().unwrap() = offline(true);
        h.user_disconnect().await;
        h.settle().await;
        assert_eq!(h.daemon.disconnect_calls.load(Ordering::SeqCst), 1);
        assert_eq!(
            h.effects()[3..],
            [Effect::StateChanged(service_state::STOPPING)]
        );
        h.send(offline(false)).await;
        h.settle().await;
        assert_eq!(
            h.effects()[3..],
            [
                Effect::StateChanged(service_state::STOPPING),
                Effect::StateChanged(service_state::STOPPED)
            ]
        );
    }

    #[tokio::test]
    async fn disconnect_during_offline_recovery_without_user_request_leaves_daemon() {
        let h = Harness::start(true, connected(V4, None));
        h.connect().await.unwrap();
        h.send(offline(true)).await;
        h.settle().await;
        *h.daemon.state.lock().unwrap() = offline(true);
        h.disconnect().await;
        h.settle().await;
        assert_eq!(h.daemon.disconnect_calls.load(Ordering::SeqCst), 0);
        assert_eq!(
            h.effects()[3..],
            [Effect::StateChanged(service_state::STOPPED)]
        );
    }

    #[tokio::test]
    async fn daemon_reconnect_after_network_loss_reactivates_connection() {
        let h = Harness::start(true, connected(V4, None));
        h.connect().await.unwrap();
        h.send(offline(true)).await;
        h.settle().await;
        *h.daemon.state.lock().unwrap() = offline(true);
        h.disconnect().await;
        h.send(connecting()).await;
        h.settle().await;
        assert!(h.activations().is_empty());
        h.send(connected(V4, None)).await;
        h.settle().await;
        assert_eq!(h.activations(), ["4b1c"]);
        assert_eq!(
            h.effects()[3..],
            [Effect::StateChanged(service_state::STOPPED)]
        );
    }

    #[tokio::test(start_paused = true)]
    async fn waiting_to_reactivate_keeps_the_plugin_alive() {
        let h = Harness::start(true, connected(V4, None));
        h.connect().await.unwrap();
        h.disconnect().await;
        tokio::time::advance(IDLE_EXIT * 3).await;
        h.settle().await;
        assert!(!h.task.is_finished());
        h.send(offline(true)).await;
        h.send(connected(V4, None)).await;
        h.settle().await;
        assert_eq!(h.activations(), ["4b1c"]);
    }

    #[tokio::test(start_paused = true)]
    async fn daemon_going_down_after_network_loss_cancels_reactivation() {
        let h = Harness::start(true, connected(V4, None));
        h.connect().await.unwrap();
        h.disconnect().await;
        h.send(TunnelState::Disconnected).await;
        h.settle().await;
        assert!(h.activations().is_empty());
        tokio::time::advance(IDLE_EXIT + std::time::Duration::from_secs(1)).await;
        h.settle().await;
        assert!(h.task.is_finished());
    }

    #[tokio::test]
    async fn connect_during_offline_recovery_without_gateway_fails_and_leaves_daemon() {
        let h = Harness::start(true, offline(true));
        assert!(h.connect().await.is_err());
        assert_eq!(h.daemon.connect_calls.load(Ordering::SeqCst), 0);
        assert_eq!(
            h.effects(),
            vec![
                Effect::StateChanged(service_state::STARTING),
                Effect::Failure(failure::CONNECT_FAILED),
                Effect::StateChanged(service_state::STOPPED),
            ]
        );
        h.disconnect().await;
        h.settle().await;
        assert_eq!(h.daemon.disconnect_calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn connect_then_offline_without_gateway_fails_and_leaves_daemon() {
        let h = Harness::start(true, TunnelState::Disconnected);
        assert!(h.connect().await.is_ok());
        assert_eq!(h.daemon.connect_calls.load(Ordering::SeqCst), 1);
        h.send(offline(true)).await;
        h.settle().await;
        assert_eq!(
            h.effects(),
            vec![
                Effect::StateChanged(service_state::STARTING),
                Effect::Failure(failure::CONNECT_FAILED),
                Effect::StateChanged(service_state::STOPPED),
            ]
        );
        h.disconnect().await;
        h.settle().await;
        assert_eq!(h.daemon.disconnect_calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn user_disconnect_in_error_state_disconnects_daemon() {
        let h = Harness::start(true, connected(V4, None));
        h.connect().await.unwrap();
        h.send(error()).await;
        h.settle().await;
        assert_eq!(h.state.load(Ordering::SeqCst), service_state::STARTED);
        *h.daemon.state.lock().unwrap() = error();
        h.user_disconnect().await;
        h.settle().await;
        assert_eq!(h.daemon.disconnect_calls.load(Ordering::SeqCst), 1);
        h.send(TunnelState::Disconnected).await;
        h.settle().await;
        assert_eq!(
            h.effects()[3..],
            [
                Effect::StateChanged(service_state::STOPPING),
                Effect::StateChanged(service_state::STOPPED)
            ]
        );
    }

    #[tokio::test]
    async fn user_disconnect_while_connecting_disconnects_daemon() {
        let h = Harness::start(true, TunnelState::Disconnected);
        h.connect().await.unwrap();
        *h.daemon.state.lock().unwrap() = connecting();
        h.user_disconnect().await;
        h.settle().await;
        assert_eq!(h.daemon.disconnect_calls.load(Ordering::SeqCst), 1);
        assert_eq!(
            h.effects()[1..],
            [Effect::StateChanged(service_state::STOPPING)]
        );
    }

    #[tokio::test]
    async fn disconnect_without_connection_uuid_leaves_daemon() {
        let h = Harness::start(true, connected(V4, None));
        let (reply, result) = oneshot::channel();
        h.tx.send(Command::Connect { uuid: None, reply })
            .await
            .unwrap();
        result.await.unwrap().unwrap();
        h.user_disconnect().await;
        h.settle().await;
        assert_eq!(h.daemon.disconnect_calls.load(Ordering::SeqCst), 0);
        assert_eq!(
            h.effects().last(),
            Some(&Effect::StateChanged(service_state::STOPPED))
        );
    }

    #[tokio::test]
    async fn disconnect_after_bad_ip_config_leaves_daemon() {
        let h = Harness::start(
            true,
            connected("[2001:db8::1]:51820".parse().unwrap(), None),
        );
        assert!(h.connect().await.is_err());
        assert_eq!(
            h.effects(),
            vec![
                Effect::StateChanged(service_state::STARTING),
                Effect::Failure(failure::BAD_IP_CONFIG),
                Effect::StateChanged(service_state::STOPPED),
            ]
        );
        // NetworkManager calls Disconnect on every transition to FAILED; the tunnel must stay up.
        h.disconnect().await;
        h.settle().await;
        assert_eq!(h.daemon.disconnect_calls.load(Ordering::SeqCst), 0);
        assert_eq!(h.effects().len(), 3);
    }

    #[tokio::test(start_paused = true)]
    async fn connect_times_out_when_daemon_hangs() {
        let h = Harness::start_with(true, true, TunnelState::Disconnected);
        let (reply, result) = oneshot::channel();
        h.tx.send(Command::Connect { uuid: None, reply })
            .await
            .unwrap();
        h.settle().await;
        tokio::time::advance(RPC_TIMEOUT + std::time::Duration::from_secs(1)).await;
        h.settle().await;
        assert!(result.await.unwrap().is_err());
        assert_eq!(
            h.effects(),
            vec![
                Effect::StateChanged(service_state::STARTING),
                Effect::Failure(failure::CONNECT_FAILED),
                Effect::StateChanged(service_state::STOPPED),
            ]
        );
    }

    #[tokio::test(start_paused = true)]
    async fn user_disconnect_times_out_to_stopped() {
        let h = Harness::start(true, connected(V4, None));
        h.connect().await.unwrap();
        h.user_disconnect().await;
        h.settle().await;
        tokio::time::advance(DISCONNECT_WAIT - std::time::Duration::from_secs(1)).await;
        h.settle().await;
        assert_eq!(
            h.effects().last(),
            Some(&Effect::StateChanged(service_state::STOPPING))
        );
        tokio::time::advance(std::time::Duration::from_secs(2)).await;
        h.settle().await;
        assert_eq!(
            h.effects().last(),
            Some(&Effect::StateChanged(service_state::STOPPED))
        );
    }

    #[tokio::test(start_paused = true)]
    async fn idle_exit_after_stopped() {
        let h = Harness::start(true, connected(V4, None));
        h.connect().await.unwrap();
        h.send(TunnelState::Disconnected).await;
        h.settle().await;
        assert!(!h.task.is_finished());
        tokio::time::advance(IDLE_EXIT + std::time::Duration::from_secs(1)).await;
        h.settle().await;
        assert!(h.task.is_finished());
    }

    #[tokio::test(start_paused = true)]
    async fn idle_exit_when_never_connected() {
        let h = Harness::start(true, TunnelState::Disconnected);
        tokio::time::advance(IDLE_EXIT + std::time::Duration::from_secs(1)).await;
        h.settle().await;
        assert!(h.task.is_finished());
    }
}
