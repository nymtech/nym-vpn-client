use futures::stream::StreamExt;
use nym_statistics_common::clients::packet_statistics::{
    MixnetBandwidthStatisticsEvent, PacketRates,
};
use tokio::{sync::mpsc, task::JoinHandle};

use nym_bandwidth_controller::BandwidthStatusMessage;
use nym_connection_monitor::ConnectionMonitorStatus;
use nym_task::{StatusReceiver, TaskStatus};

use crate::tunnel_state_machine::{
    BandwidthEvent, ConnectionEvent, ConnectionStatisticsEvent, MixnetEvent, SphinxPacketRates,
};

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
                tracing::debug!("Received ignored TaskStatus message: {msg}");
            } else if let Some(msg) = msg.as_any().downcast_ref::<ConnectionMonitorStatus>() {
                self.send_event(MixnetEvent::Connection(ConnectionEvent::from(msg)));
            } else if let Some(msg) = msg.as_any().downcast_ref::<BandwidthStatusMessage>() {
                self.send_event(MixnetEvent::Bandwidth(BandwidthEvent::from(msg)));
            } else if let Some(msg) = msg
                .as_any()
                .downcast_ref::<MixnetBandwidthStatisticsEvent>()
            {
                self.send_event(MixnetEvent::ConnectionStatistics(
                    ConnectionStatisticsEvent::from(msg),
                ));
            } else {
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

impl From<&MixnetBandwidthStatisticsEvent> for ConnectionStatisticsEvent {
    fn from(value: &MixnetBandwidthStatisticsEvent) -> Self {
        Self {
            rates: SphinxPacketRates::from(value.rates.clone()),
        }
    }
}

impl From<PacketRates> for SphinxPacketRates {
    fn from(value: PacketRates) -> Self {
        Self {
            real_packets_sent: value.real_packets_sent,
            real_packets_sent_size: value.real_packets_sent_size,
            cover_packets_sent: value.cover_packets_sent,
            cover_packets_sent_size: value.cover_packets_sent_size,
            real_packets_received: value.real_packets_received,
            real_packets_received_size: value.real_packets_received_size,
            cover_packets_received: value.cover_packets_received,
            cover_packets_received_size: value.cover_packets_received_size,
            total_acks_received: value.total_acks_received,
            total_acks_received_size: value.total_acks_received_size,
            real_acks_received: value.real_acks_received,
            real_acks_received_size: value.real_acks_received_size,
            cover_acks_received: value.cover_acks_received,
            cover_acks_received_size: value.cover_acks_received_size,
            real_packets_queued: value.real_packets_queued,
            retransmissions_queued: value.retransmissions_queued,
            reply_surbs_queued: value.reply_surbs_queued,
            additional_reply_surbs_queued: value.additional_reply_surbs_queued,
        }
    }
}
