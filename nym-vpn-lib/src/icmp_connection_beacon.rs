// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{net::Ipv4Addr, time::Duration};

use nym_sdk::mixnet::{MixnetClientSender, MixnetMessageSender, Recipient};
use nym_task::TaskClient;
use pnet::packet::{
    icmp::{echo_request::MutableEchoRequestPacket, IcmpPacket},
    Packet,
};
use tokio::task::JoinHandle;
use tracing::{debug, error, trace};

use crate::error::Result;

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

    async fn send_icmp_ping(&mut self) -> Result<()> {
        let mut buffer = vec![0; 64];
        let sequence_number = self.get_next_sequence_number();
        let identifier = self.icmp_identifier;
        let mut icmp_echo_request =
            create_icmp_echo_request(&mut buffer, sequence_number, identifier);

        let icmp = IcmpPacket::new(icmp_echo_request.packet()).unwrap();
        let checksum = pnet::packet::icmp::checksum(&icmp);
        icmp_echo_request.set_checksum(checksum);
        // dbg!(&icmp_packet);

        let destination = Ipv4Addr::new(10, 0, 0, 1);
        // let destination = Ipv4Addr::new(8, 8, 8, 8);
        let source = self.our_ip;

        let total_length = 20 + icmp_echo_request.packet().len() as u16; // 20 bytes for IPv4 header + ICMP payload
        let mut ipv4_buffer = vec![0u8; 20 + icmp_echo_request.packet().len()]; // IPv4 header + ICMP payload
        let mut ipv4_packet = pnet::packet::ipv4::MutableIpv4Packet::new(&mut ipv4_buffer).unwrap();

        ipv4_packet.set_version(4);
        ipv4_packet.set_header_length(5);
        ipv4_packet.set_total_length(total_length);
        ipv4_packet.set_ttl(64);
        ipv4_packet.set_next_level_protocol(pnet::packet::ip::IpNextHeaderProtocols::Icmp);
        ipv4_packet.set_source(source);
        ipv4_packet.set_destination(destination);
        ipv4_packet.set_flags(pnet::packet::ipv4::Ipv4Flags::DontFragment);
        ipv4_packet.set_checksum(0);
        ipv4_packet.set_payload(icmp_echo_request.packet());

        let ipv4_checksum = compute_ipv4_checksum(&ipv4_packet.to_immutable());
        ipv4_packet.set_checksum(ipv4_checksum);

        // ipv4_packet now contains the entire packet
        // dbg!(&ipv4_packet);
        let final_packet = ipv4_packet.packet().to_vec();
        // println!("final_packet: {:?}", final_packet);

        let mut multi_ip_packet_encoder = nym_ip_packet_requests::codec::MultiIpPacketCodec::new(
            nym_ip_packet_requests::codec::BUFFER_TIMEOUT,
        );

        multi_ip_packet_encoder.append_packet(final_packet.into());
        let bundled_packet = multi_ip_packet_encoder.flush_current_buffer();

        let message_creator = crate::mixnet_processor::MessageCreator::new(self.ipr_address, false);
        let mixnet_message = message_creator
            .create_input_message(bundled_packet)
            .expect("Failed to create input message");

        self.mixnet_client_sender
            .send(mixnet_message)
            .await
            .unwrap();

        Ok(())
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
                    if let Err(err) = self.send_icmp_ping().await {
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
    buffer: &mut [u8],
    sequence_number: u16,
    identifier: u16,
) -> MutableEchoRequestPacket {
    let mut icmp_packet = MutableEchoRequestPacket::new(buffer).unwrap();
    icmp_packet.set_identifier(identifier);
    icmp_packet.set_sequence_number(sequence_number);
    icmp_packet.set_icmp_type(pnet::packet::icmp::IcmpTypes::EchoRequest);
    icmp_packet.set_icmp_code(pnet::packet::icmp::IcmpCode::new(0));
    icmp_packet
}

// Compute IPv4 checksum: sum all 16-bit words, add carry, take one's complement
fn compute_ipv4_checksum(header: &pnet::packet::ipv4::Ipv4Packet) -> u16 {
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
