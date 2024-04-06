// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    net::{Ipv4Addr, Ipv6Addr},
    time::Duration,
};

use bytes::Bytes;
use nym_ip_packet_requests::{codec::MultiIpPacketCodec, request::IpPacketRequest, IpPair};
use nym_sdk::{
    mixnet::{InputMessage, MixnetClientSender, MixnetMessageSender, Recipient, TransmissionLane},
    TaskClient,
};
use pnet_packet::{
    icmp::{
        echo_reply::EchoReplyPacket,
        echo_request::{EchoRequestPacket, MutableEchoRequestPacket},
        IcmpPacket,
    },
    icmpv6,
    ipv4::{Ipv4Packet, MutableIpv4Packet},
    ipv6::{Ipv6Packet, MutableIpv6Packet},
    Packet,
};
use tokio::task::JoinHandle;
use tracing::{debug, error, trace};

use crate::error::{Error, Result};

const ICMP_BEACON_PING_INTERVAL: Duration = Duration::from_millis(1000);

// TODO: extract these from the ip-packet-router crate
pub(crate) const ICMP_IPR_TUN_IP_V4: Ipv4Addr = Ipv4Addr::new(10, 0, 0, 1);
// 2001:db8:a160::1
pub(crate) const ICMP_IPR_TUN_IP_V6: Ipv6Addr =
    Ipv6Addr::new(0x2001, 0xdb8, 0xa160, 0, 0, 0, 0, 0x1);

// This can be anything really, we just want to check if the exit IPR can reach the internet
// TODO: have a pool of IPs to ping
pub(crate) const ICMP_IPR_TUN_EXTERNAL_PING_V4: Ipv4Addr = Ipv4Addr::new(8, 8, 8, 8);
pub(crate) const ICMP_IPR_TUN_EXTERNAL_PING_V6: Ipv6Addr =
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
        self.send_icmp_v4_ping(ICMP_IPR_TUN_EXTERNAL_PING_V4).await
    }

    async fn ping_v6_some_external_ip_over_the_mixnet(&mut self) -> Result<()> {
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

fn create_icmpv4_echo_request(
    sequence_number: u16,
    identifier: u16,
) -> Result<EchoRequestPacket<'static>> {
    let buffer = vec![0; 64];
    let mut icmp_echo_request = MutableEchoRequestPacket::owned(buffer)
        .ok_or(Error::IcmpEchoRequestPacketCreationFailure)?;

    // Configure the ICMP echo request packet
    icmp_echo_request.set_identifier(identifier);
    icmp_echo_request.set_sequence_number(sequence_number);
    icmp_echo_request.set_icmp_type(pnet_packet::icmp::IcmpTypes::EchoRequest);
    icmp_echo_request.set_icmp_code(pnet_packet::icmp::IcmpCode::new(0));

    // Calculate checksum once we've set all the fields
    let icmp_packet =
        IcmpPacket::new(icmp_echo_request.packet()).ok_or(Error::IcmpPacketCreationFailure)?;
    let checksum = pnet_packet::icmp::checksum(&icmp_packet);
    icmp_echo_request.set_checksum(checksum);

    Ok(icmp_echo_request.consume_to_immutable())
}

fn create_icmpv6_echo_request(
    sequence_number: u16,
    identifier: u16,
    source: &Ipv6Addr,
    destination: &Ipv6Addr,
) -> Result<icmpv6::echo_request::EchoRequestPacket<'static>> {
    let buffer = vec![0; 64];
    // let mut icmp_echo_request = MutableEchoRequestPacket::owned(buffer)
    let mut icmp_echo_request = icmpv6::echo_request::MutableEchoRequestPacket::owned(buffer)
        .ok_or(Error::IcmpEchoRequestPacketCreationFailure)?;

    // Configure the ICMP echo request packet
    icmp_echo_request.set_identifier(identifier);
    icmp_echo_request.set_sequence_number(sequence_number);
    icmp_echo_request.set_icmpv6_type(pnet_packet::icmpv6::Icmpv6Types::EchoRequest);
    icmp_echo_request.set_icmpv6_code(pnet_packet::icmpv6::Icmpv6Code::new(0));

    // Calculate checksum once we've set all the fields
    let icmp_packet = icmpv6::Icmpv6Packet::new(icmp_echo_request.packet())
        .ok_or(Error::IcmpPacketCreationFailure)?;
    let checksum = pnet_packet::icmpv6::checksum(&icmp_packet, source, destination);
    icmp_echo_request.set_checksum(checksum);

    Ok(icmp_echo_request.consume_to_immutable())
}

fn wrap_icmp_in_ipv4(
    icmp_echo_request: EchoRequestPacket,
    source: Ipv4Addr,
    destination: Ipv4Addr,
) -> Result<Ipv4Packet> {
    // 20 bytes for IPv4 header + ICMP payload
    let total_length = 20 + icmp_echo_request.packet().len();
    // IPv4 header + ICMP payload
    let ipv4_buffer = vec![0u8; 20 + icmp_echo_request.packet().len()];
    let mut ipv4_packet =
        MutableIpv4Packet::owned(ipv4_buffer).ok_or(Error::Ipv4PacketCreationFailure)?;

    ipv4_packet.set_version(4);
    ipv4_packet.set_header_length(5);
    ipv4_packet.set_total_length(total_length as u16);
    ipv4_packet.set_ttl(64);
    ipv4_packet.set_next_level_protocol(pnet_packet::ip::IpNextHeaderProtocols::Icmp);
    ipv4_packet.set_source(source);
    ipv4_packet.set_destination(destination);
    ipv4_packet.set_flags(pnet_packet::ipv4::Ipv4Flags::DontFragment);
    ipv4_packet.set_checksum(0);
    ipv4_packet.set_payload(icmp_echo_request.packet());

    let ipv4_checksum = compute_ipv4_checksum(&ipv4_packet.to_immutable());
    ipv4_packet.set_checksum(ipv4_checksum);

    Ok(ipv4_packet.consume_to_immutable())
}

fn wrap_icmp_in_ipv6(
    icmp_echo_request: icmpv6::echo_request::EchoRequestPacket,
    source: Ipv6Addr,
    destination: Ipv6Addr,
) -> Result<Ipv6Packet> {
    let ipv6_buffer = vec![0u8; 40 + icmp_echo_request.packet().len()];
    let mut ipv6_packet =
        MutableIpv6Packet::owned(ipv6_buffer).ok_or(Error::Ipv4PacketCreationFailure)?;

    ipv6_packet.set_version(6);
    ipv6_packet.set_payload_length(icmp_echo_request.packet().len() as u16);
    ipv6_packet.set_next_header(pnet_packet::ip::IpNextHeaderProtocols::Icmpv6);
    ipv6_packet.set_hop_limit(64);
    ipv6_packet.set_source(source);
    ipv6_packet.set_destination(destination);
    ipv6_packet.set_payload(icmp_echo_request.packet());

    Ok(ipv6_packet.consume_to_immutable())
}

// Compute IPv4 checksum: sum all 16-bit words, add carry, take one's complement
fn compute_ipv4_checksum(header: &Ipv4Packet) -> u16 {
    // Header length in 16-bit words
    let len = header.get_header_length() as usize * 2;
    let mut sum = 0u32;

    for i in 0..len {
        let word = (header.packet()[2 * i] as u32) << 8 | header.packet()[2 * i + 1] as u32;
        sum += word;
    }

    // Add the carry
    while (sum >> 16) > 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }

    // One's complement
    !sum as u16
}

pub(crate) fn is_icmp_echo_reply(packet: &Bytes) -> Option<(u16, Ipv4Addr, Ipv4Addr)> {
    if let Some(ipv4_packet) = Ipv4Packet::new(packet) {
        if let Some(icmp_packet) = IcmpPacket::new(ipv4_packet.payload()) {
            if let Some(echo_reply) = EchoReplyPacket::new(icmp_packet.packet()) {
                return Some((
                    echo_reply.get_identifier(),
                    ipv4_packet.get_source(),
                    ipv4_packet.get_destination(),
                ));
            }
        }
    }
    None
}

pub(crate) fn is_icmp_v6_echo_reply(packet: &Bytes) -> Option<(u16, Ipv6Addr, Ipv6Addr)> {
    if let Some(ipv6_packet) = Ipv6Packet::new(packet) {
        if let Some(icmp_packet) = IcmpPacket::new(ipv6_packet.payload()) {
            if let Some(echo_reply) =
                pnet_packet::icmpv6::echo_reply::EchoReplyPacket::new(icmp_packet.packet())
            {
                return Some((
                    echo_reply.get_identifier(),
                    ipv6_packet.get_source(),
                    ipv6_packet.get_destination(),
                ));
            }
        }
    }
    None
}

pub fn create_input_message(
    recipient: Recipient,
    bundled_packets: Bytes,
    enable_two_hop: bool,
) -> Result<InputMessage> {
    let packet = IpPacketRequest::new_data_request(bundled_packets).to_bytes()?;

    let lane = TransmissionLane::General;
    let packet_type = None;
    let hops = enable_two_hop.then_some(0);
    let input_message =
        InputMessage::new_regular_with_custom_hops(recipient, packet, lane, packet_type, hops);
    Ok(input_message)
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
