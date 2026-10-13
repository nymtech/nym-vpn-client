// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{net::SocketAddr, time::Duration};

use nym_bridges::types::{PersistedClientConfig, Sufficiency};
pub(crate) use nym_bridges::{
    config::ClientConfig, connection::BridgeConn, error::TransportError, forward::UdpForwarder,
};
use nym_wg_go::amnezia::AmneziaConfig;

/// First WG datagram wait: five handshake attempts at RekeyTimeout (5s).
pub(crate) const INITIAL_FWD_CONNECTION_TIMEOUT: Duration = Duration::from_secs(21);
pub(crate) const BRIDGE_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

pub(crate) fn filter_bridge_params_ipv4_only(mut params: ClientConfig) -> ClientConfig {
    let addresses = match &mut params {
        ClientConfig::QuicPlain(cfg) => &mut cfg.addresses,
        ClientConfig::TlsPlain(cfg) => &mut cfg.addresses,
        ClientConfig::SshPlain(cfg) => &mut cfg.addresses,
    };
    addresses.retain(|addr| addr.is_ipv4());
    params
}

/// AmneziaWG translator protocol the bundled amneziawg-go speaks; bump together with libwg.
const AMNEZIA_PROTOCOL_VERSION: u16 = 1;

/// Bridge translator endpoint on the entry gateway's own IP and the AmneziaWG config to dial it
/// with, when the gateway advertises valid params.
pub(crate) fn amnezia_entry(
    params: &PersistedClientConfig,
    entry_endpoint: SocketAddr,
) -> Option<(SocketAddr, AmneziaConfig)> {
    params
        .amnezia
        .iter()
        .filter(|a| a.protocol_version == AMNEZIA_PROTOCOL_VERSION && a.is_sufficient())
        .find_map(|amnezia| {
            let endpoint = *amnezia
                .addresses
                .iter()
                .find(|addr| addr.ip() == entry_endpoint.ip())?;
            let p = &amnezia.params;
            let config = AmneziaConfig {
                init_pkt_junk_size: p.init_pkt_junk_size,
                response_pkt_junk_size: p.response_pkt_junk_size,
                under_load_pkt_magic_header: p.under_load_pkt_magic_header.try_into().ok()?,
                transport_pkt_magic_header: p.transport_pkt_magic_header.try_into().ok()?,
                // Handshake types stay WireGuard's (1, 2) from `BASE`: the bridge cannot rewrite them.
                ..AmneziaConfig::BASE
            };
            config.validate().then_some((endpoint, config))
        })
}

#[cfg(test)]
mod tests {
    use nym_bridges::types::amnezia::{AmneziaClientOptions, AmneziaParams};

    use super::*;

    fn profile(
        protocol_version: u16,
        addresses: Vec<SocketAddr>,
        transport_header: u32,
    ) -> AmneziaClientOptions {
        AmneziaClientOptions {
            protocol_version,
            addresses,
            params: AmneziaParams {
                init_pkt_junk_size: 40,
                response_pkt_junk_size: 50,
                under_load_pkt_magic_header: 1_766_607_858,
                transport_pkt_magic_header: transport_header,
                cookie_pkt_junk_size: 0,
                transport_pkt_junk_size: 0,
                under_load_pkt_magic_header_max: None,
                transport_pkt_magic_header_max: None,
                header_protection_key: None,
                random_trailers: false,
            },
        }
    }

    fn bridge_params(addresses: Vec<SocketAddr>, transport_header: u32) -> PersistedClientConfig {
        PersistedClientConfig {
            version: "0".into(),
            transports: vec![],
            amnezia: vec![profile(1, addresses, transport_header)],
        }
    }

    #[test]
    fn amnezia_entry_uses_translator_on_entry_ip() {
        let entry: SocketAddr = "1.2.3.4:51822".parse().unwrap();
        let params = bridge_params(
            vec![
                "[2001:db8::1]:51830".parse().unwrap(),
                "1.2.3.4:51830".parse().unwrap(),
            ],
            2_028_465_083,
        );

        let (endpoint, config) = amnezia_entry(&params, entry).unwrap();

        assert_eq!(endpoint, "1.2.3.4:51830".parse().unwrap());
        assert_eq!(config.init_pkt_junk_size, 40);
        assert_eq!(config.transport_pkt_magic_header, 2_028_465_083);
        assert_eq!(config.junk_pkt_count, AmneziaConfig::BASE.junk_pkt_count);
        assert_eq!(config.init_pkt_magic_header, 1);
        assert_eq!(config.response_pkt_magic_header, 2);
    }

    #[test]
    fn amnezia_entry_falls_back_without_usable_params() {
        let entry: SocketAddr = "1.2.3.4:51822".parse().unwrap();
        let other_ip = bridge_params(vec!["5.6.7.8:51830".parse().unwrap()], 2_028_465_083);
        let colliding = bridge_params(vec!["1.2.3.4:51830".parse().unwrap()], 1_766_607_858);
        let not_i32 = bridge_params(vec!["1.2.3.4:51830".parse().unwrap()], u32::MAX);
        let absent = PersistedClientConfig {
            amnezia: vec![],
            ..bridge_params(vec![], 0)
        };
        let unknown_version = PersistedClientConfig {
            amnezia: vec![profile(
                2,
                vec!["1.2.3.4:51830".parse().unwrap()],
                2_028_465_083,
            )],
            ..bridge_params(vec![], 0)
        };

        assert!(amnezia_entry(&unknown_version, entry).is_none());
        assert!(amnezia_entry(&other_ip, entry).is_none());
        assert!(amnezia_entry(&colliding, entry).is_none());
        assert!(amnezia_entry(&not_i32, entry).is_none());
        assert!(amnezia_entry(&absent, entry).is_none());
    }

    #[test]
    fn amnezia_entry_picks_supported_version_during_upgrade() {
        let entry: SocketAddr = "1.2.3.4:51822".parse().unwrap();
        let params = PersistedClientConfig {
            amnezia: vec![
                profile(2, vec!["1.2.3.4:51831".parse().unwrap()], 2_028_465_083),
                profile(1, vec!["1.2.3.4:51830".parse().unwrap()], 2_028_465_083),
            ],
            ..bridge_params(vec![], 0)
        };

        let (endpoint, _) = amnezia_entry(&params, entry).unwrap();

        assert_eq!(endpoint, "1.2.3.4:51830".parse().unwrap());
    }
}
