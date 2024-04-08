// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    net::{Ipv4Addr, Ipv6Addr},
    time::Duration,
};

use bytes::Bytes;
use nym_ip_packet_requests::{codec::MultiIpPacketCodec, request::IpPacketRequest, IpPair};
use nym_sdk::{
    mixnet::{InputMessage, MixnetClientSender, MixnetMessageSender, Recipient},
    TaskClient,
};
use nym_task::connections::TransmissionLane;
use pnet_packet::Packet;
use tokio::task::JoinHandle;
use tracing::{debug, error, trace};

use crate::{
    error::Result,
    packet_helpers::{
        create_icmpv4_echo_request, create_icmpv6_echo_request, is_icmp_echo_reply,
        is_icmp_v6_echo_reply, wrap_icmp_in_ipv4, wrap_icmp_in_ipv6,
    },
};

const ICMP_BEACON_PING_INTERVAL: Duration = Duration::from_millis(1000);

// TODO: extract these from the ip-packet-router crate
const ICMP_IPR_TUN_IP_V4: Ipv4Addr = Ipv4Addr::new(10, 0, 0, 1);
// 2001:db8:a160::1
const ICMP_IPR_TUN_IP_V6: Ipv6Addr = Ipv6Addr::new(0x2001, 0xdb8, 0xa160, 0, 0, 0, 0, 0x1);

// This can be anything really, we just want to check if the exit IPR can reach the internet
// TODO: have a pool of IPs to ping
const ICMP_IPR_TUN_EXTERNAL_PING_V4: Ipv4Addr = Ipv4Addr::new(8, 8, 8, 8);
const ICMP_IPR_TUN_EXTERNAL_PING_V6: Ipv6Addr =
    Ipv6Addr::new(0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8888);

struct IcmpConnectionBeacon {
    mixnet_client_sender: MixnetClientSender,
    our_ips: IpPair,
    ipr_address: Recipient,
    sequence_number: u16,
    icmp_identifier: u16,
}

impl IcmpConnectionBeacon {
    fn new(
        mixnet_client_sender: MixnetClientSender,
        our_ips: IpPair,
        ipr_address: Recipient,
        icmp_identifier: u16,
    ) -> Self {
        IcmpConnectionBeacon {
            mixnet_client_sender,
            our_ips,
            ipr_address,
            sequence_number: 0,
            icmp_identifier,
        }
    }

    fn get_next_sequence_number(&mut self) -> u16 {
        let sequence_number = self.sequence_number;
        self.sequence_number = self.sequence_number.wrapping_add(1);
        sequence_number
    }

    async fn send_icmp_v4_ping(&mut self, destination: Ipv4Addr) -> Result<()> {
        // Create ICMP/IPv4 echo request packet
        let sequence_number = self.get_next_sequence_number();
        let identifier = self.icmp_identifier;
        let icmp_echo_request = create_icmpv4_echo_request(sequence_number, identifier)?;
        let ipv4_packet = wrap_icmp_in_ipv4(icmp_echo_request, self.our_ips.ipv4, destination)?;

        // Wrap the IPv4 packet in a MultiIpPacket
        let bundled_packet =
            MultiIpPacketCodec::bundle_one_packet(ipv4_packet.packet().to_vec().into());

        // Wrap into a mixnet input message addressed to the IPR
        let two_hop = true;
        let mixnet_message = create_input_message(self.ipr_address, bundled_packet, two_hop)?;

        // Send across the mixnet
        self.mixnet_client_sender
            .send(mixnet_message)
            .await
            .map_err(|err| err.into())
    }

    async fn send_icmp_v6_ping(&mut self, destination: Ipv6Addr) -> Result<()> {
        // Create ICMP/IPv6 echo request packet
        let sequence_number = self.get_next_sequence_number();
        let identifier = self.icmp_identifier;
        let icmp_echo_request = create_icmpv6_echo_request(
            sequence_number,
            identifier,
            &self.our_ips.ipv6,
            &destination,
        )?;
        let ipv6_packet = wrap_icmp_in_ipv6(icmp_echo_request, self.our_ips.ipv6, destination)?;

        // Wrap the IPv6 packet in a MultiIpPacket
        let bundled_packet =
            MultiIpPacketCodec::bundle_one_packet(ipv6_packet.packet().to_vec().into());

        // Wrap into a mixnet input message addressed to the IPR
        let two_hop = true;
        let mixnet_message = create_input_message(self.ipr_address, bundled_packet, two_hop)?;

        // Send across the mixnet
        self.mixnet_client_sender
            .send(mixnet_message)
            .await
            .map_err(|err| err.into())
    }

    async fn ping_v4_ipr_tun_device_over_the_mixnet(&mut self) -> Result<()> {
        self.send_icmp_v4_ping(ICMP_IPR_TUN_IP_V4).await
    }

    async fn ping_v6_ipr_tun_device_over_the_mixnet(&mut self) -> Result<()> {
        self.send_icmp_v6_ping(ICMP_IPR_TUN_IP_V6).await
    }

    async fn ping_v4_some_external_ip_over_the_mixnet(&mut self) -> Result<()> {
        // TODO: ramdon external IP from a pool
        self.send_icmp_v4_ping(ICMP_IPR_TUN_EXTERNAL_PING_V4).await
    }

    async fn ping_v6_some_external_ip_over_the_mixnet(&mut self) -> Result<()> {
        // TODO: ramdon external IP from a pool
        self.send_icmp_v6_ping(ICMP_IPR_TUN_EXTERNAL_PING_V6).await
    }

    pub async fn run(mut self, mut shutdown: TaskClient) -> Result<()> {
        debug!("Icmp connection beacon is running");
        let mut ping_interval = tokio::time::interval(ICMP_BEACON_PING_INTERVAL);
        loop {
            tokio::select! {
                _ = shutdown.recv() => {
                    trace!("IcmpConnectionBeacon: Received shutdown");
                    break;
                }
                _ = ping_interval.tick() => {
                    if let Err(err) = self.ping_v4_ipr_tun_device_over_the_mixnet().await {
                        error!("Failed to send ICMP ping: {err}");
                    }
                    if let Err(err) = self.ping_v6_ipr_tun_device_over_the_mixnet().await {
                        error!("Failed to send ICMPv6 ping: {err}");
                    }
                    if let Err(err) = self.ping_v4_some_external_ip_over_the_mixnet().await {
                        error!("Failed to send ICMP ping: {err}");
                    }
                    if let Err(err) = self.ping_v6_some_external_ip_over_the_mixnet().await {
                        error!("Failed to send ICMPv6 ping: {err}");
                    }
                }
            }
        }
        debug!("IcmpConnectionBeacon: Exiting");
        Ok(())
    }
}

fn create_input_message(
    recipient: Recipient,
    bundled_packets: Bytes,
    enable_two_hop: bool,
) -> Result<InputMessage> {
    let packet = IpPacketRequest::new_data_request(bundled_packets).to_bytes()?;

    let lane = TransmissionLane::General;
    let packet_type = None;
    let hops = enable_two_hop.then_some(0);
    Ok(InputMessage::new_regular_with_custom_hops(
        recipient,
        packet,
        lane,
        packet_type,
        hops,
    ))
}

pub enum IcmpBeaconReply {
    TunDeviceReply,
    ExternalPingReply(Ipv4Addr),
}

pub enum Icmpv6BeaconReply {
    TunDeviceReply,
    ExternalPingReply(Ipv6Addr),
}

pub fn is_icmp_beacon_reply(
    packet: &Bytes,
    identifier: u16,
    destination: Ipv4Addr,
) -> Option<IcmpBeaconReply> {
    if let Some((reply_identifier, reply_source, reply_destination)) = is_icmp_echo_reply(packet) {
        if reply_identifier == identifier && reply_destination == destination {
            if reply_source == ICMP_IPR_TUN_IP_V4 {
                return Some(IcmpBeaconReply::TunDeviceReply);
            } else if reply_source == ICMP_IPR_TUN_EXTERNAL_PING_V4 {
                return Some(IcmpBeaconReply::ExternalPingReply(reply_source));
            }
        }
    }
    None
}

pub fn is_icmp_v6_beacon_reply(
    packet: &Bytes,
    identifier: u16,
    destination: Ipv6Addr,
) -> Option<Icmpv6BeaconReply> {
    if let Some((reply_identifier, reply_source, reply_destination)) = is_icmp_v6_echo_reply(packet)
    {
        if reply_identifier == identifier && reply_destination == destination {
            if reply_source == ICMP_IPR_TUN_IP_V6 {
                return Some(Icmpv6BeaconReply::TunDeviceReply);
            } else if reply_source == ICMP_IPR_TUN_EXTERNAL_PING_V6 {
                return Some(Icmpv6BeaconReply::ExternalPingReply(reply_source));
            }
        }
    }
    None
}

pub fn start_icmp_connection_beacon(
    mixnet_client_sender: MixnetClientSender,
    our_ips: IpPair,
    ipr_address: Recipient,
    icmp_identifier: u16,
    shutdown_listener: TaskClient,
) -> JoinHandle<Result<()>> {
    debug!("Creating icmp connection beacon");
    let beacon =
        IcmpConnectionBeacon::new(mixnet_client_sender, our_ips, ipr_address, icmp_identifier);
    tokio::spawn(async move {
        beacon.run(shutdown_listener).await.inspect_err(|err| {
            error!("Icmp connection beacon error: {err}");
        })
    })
}
