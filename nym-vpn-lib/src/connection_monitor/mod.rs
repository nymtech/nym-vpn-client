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

pub(crate) fn start_connection_monitor(
    mixnet_client_sender: MixnetClientSender,
    our_nym_address: Recipient,
    our_ips: IpPair,
    exit_router_address: &IpPacketRouterAddress,
    icmp_beacon_identifier: u16,
    connection_event_rx: mpsc::UnboundedReceiver<monitor::ConnectionStatusEvent>,
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
        icmp_beacon_identifier,
        task_manager.subscribe_named("icmp_beacon"),
    );

    info!("Setting up connection monitor");
    monitor::start_connection_monitor(
        connection_event_rx,
        task_manager.subscribe_named("connection_monitor"),
    );
}
