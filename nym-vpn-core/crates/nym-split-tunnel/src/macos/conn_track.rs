// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{collections::HashMap, fmt::Write, net::SocketAddr, sync::Arc};

use tokio::sync::Mutex;

use libc::pid_t;
use pnet_packet::{
    Packet,
    ethernet::EtherTypes,
    ip::IpNextHeaderProtocols,
    ipv4::Ipv4Packet,
    ipv6::Ipv6Packet,
    tcp::{TcpFlags, TcpPacket},
};

use super::tun::PktapPacket;

/// TCP connection identifier
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct TcpConnectionIdent {
    pub src: SocketAddr,
    pub dst: SocketAddr,
}

/// TCP connection metadata
#[derive(Debug, Default, Clone, Copy, Hash, Eq, PartialEq)]
pub struct TcpConnectionMeta {
    /// Last seen ack of outgoing packet
    pub ack: u32,
}

/// TCP connection tracking data
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct TcpConnectionTrack {
    /// 4-tuple identifying the TCP connection
    pub ident: TcpConnectionIdent,
    /// Metadata associated with TCP connection
    pub meta: TcpConnectionMeta,
}

/// Registry of active TCP connections per process
#[derive(Debug, Clone, Default)]
pub struct TcpConnectionStates {
    inner: Arc<Mutex<HashMap<pid_t, HashMap<TcpConnectionIdent, TcpConnectionMeta>>>>,
}

impl TcpConnectionStates {
    /// Add connection metadata corresponding to the process and individual TCP connection.
    pub async fn add(
        &mut self,
        pid: pid_t,
        ident: TcpConnectionIdent,
        metadata: TcpConnectionMeta,
    ) {
        let mut states = self.inner.lock().await;

        let _ = states.entry(pid).or_default().insert(ident, metadata);
    }

    /// Remove connection metadata corresponding to the process and individual TCP connection.
    pub async fn remove(&mut self, pid: pid_t, ident: TcpConnectionIdent) {
        let mut states = self.inner.lock().await;

        if let Some(conns) = states.get_mut(&pid) {
            conns.remove(&ident);
            if conns.is_empty() {
                let _ = states.remove(&pid);
            }
        }
    }

    /// Update connection metadata but only if pid/ident exist
    pub async fn update_meta(
        &mut self,
        pid: pid_t,
        ident: TcpConnectionIdent,
        mutator: impl FnOnce(&mut TcpConnectionMeta),
    ) {
        let mut states = self.inner.lock().await;

        if let Some(proc_conn_states) = states.get_mut(&pid)
            && let Some(meta) = proc_conn_states.get_mut(&ident)
        {
            mutator(meta)
        }
    }

    /// Clear all connection metadata associated with the given processes.
    ///
    /// Returns removed connection states.
    pub async fn clear(&mut self, pids: &[pid_t]) -> Vec<TcpConnectionTrack> {
        let mut states = self.inner.lock().await;

        pids.iter()
            .flat_map(|pid| {
                states.remove(pid).map(|tcp_conns| {
                    tcp_conns
                        .into_iter()
                        .map(|(ident, meta)| TcpConnectionTrack { ident, meta })
                        .collect::<Vec<TcpConnectionTrack>>()
                })
            })
            .flatten()
            .collect()
    }

    /// Clear all connection states for all processes.
    pub async fn clear_all(&mut self) {
        let mut states = self.inner.lock().await;

        states.clear();
    }
}

/// Update TCP connections state from outgoing packet
pub async fn track_connection_state(
    tcp_conn_states: &mut TcpConnectionStates,
    packet: &PktapPacket,
) {
    match packet.frame.get_ethertype() {
        EtherTypes::Ipv4 => {
            let Some(ipv4_packet) = Ipv4Packet::new(packet.frame.payload()) else {
                return;
            };

            if ipv4_packet.get_next_level_protocol() != IpNextHeaderProtocols::Tcp {
                return;
            }

            let Some(tcp_packet) = TcpPacket::new(ipv4_packet.payload()) else {
                return;
            };

            let src = SocketAddr::from((ipv4_packet.get_source(), tcp_packet.get_source()));
            let dst =
                SocketAddr::from((ipv4_packet.get_destination(), tcp_packet.get_destination()));
            let ident = TcpConnectionIdent { src, dst };

            handle_tcp_packet(tcp_conn_states, packet.header.pth_pid, ident, tcp_packet).await;
        }
        EtherTypes::Ipv6 => {
            let Some(ipv6_packet) = Ipv6Packet::new(packet.frame.payload()) else {
                return;
            };

            if ipv6_packet.get_next_header() != IpNextHeaderProtocols::Tcp {
                return;
            }

            let Some(tcp_packet) = TcpPacket::new(ipv6_packet.payload()) else {
                return;
            };

            let src = SocketAddr::from((ipv6_packet.get_source(), tcp_packet.get_source()));
            let dst =
                SocketAddr::from((ipv6_packet.get_destination(), tcp_packet.get_destination()));
            let ident = TcpConnectionIdent { src, dst };

            handle_tcp_packet(tcp_conn_states, packet.header.pth_pid, ident, tcp_packet).await;
        }
        _ => {}
    }
}

async fn handle_tcp_packet<'a>(
    tcp_conn_states: &mut TcpConnectionStates,
    pid: pid_t,
    ident: TcpConnectionIdent,
    tcp_packet: TcpPacket<'a>,
) {
    let tcp_flags = tcp_packet.get_flags();

    tracing::trace!(
        "TCP {} {} -> {} seq:{} ack:{}",
        DisplayTcpFlags(tcp_flags),
        ident.src,
        ident.dst,
        tcp_packet.get_sequence(),
        tcp_packet.get_acknowledgement()
    );

    // For simplicity, FIN (or RST) immediately remove the state associated with connection.
    // A follow up ACK sent by the client will not match any state and thus be no-op
    if (tcp_flags & TcpFlags::FIN) != 0 || (tcp_flags & TcpFlags::RST) != 0 {
        tcp_conn_states.remove(pid, ident).await;
    } else if (tcp_flags & TcpFlags::SYN) != 0 {
        // Initial handshake client -> server
        tcp_conn_states
            .add(pid, ident, TcpConnectionMeta::default())
            .await;
    } else if (tcp_flags & TcpFlags::ACK) != 0 {
        // Either received during handshake or during data transfer
        tcp_conn_states
            .update_meta(pid, ident, |meta| {
                meta.ack = tcp_packet.get_acknowledgement();
            })
            .await;
    }
}

/// Type implementing `Display` trait for raw TCP flags
#[repr(transparent)]
pub struct DisplayTcpFlags(pub u8);

impl std::fmt::Display for DisplayTcpFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut matches = 0;
        for (flag, str) in [
            (TcpFlags::SYN, "SYN"),
            (TcpFlags::ACK, "ACK"),
            (TcpFlags::FIN, "FIN"),
            (TcpFlags::RST, "RST"),
        ] {
            if (self.0 & flag) != 0 {
                if matches > 0 {
                    f.write_char('+')?;
                }
                f.write_str(str)?;
                matches += 1;
            }
        }
        Ok(())
    }
}
