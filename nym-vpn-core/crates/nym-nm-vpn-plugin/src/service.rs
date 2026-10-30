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
    nm::{NetworkManagerProxy, failure},
    plugin::{Command, VpnPlugin, VpnPluginSignals},
    state::{DisconnectDecision, Effect, Machine, Phase, config_dict, disconnect_decision},
};

/// How long the plugin stays alive once idle before letting NetworkManager respawn it later.
pub const IDLE_EXIT: Duration = Duration::from_secs(20);
/// How long to wait for the daemon to report a terminal state after `DisconnectTunnel`.
pub const DISCONNECT_WAIT: Duration = Duration::from_secs(15);
/// Upper bound on any single daemon RPC; NetworkManager gives our method handlers 10 s in total.
pub const RPC_TIMEOUT: Duration = Duration::from_secs(5);

pub type EventStream = Pin<Box<dyn Stream<Item = TunnelEvent> + Send>>;

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

/// NetworkManager's overall state, used to tell a user disconnect from a network-loss one.
pub trait NmState: Send {
    fn state(&self) -> impl Future<Output = Option<u32>> + Send;
}

pub struct Service<D, S, N> {
    machine: Machine,
    state: Arc<AtomicU32>,
    daemon: D,
    sink: S,
    nm: N,
    events: Option<EventStream>,
    last_daemon_state: Option<TunnelState>,
    disconnect_deadline: Option<Instant>,
    idle_since: Instant,
}

impl<D: Daemon, S: Sink, N: NmState> Service<D, S, N> {
    pub fn new(daemon: D, sink: S, nm: N, state: Arc<AtomicU32>) -> Self {
        Self {
            machine: Machine::new(),
            state,
            daemon,
            sink,
            nm,
            events: None,
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
            self.events = None;
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

    async fn handle_connect(&mut self) -> Result<(), String> {
        let effects = self.machine.begin_connect();
        self.apply(effects).await;

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
        // The current state is a snapshot from before our request, so only `Connected` means
        // anything to the machine; a terminal state here is the normal starting point. A daemon
        // that is connecting or waiting to reconnect is already on its way up.
        match &state {
            TunnelState::Connected { .. } => {
                let effects = self.machine.on_daemon_state(&state);
                self.apply(effects).await;
            }
            TunnelState::Connecting { .. } | TunnelState::Offline { reconnect: true } => {}
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
        let nm_state = self.nm.state().await;
        match disconnect_decision(&daemon_state, nm_state) {
            DisconnectDecision::AlreadyStopped => {
                tracing::info!("daemon already {daemon_state}; reporting stopped");
                let effects = self.machine.finish_stop();
                self.apply(effects).await;
            }
            DisconnectDecision::LeaveDaemon => {
                tracing::info!(
                    "NetworkManager state {} < 50: network loss or sleep, leaving the daemon connected",
                    nm_state.unwrap_or_default()
                );
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
                self.last_daemon_state = Some(state);
                self.apply(effects).await;
            }
            Some(_) => {}
            None => {
                tracing::warn!("daemon event stream closed");
                self.events = None;
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
                    Some(Command::Connect(reply)) => {
                        let result = self.handle_connect().await;
                        let _ = reply.send(result);
                    }
                    Some(Command::Disconnect(reply)) => {
                        self.handle_disconnect().await;
                        let _ = reply.send(());
                    }
                    None => return Ok(()),
                },
                event = next_event(&mut self.events), if self.events.is_some() => {
                    self.handle_event(event).await;
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

async fn next_event(events: &mut Option<EventStream>) -> Option<TunnelEvent> {
    match events {
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

impl NmState for Option<NetworkManagerProxy<'static>> {
    async fn state(&self) -> Option<u32> {
        match self {
            Some(proxy) => proxy.state().await.ok(),
            None => None,
        }
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

    impl NmState for Option<u32> {
        async fn state(&self) -> Option<u32> {
            *self
        }
    }

    struct Harness {
        tx: mpsc::Sender<Command>,
        events: Option<mpsc::Sender<TunnelEvent>>,
        sink: FakeSink,
        daemon: FakeDaemon,
        state: Arc<AtomicU32>,
        task: tokio::task::JoinHandle<anyhow::Result<()>>,
    }

    impl Harness {
        fn start(reachable: bool, initial: TunnelState, nm_state: Option<u32>) -> Self {
            Self::start_with(reachable, false, initial, nm_state)
        }

        fn start_with(
            reachable: bool,
            hang: bool,
            initial: TunnelState,
            nm_state: Option<u32>,
        ) -> Self {
            let (events, events_rx) = mpsc::channel(8);
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
            let service = Service::new(daemon.clone(), sink.clone(), nm_state, state.clone());
            let task = tokio::spawn(service.run(rx));
            Self {
                tx,
                events: Some(events),
                sink,
                daemon,
                state,
                task,
            }
        }

        async fn connect(&self) -> Result<(), String> {
            let (reply, result) = oneshot::channel();
            self.tx.send(Command::Connect(reply)).await.unwrap();
            result.await.unwrap()
        }

        async fn disconnect(&self) {
            let (reply, ack) = oneshot::channel();
            self.tx.send(Command::Disconnect(reply)).await.unwrap();
            ack.await.unwrap();
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
    }

    #[tokio::test]
    async fn connect_failure_when_daemon_unreachable() {
        let h = Harness::start(false, TunnelState::Disconnected, Some(70));
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
        let h = Harness::start(true, connected(V4, None), Some(70));
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
        let h = Harness::start(true, TunnelState::Disconnected, Some(70));
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
        let mut h = Harness::start(true, connected(V4, None), Some(70));
        h.connect().await.unwrap();
        h.close_events();
        h.settle().await;
        assert_eq!(
            h.effects().last(),
            Some(&Effect::StateChanged(service_state::STOPPED))
        );
    }

    #[tokio::test]
    async fn disconnect_leaves_daemon_when_nm_not_connected() {
        let h = Harness::start(true, connected(V4, None), Some(10));
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
    async fn disconnect_waits_for_terminal_event() {
        let h = Harness::start(true, connected(V4, None), Some(70));
        h.connect().await.unwrap();
        h.disconnect().await;
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
    }

    #[tokio::test]
    async fn disconnect_when_daemon_already_down_reports_stopped_without_rpc() {
        let h = Harness::start(true, connected(V4, None), Some(70));
        h.connect().await.unwrap();
        *h.daemon.state.lock().unwrap() = TunnelState::Disconnected;
        h.disconnect().await;
        h.settle().await;
        assert_eq!(h.daemon.disconnect_calls.load(Ordering::SeqCst), 0);
        assert_eq!(
            h.effects()[3..],
            [Effect::StateChanged(service_state::STOPPED)]
        );
    }

    #[tokio::test]
    async fn offline_recovery_keeps_started_and_event_stream() {
        let h = Harness::start(true, connected(V4, None), Some(70));
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
    async fn disconnect_during_offline_recovery_disconnects_daemon() {
        let h = Harness::start(true, connected(V4, None), Some(70));
        h.connect().await.unwrap();
        h.send(offline(true)).await;
        h.settle().await;
        *h.daemon.state.lock().unwrap() = offline(true);
        h.disconnect().await;
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
    async fn disconnect_during_offline_recovery_on_network_loss_leaves_daemon() {
        let h = Harness::start(true, connected(V4, None), Some(20));
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
    async fn connect_during_offline_recovery_waits_for_daemon() {
        let h = Harness::start(true, offline(true), Some(70));
        assert!(h.connect().await.is_ok());
        assert_eq!(h.daemon.connect_calls.load(Ordering::SeqCst), 0);
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
    }

    #[tokio::test]
    async fn disconnect_after_bad_ip_config_leaves_daemon() {
        let h = Harness::start(
            true,
            connected("[2001:db8::1]:51820".parse().unwrap(), None),
            Some(70),
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
        let h = Harness::start_with(true, true, TunnelState::Disconnected, Some(70));
        let (reply, result) = oneshot::channel();
        h.tx.send(Command::Connect(reply)).await.unwrap();
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
    async fn disconnect_times_out_to_stopped() {
        let h = Harness::start(true, connected(V4, None), Some(70));
        h.connect().await.unwrap();
        h.disconnect().await;
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
        let h = Harness::start(true, connected(V4, None), Some(70));
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
        let h = Harness::start(true, TunnelState::Disconnected, Some(70));
        tokio::time::advance(IDLE_EXIT + std::time::Duration::from_secs(1)).await;
        h.settle().await;
        assert!(h.task.is_finished());
    }
}
