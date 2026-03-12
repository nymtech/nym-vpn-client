// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::{SocketAddrV4, SocketAddrV6};

use pnet_packet::{
    ip::IpNextHeaderProtocols,
    ipv4::{Ipv4Flags, Ipv4Packet, MutableIpv4Packet},
    ipv6::{Ipv6Packet, MutableIpv6Packet},
    tcp::{MutableTcpPacket, TcpFlags, TcpPacket},
};

const IPV4_HEADER_VERSION: u8 = 4;
const IPV6_HEADER_VERSION: u8 = 6;

const IPV4_HEADER_LEN: usize = Ipv4Packet::minimum_packet_size();
/// IPv4 header length measured in 32-bit words
const IPV4_HEADER_WORD_LEN: u8 = (IPV4_HEADER_LEN / 4) as u8;
const IPV6_HEADER_LEN: usize = Ipv6Packet::minimum_packet_size();

const TCP_HEADER_LEN: usize = TcpPacket::minimum_packet_size();
/// TCP header length measured in 32-bit words
const TCP_HEADER_WORD_LEN: u8 = (TCP_HEADER_LEN / 4) as u8;

pub const IPV4_PACKET_LEN: usize = IPV4_HEADER_LEN + TCP_HEADER_LEN;
pub const IPV6_PACKET_LEN: usize = IPV6_HEADER_LEN + TCP_HEADER_LEN;

/// Fill buffer with IPv4 header and TCP/RST payload
pub fn fill_tcp_rst_packet_v4(
    buf: &mut [u8; IPV4_PACKET_LEN],
    src: SocketAddrV4,
    dst: SocketAddrV4,
    seq: u32,
) {
    let mut tcp_header = MutableTcpPacket::new(&mut buf[IPV4_HEADER_LEN..]).unwrap();
    tcp_header.set_source(src.port());
    tcp_header.set_destination(dst.port());
    tcp_header.set_flags(TcpFlags::RST);
    tcp_header.set_window(0);
    tcp_header.set_data_offset(TCP_HEADER_WORD_LEN);
    tcp_header.set_sequence(seq); // no increment, RST does not consume sequence
    tcp_header.set_acknowledgement(0);
    let chksum = pnet_packet::tcp::ipv4_checksum(&tcp_header.to_immutable(), src.ip(), dst.ip());
    tcp_header.set_checksum(chksum);

    let mut ip_header = MutableIpv4Packet::new(&mut buf[..]).unwrap();
    ip_header.set_version(IPV4_HEADER_VERSION);
    ip_header.set_header_length(IPV4_HEADER_WORD_LEN);
    ip_header.set_total_length(IPV4_PACKET_LEN as u16);
    ip_header.set_ttl(64);
    ip_header.set_next_level_protocol(IpNextHeaderProtocols::Tcp);
    ip_header.set_destination(*dst.ip());
    ip_header.set_source(*src.ip());
    ip_header.set_flags(Ipv4Flags::DontFragment);
    ip_header.set_fragment_offset(0);
    ip_header.set_identification(0); // can be zero since packet is not fragmented?
    let chksum = pnet_packet::ipv4::checksum(&ip_header.to_immutable());
    ip_header.set_checksum(chksum);
}

/// Fill buffer with IPv6 header and TCP/RST payload
pub fn fill_tcp_rst_packet_v6(
    buf: &mut [u8; IPV6_PACKET_LEN],
    src: SocketAddrV6,
    dst: SocketAddrV6,
    seq: u32,
) {
    let mut ip_header = MutableIpv6Packet::new(&mut buf[..]).unwrap();
    ip_header.set_version(IPV6_HEADER_VERSION);
    ip_header.set_payload_length(TCP_HEADER_LEN as u16);
    ip_header.set_hop_limit(64);
    ip_header.set_next_header(IpNextHeaderProtocols::Tcp);
    ip_header.set_destination(*dst.ip());
    ip_header.set_source(*src.ip());

    let mut tcp_header = MutableTcpPacket::new(&mut buf[IPV6_HEADER_LEN..]).unwrap();
    tcp_header.set_source(src.port());
    tcp_header.set_destination(dst.port());
    tcp_header.set_flags(TcpFlags::RST);
    tcp_header.set_window(0);
    tcp_header.set_data_offset(TCP_HEADER_WORD_LEN);
    tcp_header.set_sequence(seq); // no increment, RST does not consume sequence
    tcp_header.set_acknowledgement(0);
    let chksum = pnet_packet::tcp::ipv6_checksum(&tcp_header.to_immutable(), src.ip(), dst.ip());
    tcp_header.set_checksum(chksum);
}
