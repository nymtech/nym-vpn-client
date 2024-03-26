// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use futures::channel::mpsc;
use nym_ip_packet_requests::IpPair;
use nym_sdk::mixnet::{MixnetClientSender, Recipient};
use tracing::info;

use crate::mixnet_processor::IpPacketRouterAddress;

pub(crate) mod icmp_beacon;
pub(crate) mod mixnet_beacon;
pub(crate) mod monitor;

pub(crate) fn create_icmp_beacon_identifier() -> u16 {
    std::process::id() as u16
}

pub(crate) struct ConnectionMonitorTask {
    icmp_beacon_identifier: u16,
    connection_event_tx: mpsc::UnboundedSender<monitor::ConnectionStatusEvent>,
    connection_event_rx: mpsc::UnboundedReceiver<monitor::ConnectionStatusEvent>,
}

impl ConnectionMonitorTask {
    pub(crate) fn setup() -> ConnectionMonitorTask {
        let (connection_event_tx, connection_event_rx) = mpsc::unbounded();
        let icmp_beacon_identifier = create_icmp_beacon_identifier();
        ConnectionMonitorTask {
            icmp_beacon_identifier,
            connection_event_tx,
            connection_event_rx,
        }
    }

    pub(crate) fn event_sender(&self) -> mpsc::UnboundedSender<monitor::ConnectionStatusEvent> {
        self.connection_event_tx.clone()
    }

    pub(crate) fn icmp_beacon_identifier(&self) -> u16 {
        self.icmp_beacon_identifier
    }

    pub(crate) fn start(
        self,
        mixnet_client_sender: MixnetClientSender,
        our_nym_address: Recipient,
        our_ips: IpPair,
        exit_router_address: &IpPacketRouterAddress,
        task_manager: &nym_task::TaskManager,
    ) {
        info!("Setting up mixnet connection beacon");
        mixnet_beacon::start_mixnet_connection_beacon(
            mixnet_client_sender.clone(),
            our_nym_address,
            task_manager.subscribe_named("mixnet_beacon"),
        );

        info!("Setting up ICMP connection beacon");
        icmp_beacon::start_icmp_connection_beacon(
            mixnet_client_sender,
            our_ips,
            exit_router_address.0,
            self.icmp_beacon_identifier,
            task_manager.subscribe_named("icmp_beacon"),
        );

        info!("Setting up connection monitor");
        monitor::start_connection_monitor(
            self.connection_event_rx,
            task_manager.subscribe_named("connection_monitor"),
        );
    }
}
