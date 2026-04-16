// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! DNS filter proxy for Android.
//!
//! On Android we cannot bind a DNS server on port 53 without root. Instead this proxy sits
//! between the Android tun device and wireguard-go, intercepting DNS queries at the raw IP
//! packet level and injecting NXDOMAIN responses for blocked domains.
//!
//! Architecture:
//! ```
//! [Android tun] <—read/write—> [DnsFilterProxy task] <—AF_UNIX SOCK_DGRAM—> [wireguard-go]
//! ```
//! The wireguard-go exit tunnel receives `sock_wg_fd` as its "tun device". Because
//! `tun.CreateTUNFromFd` on Android only does raw file I/O (no tun-specific ioctls), a
//! `SOCK_DGRAM` socket pair works as a drop-in replacement.

use std::os::fd::{AsFd, FromRawFd, IntoRawFd, OwnedFd};

use hickory_proto::op::{Message, MessageType, ResponseCode};
use nix::{
    fcntl::{FcntlArg, OFlag, fcntl},
    sys::socket::{AddressFamily, SockFlag, SockType, socketpair},
};
use pnet_packet::{
    MutablePacket, Packet,
    ip::IpNextHeaderProtocols,
    ipv4::{self, Ipv4Packet, MutableIpv4Packet},
    ipv6::{Ipv6Packet, MutableIpv6Packet},
    tcp::{self, MutableTcpPacket, TcpFlags, TcpPacket},
    udp::{self, MutableUdpPacket, UdpPacket},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixDatagram,
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use tun::AsyncDevice;

use crate::dns_filter::{DnsFilter, DnsFilterDecision};

/// Maximum IP packet size we handle.
const MAX_PACKET_SIZE: usize = 65536;

/// UDP port used by DNS.
const DNS_PORT: u16 = 53;

/// Port for RST
const DOT_PORT: u16 = 853;

/// IPv4 version nibble.
const IP_VERSION_4: u8 = 4;

/// IPv6 version nibble.
const IP_VERSION_6: u8 = 6;

/// IPv6 header length in bytes.
const IPV6_HEADER_LEN: usize = 40;

/// A proxy that sits between the Android tun device and wireguard-go, intercepting DNS packets.
pub struct DnsFilterProxy {
    /// File descriptor to hand to wireguard-go as its "tun device".
    pub wg_fd: OwnedFd,
    /// Background task handle — the proxy keeps running until `shutdown_token` is cancelled.
    pub join_handle: JoinHandle<()>,
}

impl DnsFilterProxy {
    /// Start the proxy.
    ///
    /// `tun_device` is moved into the proxy task and will be dropped when the task stops.
    /// `wg_fd` must be passed to `wireguard_go::Tunnel::start` immediately after this call.
    pub fn start(
        tun_device: AsyncDevice,
        dns_filter: DnsFilter,
        shutdown_token: CancellationToken,
    ) -> Result<Self, std::io::Error> {
        // Create a Unix datagram socket pair.
        // sock_filter_end ←→ sock_wg_end
        let (sock_filter_raw, sock_wg_raw) = socketpair(
            AddressFamily::Unix,
            SockType::Datagram,
            None,
            SockFlag::empty(),
        )
        .map_err(std::io::Error::from)?;

        // Ensure both ends are O_NONBLOCK so wireguard-go can poll them.
        set_nonblock(&sock_filter_raw)?;
        set_nonblock(&sock_wg_raw)?;

        let sock_filter_fd = sock_filter_raw.into_raw_fd();
        let wg_fd = sock_wg_raw; // OwnedFd passed to wireguard-go

        // Wrap the filter end in tokio's UnixDatagram for async I/O.
        let filter_socket = unsafe {
            let std_sock = std::os::unix::net::UnixDatagram::from_raw_fd(sock_filter_fd);
            UnixDatagram::from_std(std_sock)?
        };

        let join_handle = tokio::spawn(run_proxy(
            tun_device,
            filter_socket,
            dns_filter,
            shutdown_token,
        ));

        Ok(Self { wg_fd, join_handle })
    }
}

fn set_nonblock(fd: &impl AsFd) -> Result<(), std::io::Error> {
    let flags =
        OFlag::from_bits_retain(fcntl(fd, FcntlArg::F_GETFL).map_err(std::io::Error::from)?);
    if !flags.contains(OFlag::O_NONBLOCK) {
        fcntl(fd, FcntlArg::F_SETFL(flags | OFlag::O_NONBLOCK)).map_err(std::io::Error::from)?;
    }
    Ok(())
}

/// The main proxy loop: bridges traffic between the tun device and wireguard-go.
async fn run_proxy(
    mut tun_device: AsyncDevice,
    filter_socket: UnixDatagram,
    dns_filter: DnsFilter,
    shutdown_token: CancellationToken,
) {
    tracing::debug!("DNS filter proxy started");

    let mut tun_buf = vec![0u8; MAX_PACKET_SIZE];
    let mut wg_buf = vec![0u8; MAX_PACKET_SIZE];

    loop {
        tokio::select! {
            // Outbound: tun → (inspect) → wireguard
            result = tun_device.read(&mut tun_buf) => {
                match result {
                    Ok(0) => break,
                    Ok(n) => {
                        let packet = &tun_buf[..n];
                        tracing::trace!("DNS proxy: received {} bytes from tun (first byte: {:#04x})", n, packet.first().copied().unwrap_or(0));
                        match maybe_nxdomain(packet, &dns_filter).await {
                            Some(response) => {
                                if let Err(e) = tun_device.write_all(&response).await {
                                    tracing::debug!("DNS proxy: write response to tun failed: {e}");
                                }
                            }
                            None => {
                                if let Err(e) = filter_socket.send(packet).await {
                                    tracing::debug!("DNS proxy: send to wg socket failed: {e}");
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("DNS proxy: read from tun failed: {e}");
                        break;
                    }
                }
            }

            // Inbound: wireguard → tun
            result = filter_socket.recv(&mut wg_buf) => {
                match result {
                    Ok(0) => break,
                    Ok(n) => {
                        if let Err(e) = tun_device.write_all(&wg_buf[..n]).await {
                            tracing::debug!("DNS proxy: write from wg to tun failed: {e}");
                        }
                    }
                    Err(e) => {
                        tracing::error!("DNS proxy: recv from wg socket failed: {e}");
                        break;
                    }
                }
            }

            _ = shutdown_token.cancelled() => break,
        }
    }

    tracing::debug!("DNS filter proxy stopped");
}

async fn maybe_nxdomain(packet: &[u8], dns_filter: &DnsFilter) -> Option<Vec<u8>> {
    match packet.first().map(|b| b >> 4)? {
        IP_VERSION_4 => maybe_nxdomain_v4(packet, dns_filter).await,
        IP_VERSION_6 => maybe_nxdomain_v6(packet, dns_filter).await,
        version => {
            tracing::debug!("DNS filter proxy: unknown IP version {version}, passing through");
            None
        }
    }
}

async fn maybe_nxdomain_v4(packet: &[u8], dns_filter: &DnsFilter) -> Option<Vec<u8>> {
    let Some(ip) = Ipv4Packet::new(packet) else {
        tracing::debug!("DNS filter proxy: failed to parse IPv4 packet");
        return None;
    };

    match ip.get_next_level_protocol() {
        IpNextHeaderProtocols::Udp => {
            let Some(udp) = UdpPacket::new(ip.payload()) else {
                tracing::debug!("DNS filter proxy: failed to parse UDP packet from IPv4");
                return None;
            };
            tracing::trace!(
                "DNS proxy: IPv4 UDP packet src={}:{} dst={}:{}",
                ip.get_source(),
                udp.get_source(),
                ip.get_destination(),
                udp.get_destination()
            );
            if udp.get_destination() == DNS_PORT {
                let domain = blocked_domain(udp.payload(), dns_filter).await?;
                tracing::debug!("Ad-blocker: blocking DNS query for {domain}");
                let nxdomain_dns = build_nxdomain_dns(udp.payload())?;
                Some(build_udp_response_v4(&ip, &udp, nxdomain_dns))
            } else {
                None
            }
        }
        IpNextHeaderProtocols::Tcp => {
            let Some(tcp) = TcpPacket::new(ip.payload()) else {
                tracing::debug!("DNS filter proxy: failed to parse TCP packet from IPv4");
                return None;
            };
            match tcp.get_destination() {
                DOT_PORT => {
                    // Always RST DoT connections to force Android back to UDP/53.
                    tracing::trace!("DNS proxy: RST TCP/853 (DoT) from IPv4");
                    Some(build_tcp_rst_response_v4(&ip, &tcp))
                }
                DNS_PORT => {
                    // Always RST TCP/53 since we don't support it
                    tracing::trace!("DNS proxy: RST TCP/53 from IPv4");
                    Some(build_tcp_rst_response_v4(&ip, &tcp))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

async fn maybe_nxdomain_v6(packet: &[u8], dns_filter: &DnsFilter) -> Option<Vec<u8>> {
    let Some(ip) = Ipv6Packet::new(packet) else {
        tracing::debug!("DNS filter proxy: failed to parse IPv6 packet");
        return None;
    };

    match ip.get_next_header() {
        IpNextHeaderProtocols::Udp => {
            let udp = match UdpPacket::new(ip.payload()) {
                Some(udp) => udp,
                None => {
                    tracing::debug!("DNS filter proxy: failed to parse UDP packet from IPv6");
                    return None;
                }
            };
            tracing::trace!(
                "DNS proxy: IPv6 UDP packet src=[{}]:{} dst=[{}]:{}",
                ip.get_source(),
                udp.get_source(),
                ip.get_destination(),
                udp.get_destination()
            );

            if udp.get_destination() == DNS_PORT {
                let domain = blocked_domain(udp.payload(), dns_filter).await?;
                tracing::debug!("Ad-blocker: blocking DNS query for {domain}");
                let nxdomain_dns = build_nxdomain_dns(udp.payload())?;
                Some(build_udp_response_v6(&ip, &udp, nxdomain_dns))
            } else {
                None
            }
        }
        IpNextHeaderProtocols::Tcp => {
            let Some(tcp) = TcpPacket::new(ip.payload()) else {
                tracing::debug!("DNS filter proxy: failed to parse TCP packet from IPv6");
                return None;
            };
            match tcp.get_destination() {
                DOT_PORT => {
                    tracing::debug!("DNS proxy: RST TCP/853 (DoT) from IPv6");
                    Some(build_tcp_rst_response_v6(&ip, &tcp))
                }
                DNS_PORT => {
                    tracing::trace!("DNS proxy: RST TCP/53 from IPv6");
                    Some(build_tcp_rst_response_v6(&ip, &tcp))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// Returns the queried domain name if it should be blocked, `None` otherwise.
async fn blocked_domain(dns_payload: &[u8], dns_filter: &DnsFilter) -> Option<String> {
    let msg = match Message::from_vec(dns_payload) {
        Ok(msg) => msg,
        Err(e) => {
            tracing::debug!("DNS filter proxy: failed to parse DNS message: {e}");
            return None;
        }
    };
    if msg.message_type() != MessageType::Query {
        return None;
    }
    let guard = dns_filter.lock().await;
    msg.queries().iter().find_map(|query| {
        let domain = query.name().to_string();
        let domain = domain.trim_end_matches('.');
        tracing::debug!("DNS proxy: checking domain '{domain}'");
        let decision = guard.should_block(domain);
        tracing::debug!("DNS proxy: should_block('{domain}') = {decision:?}");
        matches!(decision, DnsFilterDecision::Block(_)).then_some(domain.to_string())
    })
}

fn build_nxdomain_dns(query: &[u8]) -> Option<Vec<u8>> {
    let mut msg = Message::from_vec(query)
        .inspect_err(|e| {
            tracing::debug!("DNS filter proxy: failed to parse DNS message: {e}");
        })
        .ok()?;
    msg.set_message_type(MessageType::Response);
    msg.set_response_code(ResponseCode::NXDomain);
    msg.take_answers();
    msg.take_name_servers();
    msg.take_additionals();

    msg.to_vec()
        .inspect_err(|e| {
            tracing::debug!("DNS filter proxy: failed to serialize NXDOMAIN response: {e}")
        })
        .ok()
}

fn build_udp_response_v4(orig_ip: &Ipv4Packet, orig_udp: &UdpPacket, dns: Vec<u8>) -> Vec<u8> {
    let ihl = orig_ip.get_header_length() as usize * 4;
    let udp_len = (UdpPacket::minimum_packet_size() + dns.len()) as u16;
    let total_len = ihl + UdpPacket::minimum_packet_size() + dns.len();
    let mut buf = vec![0u8; total_len];

    buf[..ihl].copy_from_slice(&orig_ip.packet()[..ihl]);

    let mut ip = MutableIpv4Packet::new(&mut buf).expect("buf is large enough");
    ip.set_source(orig_ip.get_destination());
    ip.set_destination(orig_ip.get_source());
    ip.set_total_length(total_len as u16);
    ip.set_checksum(0);

    {
        let src = ip.get_source();
        let dst = ip.get_destination();
        let mut udp = MutableUdpPacket::new(ip.payload_mut()).expect("buf is large enough");
        udp.set_source(orig_udp.get_destination());
        udp.set_destination(orig_udp.get_source());
        udp.set_length(udp_len);
        udp.set_payload(&dns);
        udp.set_checksum(udp::ipv4_checksum(&udp.to_immutable(), &dst, &src));
    }

    ip.set_checksum(ipv4::checksum(&ip.to_immutable()));

    buf
}

fn build_udp_response_v6(orig_ip: &Ipv6Packet, orig_udp: &UdpPacket, dns: Vec<u8>) -> Vec<u8> {
    let udp_len = (UdpPacket::minimum_packet_size() + dns.len()) as u16;
    let total_len = IPV6_HEADER_LEN + UdpPacket::minimum_packet_size() + dns.len();
    let mut buf = vec![0u8; total_len];

    buf[..IPV6_HEADER_LEN].copy_from_slice(&orig_ip.packet()[..IPV6_HEADER_LEN]);

    let mut ip = MutableIpv6Packet::new(&mut buf).expect("buf is large enough");
    ip.set_source(orig_ip.get_destination());
    ip.set_destination(orig_ip.get_source());
    ip.set_payload_length(udp_len);

    {
        let src = ip.get_source();
        let dst = ip.get_destination();
        let mut udp = MutableUdpPacket::new(ip.payload_mut()).expect("buf is large enough");
        udp.set_source(orig_udp.get_destination());
        udp.set_destination(orig_udp.get_source());
        udp.set_length(udp_len);
        udp.set_payload(&dns);
        udp.set_checksum(udp::ipv6_checksum(&udp.to_immutable(), &dst, &src));
    }
    buf
}

fn build_tcp_rst_response_v4(orig_ip: &Ipv4Packet, orig_tcp: &TcpPacket) -> Vec<u8> {
    let ihl = orig_ip.get_header_length() as usize * 4;
    let total_len = ihl + TcpPacket::minimum_packet_size();
    let mut buf = vec![0u8; total_len];

    buf[..ihl].copy_from_slice(&orig_ip.packet()[..ihl]);

    let mut ip = MutableIpv4Packet::new(&mut buf).expect("buf is large enough");
    ip.set_source(orig_ip.get_destination());
    ip.set_destination(orig_ip.get_source());
    ip.set_total_length(total_len as u16);
    ip.set_checksum(0);

    {
        let src = ip.get_source();
        let dst = ip.get_destination();
        let mut tcp = MutableTcpPacket::new(ip.payload_mut()).expect("buf is large enough");
        tcp.set_source(orig_tcp.get_destination());
        tcp.set_destination(orig_tcp.get_source());
        tcp.set_window(0);
        tcp.set_data_offset(5);
        tcp.set_flags(TcpFlags::RST);
        tcp.set_sequence(orig_tcp.get_acknowledgement());
        tcp.set_acknowledgement(0);
        tcp.set_checksum(tcp::ipv4_checksum(&tcp.to_immutable(), &dst, &src));
    }

    ip.set_checksum(ipv4::checksum(&ip.to_immutable()));

    buf
}

fn build_tcp_rst_response_v6(orig_ip: &Ipv6Packet, orig_tcp: &TcpPacket) -> Vec<u8> {
    let tcp_len = TcpPacket::minimum_packet_size();
    let total_len = IPV6_HEADER_LEN + tcp_len;
    let mut buf = vec![0u8; total_len];

    buf[..IPV6_HEADER_LEN].copy_from_slice(&orig_ip.packet()[..IPV6_HEADER_LEN]);

    let mut ip = MutableIpv6Packet::new(&mut buf).expect("buf is large enough");
    ip.set_source(orig_ip.get_destination());
    ip.set_destination(orig_ip.get_source());
    ip.set_payload_length(tcp_len as u16);

    {
        let src = ip.get_source();
        let dst = ip.get_destination();
        let mut tcp = MutableTcpPacket::new(ip.payload_mut()).expect("buf is large enough");
        tcp.set_source(orig_tcp.get_destination());
        tcp.set_destination(orig_tcp.get_source());
        tcp.set_window(0);
        tcp.set_data_offset(5);
        tcp.set_flags(TcpFlags::RST);
        tcp.set_sequence(orig_tcp.get_acknowledgement());
        tcp.set_acknowledgement(0);
        tcp.set_checksum(tcp::ipv6_checksum(&tcp.to_immutable(), &dst, &src));
    }

    buf
}
