// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{net::Ipv4Addr, time::Duration};

use futures::StreamExt;
use nym_ip_packet_requests::request::IpPacketRequest;
use nym_sdk::mixnet::{InputMessage, MixnetClientSender, MixnetMessageSender, Recipient};
use nym_task::{connections::TransmissionLane, TaskClient};
use pnet::packet::Packet;
use tokio::task::JoinHandle;
use tracing::{debug, error, trace};

use crate::{
    error::{Error, Result},
    mixnet_connect::SharedMixnetClient,
};

const MIXNET_SELF_PING_INTERVAL: Duration = Duration::from_millis(2000);

struct MixnetConnectionBeacon {
    mixnet_client_sender: MixnetClientSender,
    our_address: Recipient,
    our_ip: Ipv4Addr,
    ipr_address: Recipient,
    sequence_number: u16,
    icmp_identifier: u16,
}

fn create_self_ping(our_address: Recipient) -> (InputMessage, u64) {
    let (request, request_id) = IpPacketRequest::new_ping(our_address);
    (
        InputMessage::new_regular(
            our_address,
            request.to_bytes().unwrap(),
            TransmissionLane::General,
            None,
        ),
        request_id,
    )
}

// Send mixnet self ping and wait for the response
pub(crate) async fn self_ping_and_wait(
    our_address: Recipient,
    mixnet_client: SharedMixnetClient,
) -> Result<()> {
    // We want to send a bunch of pings and wait for the first one to return
    let request_ids: Vec<_> = futures::stream::iter(1..=3)
        .then(|_| async {
            let (input_message, request_id) = create_self_ping(our_address);
            mixnet_client.send(input_message).await?;
            Ok::<u64, Error>(request_id)
        })
        .collect::<Vec<_>>()
        .await;
    // Check the vec of results and return the first error, if any. If there are not errors, unwrap
    // all the results into a vec of u64s.
    let request_ids = request_ids.into_iter().collect::<Result<Vec<_>>>()?;
    wait_for_self_ping_return(&mixnet_client, &request_ids).await
}

async fn wait_for_self_ping_return(
    mixnet_client: &SharedMixnetClient,
    request_ids: &[u64],
) -> Result<()> {
    let timeout = tokio::time::sleep(Duration::from_secs(5));
    tokio::pin!(timeout);

    // Connecting is basically synchronous from the perspective of the mixnet client, so it's safe
    // to just grab ahold of the mutex and keep it until we get the response.
    let mut mixnet_client_handle = mixnet_client.lock().await;
    let mixnet_client = mixnet_client_handle.as_mut().unwrap();

    loop {
        tokio::select! {
            _ = &mut timeout => {
                error!("Timed out waiting for mixnet self ping to return");
                return Err(Error::TimeoutWaitingForConnectResponse);
            }
            Some(msgs) = mixnet_client.wait_for_messages() => {
                for msg in msgs {
                    let Ok(response) = IpPacketRequest::from_reconstructed_message(&msg) else {
                        // TODO: consider just not logging here since we expect this to be
                        // common when reconnecting to a gateway
                        error!("Failed to deserialize reconstructed message");
                        continue;
                    };
                    if request_ids.iter().any(|&id| response.id() == Some(id)) {
                        debug!("Got the ping we were waiting for");
                        return Ok(());
                    }
                }
            }
        }
    }
}

impl MixnetConnectionBeacon {
    fn new(
        mixnet_client_sender: MixnetClientSender,
        our_address: Recipient,
        our_ip: Ipv4Addr,
        ipr_address: Recipient,
        icmp_identifier: u16,
    ) -> Self {
        MixnetConnectionBeacon {
            mixnet_client_sender,
            our_address,
            our_ip,
            ipr_address,
            sequence_number: 0,
            // icmp_identifier: Self::random_u16(),
            icmp_identifier,
        }
    }

    fn random_u16() -> u16 {
        use rand::Rng;
        rand::thread_rng().gen()
    }

    async fn send_mixnet_self_ping(&self) -> Result<u64> {
        trace!("Sending mixnet self ping");
        let (input_message, request_id) = create_self_ping(self.our_address);
        self.mixnet_client_sender.send(input_message).await?;
        Ok(request_id)
    }

    fn get_next_sequence_number(&mut self) -> u16 {
        // TODO: wraparound?
        let sequence_number = self.sequence_number;
        self.sequence_number += 1;
        sequence_number
    }

    async fn send_icmp_ping(&mut self) -> Result<()> {
        // Create a ICMP IP packet as a Vec<u8>.
        // The destination is 10.0.0.1 and I need to tag it
        // properly so that I can identify it when it comes back.
        // Use pnet
        let mut buffer = vec![0; 64];
        let mut icmp_packet =
            pnet::packet::icmp::echo_request::MutableEchoRequestPacket::new(&mut buffer).unwrap();
        icmp_packet.set_identifier(self.icmp_identifier);
        icmp_packet.set_sequence_number(self.get_next_sequence_number());
        icmp_packet.set_icmp_type(pnet::packet::icmp::IcmpTypes::EchoRequest);
        icmp_packet.set_icmp_code(pnet::packet::icmp::IcmpCode::new(0));

        // let icmp_packet_immutable = icmp_packet.to_immutable();
        let icmp = pnet::packet::icmp::IcmpPacket::new(pnet::packet::Packet::packet(&icmp_packet))
            .unwrap();
        let checksum = pnet::packet::icmp::checksum(&icmp);
        icmp_packet.set_checksum(checksum);
        // dbg!(&icmp_packet);

        let destination = Ipv4Addr::new(10, 0, 0, 1);
        // let destination = Ipv4Addr::new(8, 8, 8, 8);
        let source = self.our_ip;

        let total_length = 20 + icmp_packet.packet().len() as u16; // 20 bytes for IPv4 header + ICMP payload
        let mut ipv4_buffer = vec![0u8; 20 + icmp_packet.packet().len()]; // IPv4 header + ICMP payload
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
        ipv4_packet.set_payload(icmp_packet.packet());

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
        debug!("Mixnet connection beacon is running");
        let mut ping_interval = tokio::time::interval(MIXNET_SELF_PING_INTERVAL);
        loop {
            tokio::select! {
                _ = shutdown.recv() => {
                    trace!("MixnetConnectionBeacon: Received shutdown");
                    break;
                }
                _ = ping_interval.tick() => {
                    match self.send_icmp_ping().await {
                        Ok(_) => (),
                        Err(err) => {
                            error!("Failed to send ICMP ping: {err}");
                            continue;
                        }
                    }
                }
                // _ = ping_interval.tick() => {
                //     let _ping_id = match self.send_mixnet_self_ping().await {
                //         Ok(id) => id,
                //         Err(err) => {
                //             error!("Failed to send mixnet self ping: {err}");
                //             continue;
                //         }
                //     };
                //     // TODO: store ping_id to be able to monitor or ping timeouts
                // }
            }
        }
        debug!("MixnetConnectionBeacon: Exiting");
        Ok(())
    }
}

pub fn start_mixnet_connection_beacon(
    mixnet_client_sender: MixnetClientSender,
    our_address: Recipient,
    our_ip: Ipv4Addr,
    ipr_address: Recipient,
    icmp_identifier: u16,
    shutdown_listener: TaskClient,
) -> JoinHandle<Result<()>> {
    debug!("Creating mixnet connection beacon");
    let beacon = MixnetConnectionBeacon::new(
        mixnet_client_sender,
        our_address,
        our_ip,
        ipr_address,
        icmp_identifier,
    );
    tokio::spawn(async move {
        beacon.run(shutdown_listener).await.inspect_err(|err| {
            error!("Mixnet connection beacon error: {err}");
        })
    })
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
