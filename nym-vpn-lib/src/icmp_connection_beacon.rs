// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{net::Ipv4Addr, time::Duration};

use bytes::Bytes;
use nym_ip_packet_requests::codec::MultiIpPacketCodec;
use nym_sdk::mixnet::{MixnetClientSender, MixnetMessageSender, Recipient};
use nym_task::TaskClient;
use pnet::packet::{
    icmp::{
        echo_reply::EchoReplyPacket,
        echo_request::{EchoRequestPacket, MutableEchoRequestPacket},
        IcmpPacket,
    },
    ipv4::{Ipv4Packet, MutableIpv4Packet},
    Packet,
};
use tokio::task::JoinHandle;
use tracing::{debug, error, trace};

use crate::{
    error::{Error, Result},
    mixnet_processor,
};

const ICMP_BEACON_PING_INTERVAL: Duration = Duration::from_millis(1000);

struct IcmpConnectionBeacon {
    mixnet_client_sender: MixnetClientSender,
    our_ip: Ipv4Addr,
    ipr_address: Recipient,
    sequence_number: u16,
    icmp_identifier: u16,
}

impl IcmpConnectionBeacon {
    fn new(
        mixnet_client_sender: MixnetClientSender,
        our_ip: Ipv4Addr,
        ipr_address: Recipient,
        icmp_identifier: u16,
    ) -> Self {
        IcmpConnectionBeacon {
            mixnet_client_sender,
            our_ip,
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

    async fn send_icmp_ping(&mut self, destination: Ipv4Addr) -> Result<()> {
        // Create ICMP/IPv4 echo request packet
        let sequence_number = self.get_next_sequence_number();
        let identifier = self.icmp_identifier;
        let icmp_echo_request = create_icmp_echo_request(sequence_number, identifier)?;
        let ipv4_packet = create_icmp_ip_packet(icmp_echo_request, self.our_ip, destination)?;

        // Wrap the IPv4 packet in a MultiIpPacket and send it over the mixnet
        let bundled_packet =
            MultiIpPacketCodec::bundle_one_packet(ipv4_packet.packet().to_vec().into());
        let two_hop = true;
        let message_creator = mixnet_processor::MessageCreator::new(self.ipr_address, two_hop);
        let mixnet_message = message_creator.create_input_message(bundled_packet)?;

        self.mixnet_client_sender
            .send(mixnet_message)
            .await
            .map_err(|err| err.into())
    }

    async fn ping_ipr_tun_device_over_the_mixnet(&mut self) -> Result<()> {
        // TODO: this address is assumed in a few places, extract out to common place
        let ipr_tun_device = Ipv4Addr::new(10, 0, 0, 1);
        self.send_icmp_ping(ipr_tun_device).await
    }

    async fn ping_some_external_ip_over_the_mixnet(&mut self) -> Result<()> {
        // This can be any external IP, we just want to check if the exit IPR can reach the
        // internet
        let some_external_ip = Ipv4Addr::new(8, 8, 8, 8);
        self.send_icmp_ping(some_external_ip).await
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
                    if let Err(err) = self.ping_ipr_tun_device_over_the_mixnet().await {
                        error!("Failed to send ICMP ping: {err}");
                    }
                    if let Err(err) = self.ping_some_external_ip_over_the_mixnet().await {
                        error!("Failed to send ICMP ping: {err}");
                    }
                }
            }
        }
        debug!("IcmpConnectionBeacon: Exiting");
        Ok(())
    }
}

fn create_icmp_echo_request(
    sequence_number: u16,
    identifier: u16,
) -> Result<EchoRequestPacket<'static>> {
    let buffer = vec![0; 64];
    let mut icmp_echo_request = MutableEchoRequestPacket::owned(buffer)
        .ok_or(Error::IcmpEchoRequestPacketCreationFailure)?;

    // Configure the ICMP echo request packet
    icmp_echo_request.set_identifier(identifier);
    icmp_echo_request.set_sequence_number(sequence_number);
    icmp_echo_request.set_icmp_type(pnet::packet::icmp::IcmpTypes::EchoRequest);
    icmp_echo_request.set_icmp_code(pnet::packet::icmp::IcmpCode::new(0));

    // Calculate checksum once we've set all the fields
    let icmp_packet =
        IcmpPacket::new(icmp_echo_request.packet()).ok_or(Error::IcmpPacketCreationFailure)?;
    let checksum = pnet::packet::icmp::checksum(&icmp_packet);
    icmp_echo_request.set_checksum(checksum);

    Ok(icmp_echo_request.consume_to_immutable())
}

fn create_icmp_ip_packet(
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
    ipv4_packet.set_next_level_protocol(pnet::packet::ip::IpNextHeaderProtocols::Icmp);
    ipv4_packet.set_source(source);
    ipv4_packet.set_destination(destination);
    ipv4_packet.set_flags(pnet::packet::ipv4::Ipv4Flags::DontFragment);
    ipv4_packet.set_checksum(0);
    ipv4_packet.set_payload(icmp_echo_request.packet());

    let ipv4_checksum = compute_ipv4_checksum(&ipv4_packet.to_immutable());
    ipv4_packet.set_checksum(ipv4_checksum);

    Ok(ipv4_packet.consume_to_immutable())
}

// Compute IPv4 checksum: sum all 16-bit words, add carry, take one's complement
fn compute_ipv4_checksum(header: &Ipv4Packet) -> u16 {
    let len = header.get_header_length() as usize * 2; // Header length in 16-bit words
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

pub fn start_icmp_connection_beacon(
    mixnet_client_sender: MixnetClientSender,
    our_ip: Ipv4Addr,
    ipr_address: Recipient,
    icmp_identifier: u16,
    shutdown_listener: TaskClient,
) -> JoinHandle<Result<()>> {
    debug!("Creating icmp connection beacon");
    let beacon =
        IcmpConnectionBeacon::new(mixnet_client_sender, our_ip, ipr_address, icmp_identifier);
    tokio::spawn(async move {
        beacon.run(shutdown_listener).await.inspect_err(|err| {
            error!("Icmp connection beacon error: {err}");
        })
    })
}
