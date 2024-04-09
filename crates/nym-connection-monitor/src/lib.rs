// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use futures::channel::mpsc;
use nym_ip_packet_requests::IpPair;
use nym_sdk::mixnet::{MixnetClientSender, Recipient};
use nym_task::TaskManager;
use tracing::info;

mod error;
mod icmp_beacon;
mod mixnet_beacon;
mod monitor;
mod packet_helpers;
mod sync_self_ping;

pub use error::Error;
pub use icmp_beacon::{
    is_icmp_beacon_reply, is_icmp_v6_beacon_reply, IcmpBeaconReply, Icmpv6BeaconReply,
};
pub use monitor::ConnectionStatusEvent;
pub use sync_self_ping::self_ping_and_wait;

fn create_icmp_beacon_identifier() -> u16 {
    // TODO: use something that is more unique than just process id
    std::process::id() as u16
}

// Top-level task that sets up the ICMP and mixnet connection beacons and the connection monitor,
// as well as provides the channel to send connection status events read from the mixnet, to the
// monitor.
pub struct ConnectionMonitorTask {
    icmp_beacon_identifier: u16,
    connection_event_tx: mpsc::UnboundedSender<monitor::ConnectionStatusEvent>,
    connection_event_rx: mpsc::UnboundedReceiver<monitor::ConnectionStatusEvent>,
}

impl ConnectionMonitorTask {
    pub fn setup() -> ConnectionMonitorTask {
        let (connection_event_tx, connection_event_rx) = mpsc::unbounded();
        let icmp_beacon_identifier = create_icmp_beacon_identifier();
        ConnectionMonitorTask {
            icmp_beacon_identifier,
            connection_event_tx,
            connection_event_rx,
        }
    }

    pub fn event_sender(&self) -> mpsc::UnboundedSender<monitor::ConnectionStatusEvent> {
        self.connection_event_tx.clone()
    }

    pub fn icmp_beacon_identifier(&self) -> u16 {
        self.icmp_beacon_identifier
    }

    pub fn start(
        self,
        mixnet_client_sender: MixnetClientSender,
        our_nym_address: Recipient,
        our_ips: IpPair,
        exit_router_address: Recipient,
        task_manager: &TaskManager,
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
            exit_router_address,
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
