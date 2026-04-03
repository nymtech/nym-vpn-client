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

use nix::{
    fcntl::{FcntlArg, OFlag, fcntl},
    sys::socket::{AddressFamily, SockFlag, SockType, socketpair},
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
            let std_sock =
                std::os::unix::net::UnixDatagram::from_raw_fd(sock_filter_fd);
            UnixDatagram::from_std(std_sock)?
        };

        let join_handle =
            tokio::spawn(run_proxy(tun_device, filter_socket, dns_filter, shutdown_token));

        Ok(Self { wg_fd, join_handle })
    }
}

fn set_nonblock(fd: &impl AsFd) -> Result<(), std::io::Error> {
    let flags = OFlag::from_bits_retain(
        fcntl(fd, FcntlArg::F_GETFL).map_err(std::io::Error::from)?,
    );
    if !flags.contains(OFlag::O_NONBLOCK) {
        fcntl(fd, FcntlArg::F_SETFL(flags | OFlag::O_NONBLOCK))
            .map_err(std::io::Error::from)?;
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
                    Ok(0) => continue,
                    Ok(n) => {
                        let packet = &tun_buf[..n];
                        match maybe_nxdomain(packet, &dns_filter).await {
                            Some(response) => {
                                // Blocked — write NXDOMAIN directly back to the tun device.
                                if let Err(e) = tun_device.write_all(&response).await {
                                    tracing::debug!("DNS proxy: write NXDOMAIN to tun failed: {e}");
                                }
                            }
                            None => {
                                // Pass through to wireguard-go.
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
                    Ok(0) => continue,
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

/// If `packet` is a DNS query for a blocked domain, return a pre-built NXDOMAIN response.
/// Returns `None` if the packet should pass through.
async fn maybe_nxdomain(packet: &[u8], dns_filter: &DnsFilter) -> Option<Vec<u8>> {
    if packet.is_empty() {
        return None;
    }
    match packet[0] >> 4 {
        4 => maybe_nxdomain_v4(packet, dns_filter).await,
        6 => maybe_nxdomain_v6(packet, dns_filter).await,
        _ => None,
    }
}

async fn maybe_nxdomain_v4(packet: &[u8], dns_filter: &DnsFilter) -> Option<Vec<u8>> {
    if packet.len() < 20 {
        return None;
    }
    if packet[9] != 17 {
        // Not UDP
        return None;
    }
    let ihl = ((packet[0] & 0x0f) as usize) * 4;
    if packet.len() < ihl + 8 {
        return None;
    }
    let dst_port = u16::from_be_bytes([packet[ihl + 2], packet[ihl + 3]]);
    if dst_port != DNS_PORT {
        return None;
    }
    let dns_start = ihl + 8;
    if packet.len() <= dns_start {
        return None;
    }
    let domain = parse_qname(&packet[dns_start..])?;
    if !is_blocked(&domain, dns_filter).await {
        return None;
    }
    tracing::debug!("Ad-blocker: blocking DNS query for {domain}");
    Some(build_nxdomain_v4(packet, ihl))
}

async fn maybe_nxdomain_v6(packet: &[u8], dns_filter: &DnsFilter) -> Option<Vec<u8>> {
    if packet.len() < 40 {
        return None;
    }
    if packet[6] != 17 {
        // Not UDP (ignores extension headers)
        return None;
    }
    let udp_start = 40usize;
    if packet.len() < udp_start + 8 {
        return None;
    }
    let dst_port = u16::from_be_bytes([packet[udp_start + 2], packet[udp_start + 3]]);
    if dst_port != DNS_PORT {
        return None;
    }
    let dns_start = udp_start + 8;
    if packet.len() <= dns_start {
        return None;
    }
    let domain = parse_qname(&packet[dns_start..])?;
    if !is_blocked(&domain, dns_filter).await {
        return None;
    }
    tracing::debug!("Ad-blocker: blocking DNS query for {domain}");
    Some(build_nxdomain_v6(packet, udp_start))
}

async fn is_blocked(domain: &str, dns_filter: &DnsFilter) -> bool {
    let guard = dns_filter.lock().await;
    matches!(guard.should_block(domain), DnsFilterDecision::Block(_))
}

/// Parse the first QNAME from a DNS wire-format payload.
/// Returns `None` if the payload is not a valid DNS query or parsing fails.
fn parse_qname(dns: &[u8]) -> Option<String> {
    // DNS header is 12 bytes; QR bit must be 0 (query)
    if dns.len() < 12 {
        return None;
    }
    let flags = u16::from_be_bytes([dns[2], dns[3]]);
    if flags & 0x8000 != 0 {
        return None; // Response, not query
    }
    let qdcount = u16::from_be_bytes([dns[4], dns[5]]);
    if qdcount == 0 {
        return None;
    }

    let mut offset = 12usize;
    let mut labels: Vec<&str> = Vec::new();

    loop {
        if offset >= dns.len() {
            return None;
        }
        let len = dns[offset] as usize;
        if len == 0 {
            break;
        }
        // Compression pointer (top 2 bits = 11)
        if len & 0xC0 == 0xC0 {
            break;
        }
        offset += 1;
        if offset + len > dns.len() {
            return None;
        }
        labels.push(std::str::from_utf8(&dns[offset..offset + len]).ok()?);
        offset += len;
    }

    if labels.is_empty() {
        return None;
    }
    Some(labels.join("."))
}

/// Build an NXDOMAIN response for the given IPv4 DNS query packet.
fn build_nxdomain_v4(original: &[u8], ihl: usize) -> Vec<u8> {
    let dns_start = ihl + 8;
    let nxdomain = nxdomain_dns(&original[dns_start..]);

    let new_total = ihl + 8 + nxdomain.len();
    let mut resp = vec![0u8; new_total];

    // IP header: swap src/dst
    resp[..ihl].copy_from_slice(&original[..ihl]);
    resp[12..16].copy_from_slice(&original[16..20]); // dst → src
    resp[16..20].copy_from_slice(&original[12..16]); // src → dst
    let total_len = new_total as u16;
    resp[2..4].copy_from_slice(&total_len.to_be_bytes());
    resp[10] = 0;
    resp[11] = 0;
    let cksum = ipv4_checksum(&resp[..ihl]);
    resp[10..12].copy_from_slice(&cksum.to_be_bytes());

    // UDP header: swap ports
    resp[ihl..ihl + 2].copy_from_slice(&original[ihl + 2..ihl + 4]); // dst → src
    resp[ihl + 2..ihl + 4].copy_from_slice(&original[ihl..ihl + 2]); // src → dst
    let udp_len = (8 + nxdomain.len()) as u16;
    resp[ihl + 4..ihl + 6].copy_from_slice(&udp_len.to_be_bytes());
    resp[ihl + 6] = 0;
    resp[ihl + 7] = 0;

    resp[ihl + 8..].copy_from_slice(&nxdomain);
    resp
}

/// Build an NXDOMAIN response for the given IPv6 DNS query packet.
fn build_nxdomain_v6(original: &[u8], udp_start: usize) -> Vec<u8> {
    let dns_start = udp_start + 8;
    let nxdomain = nxdomain_dns(&original[dns_start..]);

    let udp_payload_len = 8 + nxdomain.len();
    let new_total = 40 + udp_payload_len;
    let mut resp = vec![0u8; new_total];

    // IPv6 header: swap src/dst (bytes 8–23 ↔ 24–39)
    resp[..40].copy_from_slice(&original[..40]);
    resp[8..24].copy_from_slice(&original[24..40]);
    resp[24..40].copy_from_slice(&original[8..24]);
    let payload_len = udp_payload_len as u16;
    resp[4..6].copy_from_slice(&payload_len.to_be_bytes());

    // UDP header: swap ports
    resp[40..42].copy_from_slice(&original[udp_start + 2..udp_start + 4]);
    resp[42..44].copy_from_slice(&original[udp_start..udp_start + 2]);
    resp[44..46].copy_from_slice(&(udp_payload_len as u16).to_be_bytes());
    resp[46] = 0;
    resp[47] = 0;

    resp[48..].copy_from_slice(&nxdomain);
    resp
}

/// Produce a minimal NXDOMAIN DNS response from a query payload (header + question section).
fn nxdomain_dns(query: &[u8]) -> Vec<u8> {
    if query.len() < 12 {
        return query.to_vec();
    }
    let mut resp = query.to_vec();
    // Set QR=1 (response), preserve OPCODE+RD, set RCODE=3 (NXDOMAIN)
    resp[2] = query[2] | 0x80;
    resp[3] = (query[3] & 0xF8) | 0x03; // RA=0, RCODE=NXDOMAIN
    // Zero answer/authority/additional counts
    resp[6] = 0; resp[7] = 0;
    resp[8] = 0; resp[9] = 0;
    resp[10] = 0; resp[11] = 0;
    resp
}

/// Compute the one's-complement checksum of an IPv4 header.
fn ipv4_checksum(header: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut i = 0;
    while i + 1 < header.len() {
        sum += u16::from_be_bytes([header[i], header[i + 1]]) as u32;
        i += 2;
    }
    if i < header.len() {
        sum += (header[i] as u32) << 8;
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}
