// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{net::SocketAddr, time::Duration};

use nym_bridges::types::{
    PersistedClientConfig,
    amnezia::{PROTOCOL_AWG_1_0, PROTOCOL_AWG_3_1},
};
pub(crate) use nym_bridges::{
    config::ClientConfig, connection::BridgeConn, error::TransportError, forward::UdpForwarder,
};
use nym_vpn_lib_types::AmneziaProtocol;
use nym_wg_go::amnezia::AmneziaConfig;

/// First WG datagram wait: five handshake attempts at RekeyTimeout (5s).
pub(crate) const INITIAL_FWD_CONNECTION_TIMEOUT: Duration = Duration::from_secs(21);
pub(crate) const BRIDGE_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// How long QUIC gets before TLS and SSH bridges are tried.
pub(crate) const QUIC_FALLBACK_TIMEOUT: Duration = Duration::from_secs(5);

pub(crate) fn filter_bridge_params_ipv4_only(mut params: ClientConfig) -> ClientConfig {
    let addresses = match &mut params {
        ClientConfig::QuicPlain(cfg) => &mut cfg.addresses,
        ClientConfig::TlsPlain(cfg) => &mut cfg.addresses,
        ClientConfig::SshPlain(cfg) => &mut cfg.addresses,
        ClientConfig::Amnezia(cfg) => &mut cfg.addresses,
    };
    addresses.retain(|addr| addr.is_ipv4());
    params
}

/// AmneziaWG translator protocols the bundled amneziawg-go speaks, most preferred first.
const AMNEZIA_PROTOCOLS: [u16; 2] = [PROTOCOL_AWG_3_1, PROTOCOL_AWG_1_0];

/// Bridge translator endpoint on the entry gateway's own IP and the AmneziaWG config to dial it
/// with, for the most preferred protocol of at least `min_version` the gateway advertises with
/// valid params.
pub(crate) fn amnezia_entry(
    params: &PersistedClientConfig,
    entry_endpoint: SocketAddr,
    min_version: u16,
) -> Option<(SocketAddr, AmneziaProtocol, AmneziaConfig)> {
    AMNEZIA_PROTOCOLS.iter().find_map(|version| {
        params
            .amnezia_profiles()
            .filter(|a| a.protocol_version == *version && *version >= min_version)
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
                    cookie_pkt_junk_size: p.cookie_pkt_junk_size,
                    transport_pkt_junk_size: p.transport_pkt_junk_size,
                    under_load_pkt_magic_header_max: header_max(p.under_load_pkt_magic_header_max)?,
                    transport_pkt_magic_header_max: header_max(p.transport_pkt_magic_header_max)?,
                    header_protection_key: p.header_protection_key.clone(),
                    random_trailers: p.random_trailers,
                    // Handshake types stay WireGuard's (1, 2): the bridge cannot rewrite them.
                    // Junk packets are drawn per connection, so their shape is not fixed.
                    ..AmneziaConfig::junk(&mut rand::rng())
                };
                config.validate().then_some((
                    endpoint,
                    if *version == PROTOCOL_AWG_3_1 {
                        AmneziaProtocol::Awg31
                    } else {
                        AmneziaProtocol::Awg10
                    },
                    config,
                ))
            })
    })
}

/// Range end as amneziawg-go reads it; `None` inside means out of range.
fn header_max(max: Option<u32>) -> Option<Option<i32>> {
    max.map(i32::try_from).transpose().ok()
}

#[cfg(test)]
mod tests {
    use nym_bridges::types::amnezia::{AmneziaClientOptions, AmneziaParams};

    use super::*;

    fn awg(profiles: Vec<AmneziaClientOptions>) -> Vec<ClientConfig> {
        profiles.into_iter().map(ClientConfig::Amnezia).collect()
    }

    fn profile(
        protocol_version: u16,
        addresses: Vec<SocketAddr>,
        transport_header: u32,
    ) -> AmneziaClientOptions {
        let mut params = AmneziaParams {
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
        };
        if protocol_version == PROTOCOL_AWG_3_1 {
            params.cookie_pkt_junk_size = 20;
            params.transport_pkt_junk_size = 16;
            params.under_load_pkt_magic_header_max = Some(1_766_607_958);
            params.transport_pkt_magic_header_max = Some(transport_header.saturating_add(100));
            params.header_protection_key = Some("ab".repeat(32));
            params.random_trailers = true;
        }
        AmneziaClientOptions {
            protocol_version,
            addresses,
            params,
        }
    }

    fn bridge_params(addresses: Vec<SocketAddr>, transport_header: u32) -> PersistedClientConfig {
        PersistedClientConfig {
            version: "0".into(),
            transports: vec![],
            transports_v2: awg(vec![profile(1, addresses, transport_header)]),
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

        let (endpoint, _, config) = amnezia_entry(&params, entry, 0).unwrap();

        assert_eq!(endpoint, "1.2.3.4:51830".parse().unwrap());
        assert_eq!(config.init_pkt_junk_size, 40);
        assert_eq!(config.transport_pkt_magic_header, 2_028_465_083);
        assert!((3..=8).contains(&config.junk_pkt_count));
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
            transports_v2: awg(vec![]),
            ..bridge_params(vec![], 0)
        };
        let unknown_version = PersistedClientConfig {
            transports_v2: awg(vec![profile(
                99,
                vec!["1.2.3.4:51830".parse().unwrap()],
                2_028_465_083,
            )]),
            ..bridge_params(vec![], 0)
        };

        assert!(amnezia_entry(&unknown_version, entry, 0).is_none());
        assert!(amnezia_entry(&other_ip, entry, 0).is_none());
        assert!(amnezia_entry(&colliding, entry, 0).is_none());
        assert!(amnezia_entry(&not_i32, entry, 0).is_none());
        assert!(amnezia_entry(&absent, entry, 0).is_none());
    }

    #[test]
    fn amnezia_entry_prefers_awg_3_1_during_upgrade() {
        let entry: SocketAddr = "1.2.3.4:51822".parse().unwrap();
        let params = PersistedClientConfig {
            transports_v2: awg(vec![
                profile(
                    PROTOCOL_AWG_1_0,
                    vec!["1.2.3.4:51830".parse().unwrap()],
                    2_028_465_083,
                ),
                profile(
                    PROTOCOL_AWG_3_1,
                    vec!["1.2.3.4:51831".parse().unwrap()],
                    2_028_465_083,
                ),
            ]),
            ..bridge_params(vec![], 0)
        };

        let (endpoint, _, config) = amnezia_entry(&params, entry, 0).unwrap();

        assert_eq!(endpoint, "1.2.3.4:51831".parse().unwrap());
        assert_eq!(config.transport_pkt_junk_size, 16);
        assert_eq!(config.cookie_pkt_junk_size, 20);
        assert_eq!(config.transport_pkt_magic_header_max, Some(2_028_465_183));
        assert_eq!(config.header_protection_key, Some("ab".repeat(32)));
        assert!(config.random_trailers);
        assert_eq!(config.init_pkt_magic_header, 1);
    }

    #[test]
    fn amnezia_entry_skips_versions_below_the_minimum() {
        let entry: SocketAddr = "1.2.3.4:51822".parse().unwrap();
        let awg_1_0_only = bridge_params(vec!["1.2.3.4:51830".parse().unwrap()], 2_028_465_083);

        assert!(amnezia_entry(&awg_1_0_only, entry, PROTOCOL_AWG_1_0).is_some());
        assert!(amnezia_entry(&awg_1_0_only, entry, PROTOCOL_AWG_3_1).is_none());
    }

    #[test]
    fn amnezia_entry_falls_back_to_awg_1_0() {
        let entry: SocketAddr = "1.2.3.4:51822".parse().unwrap();
        let mut broken_3_1 = profile(
            PROTOCOL_AWG_3_1,
            vec!["1.2.3.4:51831".parse().unwrap()],
            2_028_465_083,
        );
        broken_3_1.params.transport_pkt_junk_size = 8;
        let params = PersistedClientConfig {
            transports_v2: awg(vec![
                broken_3_1,
                profile(
                    PROTOCOL_AWG_1_0,
                    vec!["1.2.3.4:51830".parse().unwrap()],
                    2_028_465_083,
                ),
            ]),
            ..bridge_params(vec![], 0)
        };

        let (endpoint, _, config) = amnezia_entry(&params, entry, 0).unwrap();

        assert_eq!(endpoint, "1.2.3.4:51830".parse().unwrap());
        assert_eq!(config.transport_pkt_junk_size, 0);
    }

    /// Bridge params as the nym-bridges `test-env` gateway serves them (`test-config/server.toml`).
    const LAB_BRIDGE_PARAMS: &str = r#"{"version":"0","transports":[],"transports_v2":[
        {"transport_type":"amnezia","args":{"protocol_version":1,"addresses":["192.168.100.3:51830"],"params":{
            "init_pkt_junk_size":44,"response_pkt_junk_size":52,
            "under_load_pkt_magic_header":123456789,"transport_pkt_magic_header":1623456789}}},
        {"transport_type":"amnezia","args":{"protocol_version":2,"addresses":["192.168.100.3:51831"],"params":{
            "init_pkt_junk_size":44,"response_pkt_junk_size":52,
            "cookie_pkt_junk_size":20,"transport_pkt_junk_size":16,
            "under_load_pkt_magic_header":123456789,"under_load_pkt_magic_header_max":123456889,
            "transport_pkt_magic_header":1623456789,"transport_pkt_magic_header_max":1623456889,
            "header_protection_key":"0f1e2d3c4b5a69788796a5b4c3d2e1f00112233445566778899aabbccddeeff0",
            "random_trailers":false}}}]}"#;

    /// The nym-bridges lab (`test-env/awg-lab.sh`, `awg-up ... awg10|awg31`) sends exactly these
    /// amneziawg-go settings through the translator to kernel WireGuard, so the client config that
    /// reaches libwg is the one proven on the wire.
    #[test]
    fn amnezia_entry_matches_lab_verified_uapi() {
        let params: PersistedClientConfig = serde_json::from_str(LAB_BRIDGE_PARAMS).unwrap();
        let entry: SocketAddr = "192.168.100.3:51822".parse().unwrap();
        let uapi = |config: &AmneziaConfig| {
            // Junk packets are drawn per connection; the lab verifies the rest with the BASE junk.
            let config = AmneziaConfig {
                junk_pkt_count: AmneziaConfig::BASE.junk_pkt_count,
                junk_pkt_min_size: AmneziaConfig::BASE.junk_pkt_min_size,
                junk_pkt_max_size: AmneziaConfig::BASE.junk_pkt_max_size,
                ..config.clone()
            };
            let mut builder = nym_wg_go::uapi::UapiConfigBuilder::new();
            config.append_to(&mut builder);
            String::from_utf8(builder.into_bytes()).unwrap()
        };

        let (endpoint, protocol, awg_3_1) = amnezia_entry(&params, entry, 0).unwrap();
        assert_eq!(protocol, AmneziaProtocol::Awg31);
        assert_eq!(endpoint, "192.168.100.3:51831".parse().unwrap());
        assert_eq!(
            uapi(&awg_3_1),
            "jc=4\njmin=40\njmax=70\ns1=44\ns2=52\nh1=1\nh2=2\nh3=123456789-123456889\n\
             h4=1623456789-1623456889\ns3=20\ns4=16\n\
             header_protection_key=0f1e2d3c4b5a69788796a5b4c3d2e1f00112233445566778899aabbccddeeff0\n\n"
        );

        let awg_1_0_only = PersistedClientConfig {
            transports_v2: awg(params.amnezia_profiles().take(1).cloned().collect()),
            ..params
        };
        let (endpoint, protocol, awg_1_0) = amnezia_entry(&awg_1_0_only, entry, 0).unwrap();
        assert_eq!(protocol, AmneziaProtocol::Awg10);
        assert_eq!(endpoint, "192.168.100.3:51830".parse().unwrap());
        assert_eq!(
            uapi(&awg_1_0),
            "jc=4\njmin=40\njmax=70\ns1=44\ns2=52\nh1=1\nh2=2\nh3=123456789\nh4=1623456789\n\n"
        );
    }
}
