use futures::stream::StreamExt;
use tokio::{sync::mpsc, task::JoinHandle};

use nym_bandwidth_controller::BandwidthStatusMessage;
use nym_connection_monitor::ConnectionMonitorStatus;
use nym_task::{StatusReceiver, TaskStatus};

use crate::tunnel_state_machine::{BandwidthEvent, ConnectionEvent, MixnetEvent};

pub struct StatusListener {
    rx: StatusReceiver,
    tx: mpsc::UnboundedSender<MixnetEvent>,
}

impl StatusListener {
    pub fn spawn(rx: StatusReceiver, tx: mpsc::UnboundedSender<MixnetEvent>) -> JoinHandle<()> {
        tokio::spawn(async move {
            let status_listener = Self { rx, tx };
            status_listener.run().await;
        })
    }

    async fn run(mut self) {
        tracing::debug!("Starting status listener loop");

        while let Some(msg) = self.rx.next().await {
            if let Some(msg) = msg.as_any().downcast_ref::<TaskStatus>() {
                tracing::info!("Received ignored TaskStatus message: {msg}");
            } else if let Some(msg) = msg.as_any().downcast_ref::<ConnectionMonitorStatus>() {
                tracing::info!("VPN connection monitor status: {msg}");
                self.send_event(MixnetEvent::Connection(ConnectionEvent::from(msg)));
            } else if let Some(msg) = msg.as_any().downcast_ref::<BandwidthStatusMessage>() {
                tracing::info!("VPN bandwidth status: {msg}");
                self.send_event(MixnetEvent::Bandwidth(BandwidthEvent::from(msg)));
            } else {
                tracing::warn!("VPN status: unknown: {msg}");
                tracing::debug!("Unknown status message received: {msg}");
            }
        }

        tracing::debug!("Exiting status listener loop");
    }

    fn send_event(&self, event: MixnetEvent) {
        if let Err(e) = self.tx.send(event) {
            tracing::error!("Failed to send event: {}", e);
        }
    }
}

impl From<&BandwidthStatusMessage> for BandwidthEvent {
    fn from(value: &BandwidthStatusMessage) -> Self {
        match value {
            BandwidthStatusMessage::NoBandwidth => Self::NoBandwidth,
            BandwidthStatusMessage::RemainingBandwidth(value) => Self::RemainingBandwidth(*value),
        }
    }
}

impl From<&ConnectionMonitorStatus> for ConnectionEvent {
    fn from(value: &ConnectionMonitorStatus) -> Self {
        match value {
            ConnectionMonitorStatus::ConnectedIpv4 => Self::ConnectedIpv4,
            ConnectionMonitorStatus::ConnectedIpv6 => Self::ConnectedIpv6,
            ConnectionMonitorStatus::EntryGatewayDown => Self::EntryGatewayDown,
            ConnectionMonitorStatus::ExitGatewayDownIpv4 => Self::ExitGatewayDownIpv4,
            ConnectionMonitorStatus::ExitGatewayDownIpv6 => Self::ExitGatewayDownIpv6,
            ConnectionMonitorStatus::ExitGatewayRoutingErrorIpv4 => {
                Self::ExitGatewayRoutingErrorIpv4
            }
            ConnectionMonitorStatus::ExitGatewayRoutingErrorIpv6 => {
                Self::ExitGatewayRoutingErrorIpv6
            }
        }
    }
}
