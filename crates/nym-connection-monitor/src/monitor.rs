// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::{Duration, Instant};

use futures::{channel::mpsc, StreamExt};
use nym_sdk::TaskClient;
use tokio::task::JoinHandle;
use tracing::{debug, error, trace};

use crate::error::Result;

const CONNECTION_MONITOR_REPORT_INTERVAL: Duration = Duration::from_secs(5);

// When the latest successful ping is older than this, we consider the connection to be down
const PING_REPLY_EXPIRY: Duration = Duration::from_secs(5);

// Events that are reported by other tasks to the connection monitor
#[derive(Debug)]
pub enum ConnectionStatusEvent {
    MixnetSelfPing,
    Icmpv4IprTunDevicePingReply,
    Icmpv6IprTunDevicePingReply,
    Icmpv4IprExternalPingReply,
    Icmpv6IprExternalPingReply,
}

#[derive(Debug, Default)]
struct ConnectionStats {
    // TODO: extend with all sorts of good stuff
    latest_self_ping: Option<Instant>,
    latest_ipr_tun_device_ping_v4_reply: Option<Instant>,
    latest_ipr_tun_device_ping_v6_reply: Option<Instant>,
    latest_ipr_external_ping_v4_reply: Option<Instant>,
    latest_ipr_external_ping_v6_reply: Option<Instant>,
}

impl ConnectionStats {
    fn evaluate_connectivity(&self) -> ConnectivityState {
        let entry = ConnectivityStatus::from(&self.latest_self_ping);

        let exit_ipv4 = ConnectivityStatus::from(&self.latest_ipr_tun_device_ping_v4_reply);
        let exit_ipv6 = ConnectivityStatus::from(&self.latest_ipr_tun_device_ping_v6_reply);

        let exit_routing_ipv4 = ConnectivityStatus::from(&self.latest_ipr_external_ping_v4_reply);
        let exit_routing_ipv6 = ConnectivityStatus::from(&self.latest_ipr_external_ping_v6_reply);

        ConnectivityState {
            entry,
            exit: IpConnectivity {
                ipv4: exit_ipv4,
                ipv6: exit_ipv6,
            },
            exit_routing: IpConnectivity {
                ipv4: exit_routing_ipv4,
                ipv6: exit_routing_ipv6,
            },
        }
    }

    fn log_status(&self) {
        debug!(
            "Time since latest received self ping: {}ms",
            self.latest_self_ping
                .map(|t| t.elapsed().as_millis())
                .unwrap_or(0)
        );
        debug!(
            "Time since latest received ipr tun device ping v4 reply: {}ms",
            self.latest_ipr_tun_device_ping_v4_reply
                .map(|t| t.elapsed().as_millis())
                .unwrap_or(0)
        );
        debug!(
            "Time since latest received ipr tun device ping v6 reply: {}ms",
            self.latest_ipr_tun_device_ping_v6_reply
                .map(|t| t.elapsed().as_millis())
                .unwrap_or(0)
        );
        debug!(
            "Time since latest received ipr external ping v4 reply: {}ms",
            self.latest_ipr_external_ping_v4_reply
                .map(|t| t.elapsed().as_millis())
                .unwrap_or(0)
        );
        debug!(
            "Time since latest received ipr external ping v6 reply: {}ms",
            self.latest_ipr_external_ping_v6_reply
                .map(|t| t.elapsed().as_millis())
                .unwrap_or(0)
        );
    }
}

struct ConnectionMonitor {
    connection_event_rx: mpsc::UnboundedReceiver<ConnectionStatusEvent>,
    stats: ConnectionStats,
}

#[derive(Debug, PartialEq, Eq)]
enum ConnectivityStatus {
    Ok,
    Fail,
}

impl From<&Option<Instant>> for ConnectivityStatus {
    fn from(reply: &Option<Instant>) -> Self {
        match reply {
            Some(when) if when.elapsed() < PING_REPLY_EXPIRY => ConnectivityStatus::Ok,
            Some(_) => ConnectivityStatus::Fail,
            None => ConnectivityStatus::Fail,
        }
    }
}

struct IpConnectivity {
    ipv4: ConnectivityStatus,
    ipv6: ConnectivityStatus,
}

struct ConnectivityState {
    entry: ConnectivityStatus,
    exit: IpConnectivity,
    exit_routing: IpConnectivity,
}

impl ConnectionMonitor {
    fn new(connection_event_rx: mpsc::UnboundedReceiver<ConnectionStatusEvent>) -> Self {
        ConnectionMonitor {
            connection_event_rx,
            stats: ConnectionStats::default(),
        }
    }

    fn record_event(&mut self, event: &ConnectionStatusEvent) {
        match event {
            ConnectionStatusEvent::MixnetSelfPing => {
                trace!("Received self ping event");
                self.stats.latest_self_ping = Some(Instant::now());
            }
            ConnectionStatusEvent::Icmpv4IprTunDevicePingReply => {
                trace!("Received IPR tun device ping reply event");
                self.stats.latest_ipr_tun_device_ping_v4_reply = Some(Instant::now());
            }
            ConnectionStatusEvent::Icmpv6IprTunDevicePingReply => {
                trace!("Received IPR tun device ping v6 reply event");
                self.stats.latest_ipr_tun_device_ping_v6_reply = Some(Instant::now());
            }
            ConnectionStatusEvent::Icmpv4IprExternalPingReply => {
                trace!("Received IPR external ping reply event");
                self.stats.latest_ipr_external_ping_v4_reply = Some(Instant::now());
            }
            ConnectionStatusEvent::Icmpv6IprExternalPingReply => {
                trace!("Received IPR external ping v6 reply event");
                self.stats.latest_ipr_external_ping_v6_reply = Some(Instant::now());
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
                    self.record_event(&event);
                }
                _ = report_interval.tick() => {
                    self.stats.log_status();
                    let connectivity = self.stats.evaluate_connectivity();
                    report_connectivity(&connectivity, &mut task_client);
                }
            }
        }
        debug!("ConnectionMonitor: Exiting");
        Ok(())
    }
}

fn report_connectivity(connectivity: &ConnectivityState, task_client: &mut TaskClient) {
    if connectivity.entry == ConnectivityStatus::Fail {
        error!("Entry gateway not routing our mixnet traffic");
        task_client.send_status_msg(Box::new(ConnectionMonitorStatus::EntryGatewayDown));
        return;
    }

    // If we can route external traffic, then it's ok even if we can't ping the exit IPR.
    if connectivity.exit_routing.ipv4 == ConnectivityStatus::Ok {
        debug!("ConnectionMonitor: connection success over ipv4");
        task_client.send_status_msg(Box::new(ConnectionMonitorStatus::ConnectedIpv4));
    } else if connectivity.exit.ipv4 == ConnectivityStatus::Fail {
        error!("Exit gateway (IPR) not responding to IPv4 traffic");
        task_client.send_status_msg(Box::new(ConnectionMonitorStatus::ExitGatewayDownIpv4));
    } else if connectivity.exit_routing.ipv4 == ConnectivityStatus::Fail {
        error!("Exit gateway (IPR) not routing IPv4 traffic to external destinations");
        task_client.send_status_msg(Box::new(
            ConnectionMonitorStatus::ExitGatewayRoutingErrorIpv4,
        ));
    } else {
        error!("Unexpected connectivity state - exit gateway ipv4 connectivity is ok, but routing is not?");
    }

    if connectivity.exit_routing.ipv6 == ConnectivityStatus::Ok {
        debug!("ConnectionMonitor: connection success over ipv6");
        // task_client.send_status_msg(Box::new(ConnectionMonitorStatus::ConnectedIpv6));
    } else if connectivity.exit.ipv6 == ConnectivityStatus::Fail {
        debug!("Exit gateway (IPR) not responding to IPv6 traffic");
        // task_client.send_status_msg(Box::new(ConnectionMonitorStatus::ExitGatewayDownIpv6));
    } else if connectivity.exit_routing.ipv6 == ConnectivityStatus::Fail {
        debug!("Exit gateway (IPR) not routing IPv6 traffic to external destinations");
        // task_client.send_status_msg(Box::new(
        //     ConnectionMonitorStatus::ExitGatewayRoutingErrorIpv6,
        // ));
    } else {
        debug!("Unexpected connectivity state - exit gateway ipv6 connectivity is ok, but routing is not?");
    }
}

// Events that are emitted by the connection monitor to other components of the system.
// Just like in nym_task::TaskManager and TaskStatus, strictly speaking this is not an error, but a
// status message. We're just piggybacking on the error trait for now. In the future, we might want
// to create a separate trait in nym_task::TaskManager
#[derive(thiserror::Error, Debug)]
pub enum ConnectionMonitorStatus {
    #[error("entry gateway appears down - it's not routing our mixnet traffic")]
    EntryGatewayDown,

    #[error("exit gateway (or ipr) appears down - it's not responding to IPv4 traffic")]
    ExitGatewayDownIpv4,

    #[error("exit gateway (or ipr) appears down - it's not responding to IPv6 traffic")]
    ExitGatewayDownIpv6,

    #[error("exit gateway (or ipr) appears to be having issues routing and forwarding our external IPv4 traffic")]
    ExitGatewayRoutingErrorIpv4,

    #[error("exit gateway (or ipr) appears to be having issues routing and forwarding our external IPv6 traffic")]
    ExitGatewayRoutingErrorIpv6,

    #[error("connected with ipv4")]
    ConnectedIpv4,

    #[error("connected with ipv6")]
    ConnectedIpv6,
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
