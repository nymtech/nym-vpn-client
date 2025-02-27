use std::net::{Ipv4Addr, Ipv6Addr};

use bytes::Bytes;
use nym_connection_monitor::{
    is_icmp_beacon_reply, is_icmp_v6_beacon_reply,
    packet_helpers::{
        create_icmpv4_echo_request, create_icmpv6_echo_request, wrap_icmp_in_ipv4,
        wrap_icmp_in_ipv6,
    },
    ConnectionStatusEvent, IcmpBeaconReply, Icmpv6BeaconReply,
};
use nym_gateway_directory::IpPacketRouterAddress;
use nym_ip_packet_requests::{codec::MultiIpPacketCodec, v7::request::IpPacketRequest, IpPair};
use nym_mixnet_client::SharedMixnetClient;
use nym_sdk::mixnet::{InputMessage, Recipient};
use nym_task::connections::TransmissionLane;
use pnet_packet::Packet;

use crate::Result;

pub fn icmp_identifier() -> u16 {
    8475
}

pub async fn send_ping_v4(
    shared_mixnet_client: SharedMixnetClient,
    our_ips: IpPair,
    sequence_number: u16,
    destination: Ipv4Addr,
    exit_router_address: IpPacketRouterAddress,
) -> anyhow::Result<()> {
    let icmp_identifier = icmp_identifier();
    let icmp_echo_request = create_icmpv4_echo_request(sequence_number, icmp_identifier)?;
    let ipv4_packet = wrap_icmp_in_ipv4(icmp_echo_request, our_ips.ipv4, destination)?;

    // Wrap the IPv4 packet in a MultiIpPacket
    let bundled_packet =
        MultiIpPacketCodec::bundle_one_packet(ipv4_packet.packet().to_vec().into());

    // Wrap into a mixnet input message addressed to the IPR
    let mixnet_message = create_input_message(exit_router_address.0, bundled_packet)?;

    shared_mixnet_client.send(mixnet_message).await?;
    Ok(())
}

pub async fn send_ping_v6(
    shared_mixnet_client: SharedMixnetClient,
    our_ips: IpPair,
    sequence_number: u16,
    destination: Ipv6Addr,
    exit_router_address: IpPacketRouterAddress,
) -> anyhow::Result<()> {
    let icmp_identifier = icmp_identifier();
    let icmp_echo_request = create_icmpv6_echo_request(
        sequence_number,
        icmp_identifier,
        &our_ips.ipv6,
        &destination,
    )?;
    let ipv6_packet = wrap_icmp_in_ipv6(icmp_echo_request, our_ips.ipv6, destination)?;

    // Wrap the IPv6 packet in a MultiIpPacket
    let bundled_packet =
        MultiIpPacketCodec::bundle_one_packet(ipv6_packet.packet().to_vec().into());

    // Wrap into a mixnet input message addressed to the IPR
    let mixnet_message = create_input_message(exit_router_address.0, bundled_packet)?;

    // Send across the mixnet
    shared_mixnet_client.send(mixnet_message).await?;
    Ok(())
}

fn create_input_message(recipient: Recipient, bundled_packets: Bytes) -> Result<InputMessage> {
    let packet = IpPacketRequest::new_data_request(bundled_packets).to_bytes()?;

    let lane = TransmissionLane::General;
    let packet_type = None;
    Ok(InputMessage::new_regular(
        recipient,
        packet,
        lane,
        packet_type,
    ))
}

pub fn check_for_icmp_beacon_reply(
    packet: &Bytes,
    icmp_beacon_identifier: u16,
    our_ips: IpPair,
) -> Option<ConnectionStatusEvent> {
    match is_icmp_beacon_reply(packet, icmp_beacon_identifier, our_ips.ipv4) {
        Some(IcmpBeaconReply::TunDeviceReply) => {
            tracing::debug!("Received ping response from ipr tun device");
            return Some(ConnectionStatusEvent::Icmpv4IprTunDevicePingReply);
        }
        Some(IcmpBeaconReply::ExternalPingReply(_source)) => {
            tracing::debug!("Received ping response from an external ip through the ipr");
            return Some(ConnectionStatusEvent::Icmpv4IprExternalPingReply);
        }
        None => {}
    }

    match is_icmp_v6_beacon_reply(packet, icmp_beacon_identifier, our_ips.ipv6) {
        Some(Icmpv6BeaconReply::TunDeviceReply) => {
            tracing::debug!("Received ping v6 response from ipr tun device");
            return Some(ConnectionStatusEvent::Icmpv6IprTunDevicePingReply);
        }
        Some(Icmpv6BeaconReply::ExternalPingReply(_source)) => {
            tracing::debug!("Received ping v6 response from an external ip through the ipr");
            return Some(ConnectionStatusEvent::Icmpv6IprExternalPingReply);
        }
        None => {}
    }

    None
}
