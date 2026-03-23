// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_registration_common::WireguardConfiguration;
use nym_sdk::mixnet::x25519;
use nym_vpn_lib_types::{DiagnosticResult, PingReport};

use boringtun::noise::{Tunn, TunnResult};
use pnet_packet::{Packet, ip::IpNextHeaderProtocols};
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    result::Result::Ok,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::net::UdpSocket;

const PING_PKT_ID: u16 = 0x4242;
const PING_DST_V4: IpAddr = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)); // Cloudflare DNS
const PING_DST_V6: IpAddr = IpAddr::V6(Ipv6Addr::new(0x2606, 0x4700, 0x4700, 0, 0, 0, 0, 0x1111)); // Cloudflare ipv6 DNS

const MAX_PING_ATTEMPTS: u16 = 10;
const MAX_PING_TEST_TIME: Duration = Duration::from_secs(10);

pub struct WireguardDiagnostic {
    wg_tunnel: Tunn,
    udp_socket: UdpSocket,
    private_ip: IpAddr,
    use_ipv6: bool,
}

impl WireguardDiagnostic {
    pub async fn run_diagnostic(
        wireguard_config: WireguardConfiguration,
        gateway_keypair: Arc<x25519::KeyPair>,
    ) -> anyhow::Result<Vec<PingReport>> {
        // We only have one endpoint so we have to adapt, we can't do both
        let use_ipv6 = wireguard_config.endpoint.is_ipv6();

        let (private_ip, bind_address) = if use_ipv6 {
            (wireguard_config.private_ipv6.into(), "[::]:0")
        } else {
            (wireguard_config.private_ipv4.into(), "0.0.0.0:0")
        };
        // UDP socket setup
        tracing::info!("Running wireguard test");
        let udp_socket = UdpSocket::bind(bind_address).await?;
        udp_socket.connect(wireguard_config.endpoint).await?;

        // Wireguard tunnel set up
        let wg_tunnel = Tunn::new(
            gateway_keypair.private_key().inner().clone(), // Yes this is cloning a private key. It is ephemeral and dropped at the end of the run anyway
            wireguard_config.public_key.inner(),
            wireguard_config.psk.map(Into::into),
            None,
            0,
            None,
        );

        let mut wg_diagnostic = WireguardDiagnostic {
            wg_tunnel,
            udp_socket,
            private_ip,
            use_ipv6,
        };
        let report = wg_diagnostic.ping_diagnostic().await;

        Ok(report)
    }

    async fn ping_diagnostic(&mut self) -> Vec<PingReport> {
        let mut reports = Vec::with_capacity(MAX_PING_ATTEMPTS.into());
        let timeout = tokio::time::sleep(MAX_PING_TEST_TIME);
        tokio::pin!(timeout);
        let ping_dst = if self.use_ipv6 {
            PING_DST_V6
        } else {
            PING_DST_V4
        };

        for seq in 0..10 {
            tokio::select! {
                _ = &mut timeout => {
                    tracing::debug!("Wireguard ping test reached timeout");
                    return reports;
                },
                report = self.send_ping_and_check_response(ping_dst, seq) => {
                    reports.push(PingReport {
                        dst: ping_dst,
                        delay_ms : DiagnosticResult::from(report)
                    });
                    // small sleep to avoid spamming
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
        reports
    }

    async fn send_ping_and_check_response(
        &mut self,
        dst: IpAddr,
        seq: u16,
    ) -> anyhow::Result<u128> {
        let ping_pkt = match (self.private_ip, dst) {
            (IpAddr::V4(src), IpAddr::V4(dst)) => build_icmp_ipv4_packet(src, dst, seq),
            (IpAddr::V6(src), IpAddr::V6(dst)) => build_icmp_ipv6_packet(src, dst, seq),
            _ => {
                return Err(anyhow::anyhow!(
                    "Somehow ended up mixing IPv4 and IPv6, this should not have happened"
                ));
            }
        };

        // Arbitrary big enough size
        let mut net_buf = [0u8; 256];
        let mut buf = [0u8; 256];

        let ping_start = Instant::now();

        match self.wg_tunnel.encapsulate(&ping_pkt, &mut net_buf) {
            TunnResult::WriteToNetwork(datagram) => self.udp_socket.send(datagram).await?,
            TunnResult::Err(error) => {
                return Err(anyhow::anyhow!(
                    "Wg tunnel sending reported an error : {error:?}"
                ));
            }
            _ => return Err(anyhow::anyhow!("Unexpected encapsulate return value")),
        };

        let response = 'recv: loop {
            match self.udp_socket.recv(&mut net_buf).await {
                Ok(datagram_len) => {
                    let mut rcv_res =
                        self.wg_tunnel
                            .decapsulate(None, &net_buf[..datagram_len], &mut buf);
                    'decapsulate: loop {
                        match rcv_res {
                            TunnResult::Err(error) => {
                                return Err(anyhow::anyhow!(
                                    "Wg tunnel receiveing reported an error : {error:?}"
                                ));
                            }
                            TunnResult::WriteToNetwork(pkt) => {
                                self.udp_socket.send(pkt).await?;
                                // after WriteToNetwork, call decapsulate with empty slice
                                rcv_res = self.wg_tunnel.decapsulate(None, &[], &mut buf);
                            }
                            TunnResult::WriteToTunnelV4(pkt, _)
                            | TunnResult::WriteToTunnelV6(pkt, _) => {
                                break 'recv pkt.to_vec();
                            }
                            TunnResult::Done => break 'decapsulate,
                        }
                    }
                }
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "UDP socket receiveing reported an error : {e:?}"
                    ));
                }
            }
        };
        let ping_time = ping_start.elapsed();

        // check the reply
        if self.private_ip.is_ipv4() {
            check_ipv4_reply(&response, seq)?;
        } else {
            check_ipv6_reply(&response, seq)?;
        }

        Ok(ping_time.as_millis())
    }
}

pub fn build_icmp_ipv4_packet(src: Ipv4Addr, dst: Ipv4Addr, seq: u16) -> Vec<u8> {
    use pnet_packet::{
        icmp::{IcmpCode, IcmpTypes, echo_request::MutableEchoRequestPacket},
        ipv4::MutableIpv4Packet,
        util::checksum,
    };
    // Total buffer: IP header (20 bytes) + ICMP packet (64 bytes)
    let buf_len = 20 + 64;
    let mut buf = vec![0u8; buf_len];

    // SAFETY : we just created the buffer with enough space
    #[allow(clippy::expect_used)]
    let mut icmp_packet =
        MutableEchoRequestPacket::new(&mut buf[20..]).expect("hardcoded buffer size is too small");
    icmp_packet.set_icmp_type(IcmpTypes::EchoRequest);
    icmp_packet.set_icmp_code(IcmpCode::new(0));
    icmp_packet.set_identifier(PING_PKT_ID);
    icmp_packet.set_sequence_number(seq);
    icmp_packet.set_checksum(checksum(icmp_packet.packet(), 1));

    // SAFETY : we just created the buffer with enough space
    #[allow(clippy::expect_used)]
    let mut ip_packet =
        MutableIpv4Packet::new(&mut buf[..20]).expect("hardcoded buffer size is too small");
    ip_packet.set_version(4);
    ip_packet.set_header_length(5); // 5 * 4 = 20 bytes
    ip_packet.set_total_length(buf_len as u16);
    ip_packet.set_ttl(64);
    ip_packet.set_next_level_protocol(IpNextHeaderProtocols::Icmp);
    ip_packet.set_source(src);
    ip_packet.set_destination(dst);

    // IPv4 checksum
    let cksum = checksum(ip_packet.packet(), 5);
    ip_packet.set_checksum(cksum);

    buf
}

pub fn build_icmp_ipv6_packet(src: Ipv6Addr, dst: Ipv6Addr, seq: u16) -> Vec<u8> {
    use pnet_packet::{
        icmpv6::{Icmpv6Code, Icmpv6Types, echo_request::MutableEchoRequestPacket},
        ipv6::MutableIpv6Packet,
        util::ipv6_checksum,
    };
    // Total buffer: IPv6 header (40 bytes) + ICMPv6 packet (64 bytes)
    let buf_len = 40 + 64;
    let mut buf = vec![0u8; buf_len];

    // SAFETY : we just created the buffer with enough space
    #[allow(clippy::expect_used)]
    let mut icmp_packet =
        MutableEchoRequestPacket::new(&mut buf[40..]).expect("hardcoded buffer size is too small");
    icmp_packet.set_icmpv6_type(Icmpv6Types::EchoRequest);
    icmp_packet.set_icmpv6_code(Icmpv6Code::new(0));
    icmp_packet.set_identifier(PING_PKT_ID);
    icmp_packet.set_sequence_number(seq);

    // ICMPv6 checksum (includes IPv6 pseudo-header)
    let cksum = ipv6_checksum(
        icmp_packet.packet(),
        1,
        &[],
        &src,
        &dst,
        IpNextHeaderProtocols::Icmpv6,
    );
    icmp_packet.set_checksum(cksum);

    // SAFETY : we just created the buffer with enough space
    #[allow(clippy::expect_used)]
    let mut ip_packet =
        MutableIpv6Packet::new(&mut buf[..40]).expect("hardcoded buffer size is too small");
    ip_packet.set_version(6);
    ip_packet.set_payload_length(64);
    ip_packet.set_hop_limit(64);
    ip_packet.set_next_header(IpNextHeaderProtocols::Icmpv6);
    ip_packet.set_source(src);
    ip_packet.set_destination(dst);

    buf
}

pub fn check_ipv4_reply(packet: &[u8], seq: u16) -> anyhow::Result<()> {
    use pnet_packet::{icmp::echo_reply::EchoReplyPacket, ipv4::Ipv4Packet};

    if let Some(ip) = Ipv4Packet::new(packet)
        && ip.get_next_level_protocol() == IpNextHeaderProtocols::Icmp
        && let Some(reply) = EchoReplyPacket::new(ip.payload())
    {
        if reply.get_identifier() == PING_PKT_ID && reply.get_sequence_number() == seq {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Received a ping reply with mismatched id : Expected {PING_PKT_ID}, {seq}; Actual {}, {}",
                reply.get_identifier(),
                reply.get_sequence_number()
            ))
        }
    } else {
        Err(anyhow::anyhow!("Received a packet that isn't a ping reply"))
    }
}

pub fn check_ipv6_reply(packet: &[u8], seq: u16) -> anyhow::Result<()> {
    use pnet_packet::{icmpv6::echo_reply::EchoReplyPacket, ipv6::Ipv6Packet};
    if let Some(ip) = Ipv6Packet::new(packet)
        && ip.get_next_header() == IpNextHeaderProtocols::Icmpv6
        && let Some(reply) = EchoReplyPacket::new(ip.payload())
    {
        if reply.get_identifier() == PING_PKT_ID && reply.get_sequence_number() == seq {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Received a ping reply with mismatched id : Expected {PING_PKT_ID}, {seq}; Actual {}, {}",
                reply.get_identifier(),
                reply.get_sequence_number()
            ))
        }
    } else {
        Err(anyhow::anyhow!("Received a packet that isn't a ping reply"))
    }
}
