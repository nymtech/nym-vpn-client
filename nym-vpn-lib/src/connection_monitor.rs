// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::{Duration, Instant};

use futures::{channel::mpsc, StreamExt};
use nym_task::TaskClient;
use tokio::task::JoinHandle;
use tracing::{debug, error, trace};

use crate::error::Result;

const CONNECTION_MONITOR_REPORT_INTERVAL: Duration = Duration::from_secs(5);

// When the latest successful ping is older than this, we consider the connection to the
// entry gateway as down
const SELF_PING_EXPIRY: Duration = Duration::from_secs(5);

// When the latest successful ping is older than these, we consider the connection to the IPR tun
// device on the exit router as down
const IPR_TUN_DEVICE_PING_REPLY_EXPIRY: Duration = Duration::from_secs(5);
const IPR_EXTERNAL_PING_REPLY_EXPIRY: Duration = Duration::from_secs(5);

#[derive(Debug)]
pub enum ConnectionStatusEvent {
    MixnetSelfPing,
    IcmpIprTunDevicePingReply,
    Icmpv6IprTunDevicePingReply,
    IcmpIprExternalPingReply,
    Icmpv6IprExternalPingReply,
}

#[derive(Debug, Default)]
struct ConnectionStats {
    // TODO: extend with all sorts of good stuff
    latest_self_ping: Option<Instant>,
    latest_ipr_tun_device_ping_reply: Option<Instant>,
    latest_ipr_tun_device_ping_v6_reply: Option<Instant>,
    latest_ipr_external_ping_reply: Option<Instant>,
    latest_ipr_external_ping_v6_reply: Option<Instant>,
}

struct ConnectionMonitor {
    connection_event_rx: mpsc::UnboundedReceiver<ConnectionStatusEvent>,
    stats: ConnectionStats,
}

enum ConnectionState {
    EntryTimeout,
    ExitTimeout,
    ExitRoutingError,
    Connected,
}

impl ConnectionState {
    fn map_from_stats(stats: &ConnectionStats) -> Self {
        if let Some(latest_self_ping) = stats.latest_self_ping {
            if latest_self_ping.elapsed() > SELF_PING_EXPIRY {
                return ConnectionState::EntryTimeout;
            }
        }
        if let Some(latest_ipr_tun_device_ping_reply) = stats.latest_ipr_tun_device_ping_reply {
            if latest_ipr_tun_device_ping_reply.elapsed() > IPR_TUN_DEVICE_PING_REPLY_EXPIRY {
                return ConnectionState::ExitTimeout;
            }
        }
        if let Some(latest_ipr_external_ping_reply) = stats.latest_ipr_external_ping_reply {
            if latest_ipr_external_ping_reply.elapsed() < IPR_EXTERNAL_PING_REPLY_EXPIRY {
                return ConnectionState::ExitRoutingError;
            }
        }
        ConnectionState::Connected
    }

    fn map_from_stats_v6(stats: &ConnectionStats) -> Self {
        if let Some(latest_self_ping) = stats.latest_self_ping {
            if latest_self_ping.elapsed() > SELF_PING_EXPIRY {
                return ConnectionState::EntryTimeout;
            }
        }
        if let Some(latest_ipr_tun_device_ping_v6_reply) = stats.latest_ipr_tun_device_ping_v6_reply
        {
            if latest_ipr_tun_device_ping_v6_reply.elapsed() > IPR_TUN_DEVICE_PING_REPLY_EXPIRY {
                return ConnectionState::ExitTimeout;
            }
        }
        if let Some(latest_ipr_external_ping_v6_reply) = stats.latest_ipr_external_ping_v6_reply {
            if latest_ipr_external_ping_v6_reply.elapsed() < IPR_EXTERNAL_PING_REPLY_EXPIRY {
                return ConnectionState::ExitRoutingError;
            }
        }
        ConnectionState::Connected
    }
}

impl ConnectionMonitor {
    fn new(connection_event_rx: mpsc::UnboundedReceiver<ConnectionStatusEvent>) -> Self {
        ConnectionMonitor {
            connection_event_rx,
            stats: ConnectionStats::default(),
        }
    }

    fn report_on_v4_state(&self, state: &ConnectionState, task_client: &mut TaskClient) {
        match state {
            ConnectionState::EntryTimeout => {
                error!("Entry gateway not routing our mixnet traffic");
                task_client.send_status_msg(Box::new(ConnectionMonitorStatus::EntryGatewayDown));
            }
            ConnectionState::ExitTimeout => {
                error!("Exit IPR not responding to IP traffic");
                task_client.send_status_msg(Box::new(ConnectionMonitorStatus::ExitGatewayDown));
            }
            ConnectionState::ExitRoutingError => {
                error!("Exit IPR not routing IP traffic to external destinations");
                task_client
                    .send_status_msg(Box::new(ConnectionMonitorStatus::ExitGatewayRoutingError));
            }
            ConnectionState::Connected => {
                debug!("ConnectionMonitor: All IPv4 connections are up");
                task_client.send_status_msg(Box::new(ConnectionMonitorStatus::AllConnectionsUp));
            }
        }
    }

    // This is a separate from the v4 report above since we don't want to be as harsh about the
    // potential lack of v6 connectivity
    fn report_on_v6_state(&self, state: &ConnectionState, task_client: &mut TaskClient) {
        match state {
            ConnectionState::EntryTimeout => {
                // Handled in the v4 report
            }
            ConnectionState::ExitTimeout => {
                error!("Exit IPR not responding to IPv6 traffic");
                task_client.send_status_msg(Box::new(ConnectionMonitorStatus::ExitGatewayDown6));
            }
            ConnectionState::ExitRoutingError => {
                error!("Exit IPR not routing IPv6 traffic to external destinations");
                task_client
                    .send_status_msg(Box::new(ConnectionMonitorStatus::ExitGatewayRoutingError6));
            }
            ConnectionState::Connected => {
                debug!("ConnectionMonitor: All IPv6 connections are up");
                task_client.send_status_msg(Box::new(ConnectionMonitorStatus::AllConnectionsUp));
            }
        }
    }

    async fn run(mut self, mut task_client: TaskClient) -> Result<()> {
        debug!("Connection monitor is running");
        let mut report_interval = tokio::time::interval(CONNECTION_MONITOR_REPORT_INTERVAL);
        // Reset so that we don't send a report immediately before we even have a change for any
        // self pings to be sent and received
        report_interval.reset();

        loop {
            tokio::select! {
                _ = task_client.recv() => {
                    trace!("ConnectionMonitor: Received shutdown");
                    break;
                }
                Some(event) = self.connection_event_rx.next() => {
                    match event {
                        ConnectionStatusEvent::MixnetSelfPing => {
                            trace!("Received self ping event");
                            self.stats.latest_self_ping = Some(Instant::now());
                        }
                        ConnectionStatusEvent::IcmpIprTunDevicePingReply => {
                            trace!("Received IPR tun device ping reply event");
                            self.stats.latest_ipr_tun_device_ping_reply = Some(Instant::now());
                        }
                        ConnectionStatusEvent::Icmpv6IprTunDevicePingReply => {
                            trace!("Received IPR tun device ping v6 reply event");
                            self.stats.latest_ipr_tun_device_ping_v6_reply = Some(Instant::now());
                        }
                        ConnectionStatusEvent::IcmpIprExternalPingReply => {
                            trace!("Received IPR external ping reply event");
                            self.stats.latest_ipr_external_ping_reply = Some(Instant::now());
                        }
                        ConnectionStatusEvent::Icmpv6IprExternalPingReply => {
                            trace!("Received IPR external ping v6 reply event");
                            self.stats.latest_ipr_external_ping_v6_reply = Some(Instant::now());
                        }
                    }
                }
                _ = report_interval.tick() => {
                    debug!(
                        "Time since latest received self ping: {}ms",
                        self.stats.latest_self_ping.map(|t| t.elapsed().as_millis()).unwrap_or(0)
                    );
                    debug!(
                        "Time since latest received ipr tun device ping reply: {}ms",
                        self.stats.latest_ipr_tun_device_ping_reply.map(|t| t.elapsed().as_millis()).unwrap_or(0)
                    );
                    debug!(
                        "Time since latest received ipr tun device ping v6 reply: {}ms",
                        self.stats.latest_ipr_tun_device_ping_v6_reply.map(|t| t.elapsed().as_millis()).unwrap_or(0)
                    );
                    debug!(
                        "Time since latest received ipr external ping reply: {}ms",
                        self.stats.latest_ipr_external_ping_reply.map(|t| t.elapsed().as_millis()).unwrap_or(0)
                    );
                    debug!(
                        "Time since latest received ipr external ping v6 reply: {}ms",
                        self.stats.latest_ipr_external_ping_v6_reply.map(|t| t.elapsed().as_millis()).unwrap_or(0)
                    );

                    let ipv4_state = ConnectionState::map_from_stats(&self.stats);
                    let ipv6_state = ConnectionState::map_from_stats_v6(&self.stats);

                    match (&ipv4_state, &ipv6_state) {
                        (ConnectionState::Connected, ConnectionState::Connected) => {
                            debug!("ConnectionMonitor: All connections are up");
                            task_client.send_status_msg(Box::new(ConnectionMonitorStatus::AllConnectionsUp));
                        }
                        _ => {
                            self.report_on_v4_state(&ipv4_state, &mut task_client);
                            self.report_on_v6_state(&ipv6_state, &mut task_client);
                        }
                    }
                }
            }
        }
        debug!("ConnectionMonitor: Exiting");
        Ok(())
    }
}

// Just like in nym_task::TaskManager and TaskStatus, strictly speaking this is not an error, but a
// status message. We're just piggybacking on the error trait for now. In the future, we might want
// to create a separate trait in nym_task::TaskManager
#[derive(thiserror::Error, Debug)]
pub enum ConnectionMonitorStatus {
    #[error("entry gateway appears down - it's not routing our mixnet traffic")]
    EntryGatewayDown,

    #[error("exit gateway (or ipr) appears down - it's not responding to IP traffic")]
    ExitGatewayDown,

    #[error("exit gateway (or ipr) appears down - it's not responding to IPv6 traffic")]
    ExitGatewayDown6,

    #[error("exit gateway (or ipr) appears to be having issues routing and forwarding our external IP traffic")]
    ExitGatewayRoutingError,

    #[error("exit gateway (or ipr) appears to be having issues routing and forwarding our external IPv6 traffic")]
    ExitGatewayRoutingError6,

    #[error("all connections are up")]
    AllConnectionsUp,
}

pub fn start_connection_monitor(
    connection_event_rx: futures::channel::mpsc::UnboundedReceiver<ConnectionStatusEvent>,
    shutdown_listener: TaskClient,
) -> JoinHandle<Result<()>> {
    debug!("Creating connection monitor");
    let monitor = ConnectionMonitor::new(connection_event_rx);
    tokio::spawn(async move {
        monitor.run(shutdown_listener).await.inspect_err(|err| {
            error!("Connection monitor error: {err}");
        })
    })
}
