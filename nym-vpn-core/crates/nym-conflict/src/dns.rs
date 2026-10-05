// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Detects whether something on the system is intercepting or rerouting DNS
//! queries before they reach NymVPN's own resolver.

use std::net::Ipv4Addr;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use std::{net::SocketAddr, time::Duration};

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tokio::net::UdpSocket;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use crate::Conflict;

/// Canary domain resolved by [`detect`] to detect DNS interception. Callers
/// that also run NymVPN's own DNS resolver (see `nym-vpn-lib`'s `resolver`
/// module) must answer this domain with [`PROBE_ADDR`], regardless of
/// ad-block/filter configuration.
pub const PROBE_DOMAIN: &str = "nym-conflict-probe.invalid.";

/// The address NymVPN's own DNS resolver answers [`PROBE_DOMAIN`] with.
/// Taken from the IPv4 documentation range (RFC 5737 TEST-NET-1) so it can
/// never be a real, independently-routable answer.
pub const PROBE_ADDR: Ipv4Addr = Ipv4Addr::new(192, 0, 2, 53);

#[cfg(not(any(target_os = "android", target_os = "ios")))]
const PROBE_TIMEOUT: Duration = Duration::from_secs(2);

#[cfg(not(any(target_os = "android", target_os = "ios")))]
const PROBE_ID: [u8; 2] = [0x4e, 0x59];

/// Scan for DNS interception of queries aimed at `resolver`.
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub(crate) async fn detect(resolver: SocketAddr) -> Vec<Conflict> {
    if probe_dns_interception(resolver).await {
        vec![Conflict::InterceptedDns]
    } else {
        Vec::new()
    }
}

/// Queries `resolver` directly: via libc, systemd-resolved answers `.invalid`
/// with NXDOMAIN locally (RFC 6761) and never forwards it.
#[cfg(not(any(target_os = "android", target_os = "ios")))]
async fn probe_dns_interception(resolver: SocketAddr) -> bool {
    let probe = async {
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await?;
        socket.connect(resolver).await?;
        socket.send(&probe_query()).await?;
        let mut buf = [0u8; 512];
        let len = socket.recv(&mut buf).await?;
        let reply = &buf[..len];
        Ok::<_, std::io::Error>(
            len >= 12
                && reply.starts_with(&PROBE_ID)
                && !reply[12..].windows(4).any(|w| w == PROBE_ADDR.octets()),
        )
    };
    match tokio::time::timeout(PROBE_TIMEOUT, probe).await {
        Ok(Ok(intercepted)) => intercepted,
        Ok(Err(error)) => {
            tracing::debug!("conflict probe: DNS query failed: {error}");
            false
        }
        Err(_) => {
            tracing::debug!("conflict probe: DNS query timed out");
            false
        }
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn probe_query() -> Vec<u8> {
    let mut packet = vec![
        PROBE_ID[0],
        PROBE_ID[1],
        0x01,
        0x00,
        0x00,
        0x01,
        0,
        0,
        0,
        0,
        0,
        0,
    ];
    for label in PROBE_DOMAIN.split('.').filter(|label| !label.is_empty()) {
        packet.push(label.len() as u8);
        packet.extend_from_slice(label.as_bytes());
    }
    packet.extend_from_slice(&[0, 0, 1, 0, 1]);
    packet
}

#[cfg(all(test, not(any(target_os = "android", target_os = "ios"))))]
mod tests {
    use super::*;

    async fn probe_against(id: [u8; 2], answer: Option<Ipv4Addr>) -> bool {
        let listener = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
        let resolver = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let mut buf = [0u8; 512];
            let (len, peer) = listener.recv_from(&mut buf).await.unwrap();
            let mut reply = buf[..len].to_vec();
            reply[..2].copy_from_slice(&id);
            if let Some(addr) = answer {
                reply.extend_from_slice(&[0xc0, 0x0c, 0, 1, 0, 1, 0, 0, 0, 3, 0, 4]);
                reply.extend_from_slice(&addr.octets());
            }
            listener.send_to(&reply, peer).await.unwrap();
        });
        probe_dns_interception(resolver).await
    }

    #[tokio::test]
    async fn probe_clear_when_resolver_answers_probe_addr() {
        assert!(!probe_against(PROBE_ID, Some(PROBE_ADDR)).await);
    }

    #[tokio::test]
    async fn probe_reports_other_answer_or_no_answer() {
        assert!(probe_against(PROBE_ID, Some(Ipv4Addr::new(10, 1, 2, 3))).await);
        assert!(probe_against(PROBE_ID, None).await);
    }

    #[tokio::test]
    async fn probe_ignores_reply_with_wrong_id() {
        assert!(!probe_against([0, 0], Some(Ipv4Addr::new(10, 1, 2, 3))).await);
    }
}
