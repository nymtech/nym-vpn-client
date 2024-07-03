use bytes::Bytes;
use nym_connection_monitor::{
    is_icmp_beacon_reply, is_icmp_v6_beacon_reply,
    packet_helpers::{
        create_icmpv4_echo_request, create_icmpv6_echo_request, wrap_icmp_in_ipv4,
        wrap_icmp_in_ipv6,
    },
    ConnectionStatusEvent, IcmpBeaconReply, Icmpv6BeaconReply,
};
use nym_gateway_directory::MixAddresses;
use nym_ip_packet_client::SharedMixnetClient;
use nym_ip_packet_requests::{codec::MultiIpPacketCodec, IpPair};
use nym_sdk::mixnet::{InputMessage, Recipient};
use nym_task::connections::TransmissionLane;
use pnet_packet::Packet;
use std::net::{Ipv4Addr, Ipv6Addr};

use crate::Result;

pub fn icmp_identifier() -> u16 {
    8475
}

pub async fn send_ping_v4(
    shared_mixnet_client: SharedMixnetClient,
    our_ips: IpPair,
    sequence_number: u16,
    destination: Ipv4Addr,
    exit_router_address: MixAddresses,
) -> anyhow::Result<()> {
    let icmp_identifier = icmp_identifier();
    let icmp_echo_request = create_icmpv4_echo_request(sequence_number, icmp_identifier)?;
    let ipv4_packet = wrap_icmp_in_ipv4(icmp_echo_request, our_ips.ipv4, destination)?;

    // Wrap the IPv4 packet in a MultiIpPacket
    let bundled_packet =
        MultiIpPacketCodec::bundle_one_packet(ipv4_packet.packet().to_vec().into());

    // Wrap into a mixnet input message addressed to the IPR
    let two_hop = true;
    let mixnet_message = create_input_message(
        exit_router_address.ip_packet_router_address,
        bundled_packet,
        two_hop,
    )?;

    shared_mixnet_client.send(mixnet_message).await?;
    Ok(())
}

pub async fn send_ping_v6(
    shared_mixnet_client: SharedMixnetClient,
    our_ips: IpPair,
    sequence_number: u16,
    destination: Ipv6Addr,
    exit_router_address: MixAddresses,
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
    let two_hop = true;
    let mixnet_message = create_input_message(
        exit_router_address.ip_packet_router_address,
        bundled_packet,
        two_hop,
    )?;

    // Send across the mixnet
    shared_mixnet_client.send(mixnet_message).await?;
    Ok(())
}

fn create_input_message(
    recipient: Recipient,
    bundled_packets: Bytes,
    enable_two_hop: bool,
) -> Result<InputMessage> {
    let packet =
        nym_ip_packet_requests::request::IpPacketRequest::new_data_request(bundled_packets)
            .to_bytes()?;

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

pub fn check_for_icmp_beacon_reply(
    packet: &Bytes,
    icmp_beacon_identifier: u16,
    our_ips: IpPair,
) -> Option<ConnectionStatusEvent> {
    match is_icmp_beacon_reply(packet, icmp_beacon_identifier, our_ips.ipv4) {
        Some(IcmpBeaconReply::TunDeviceReply) => {
            log::debug!("Received ping response from ipr tun device");
            return Some(ConnectionStatusEvent::Icmpv4IprTunDevicePingReply);
        }
        Some(IcmpBeaconReply::ExternalPingReply(_source)) => {
            log::debug!("Received ping response from an external ip through the ipr");
            return Some(ConnectionStatusEvent::Icmpv4IprExternalPingReply);
        }
        None => {}
    }

    match is_icmp_v6_beacon_reply(packet, icmp_beacon_identifier, our_ips.ipv6) {
        Some(Icmpv6BeaconReply::TunDeviceReply) => {
            log::debug!("Received ping v6 response from ipr tun device");
            return Some(ConnectionStatusEvent::Icmpv6IprTunDevicePingReply);
        }
        Some(Icmpv6BeaconReply::ExternalPingReply(_source)) => {
            log::debug!("Received ping v6 response from an external ip through the ipr");
            return Some(ConnectionStatusEvent::Icmpv6IprExternalPingReply);
        }
        None => {}
    }

    None
}
