use std::time::Duration;

use futures::{stream::StreamExt, FutureExt};
use nym_statistics_common::clients::packet_statistics::MixnetBandwidthStatisticsEvent;
use tokio::{sync::mpsc, task::JoinHandle};

use nym_bandwidth_controller::BandwidthStatusMessage;
use nym_connection_monitor::ConnectionMonitorStatus;
use nym_task::{StatusReceiver, TaskStatus};
use nym_vpn_lib_types::{BandwidthEvent, ConnectionEvent, ConnectionStatisticsEvent, MixnetEvent};
use tokio_util::sync::CancellationToken;

pub struct StatusListener {
    rx: StatusReceiver,
    tx: mpsc::UnboundedSender<MixnetEvent>,
    cancel_token: CancellationToken,
}

impl StatusListener {
    pub fn spawn(
        rx: StatusReceiver,
        tx: mpsc::UnboundedSender<MixnetEvent>,
        cancel_token: CancellationToken,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            let status_listener = Self {
                rx,
                tx,
                cancel_token,
            };
            status_listener.run().await;
        })
    }

    async fn run(mut self) {
        tracing::debug!("Starting status listener loop");

        // The status listener will exit when the status receiver is dropped, but to be on the safe
        // side we also listen for the cancellation token to be cancelled.
        let cancel_fut = self.cancel_token.cancelled().then(|_| async {
            tokio::time::sleep(Duration::from_secs(5)).await;
        });
        tokio::pin!(cancel_fut);

        loop {
            tokio::select! {
                msg = self.rx.next() => {
                    let Some(msg) = msg else {
                        break
                    };

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

                _ = &mut cancel_fut => {
                    break;
                }
            }
        }

        tracing::debug!("Exiting status listener loop");
    }

    fn send_event(&self, event: MixnetEvent) {
        if let Err(e) = self.tx.send(event) {
            if !self.cancel_token.is_cancelled() {
                tracing::error!("Failed to send mixnet event: {}", e);
            }
        }
    }
}
