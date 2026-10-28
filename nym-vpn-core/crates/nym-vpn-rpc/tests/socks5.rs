// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{fmt::Write, net::SocketAddr, str::FromStr};

use minicbor::{Decode, Encode};
use nym_vpn_lib_types::{
    EnableSocks5Request, ExitPoint, HttpRpcSettings, NodeIdentity, Recipient, Socks5Settings,
    Socks5State, Socks5Status,
};

const SNAPSHOT_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/snapshots/socks5.diag");

fn exit_points() -> Vec<ExitPoint> {
    vec![
        ExitPoint::Address {
            address: Box::new(
                Recipient::from_str("MNrmKzuKjNdbEhfPUzVNfjw63oBQNSayqoQKGL4JjAV.6fDcSN6faGpvA3pd3riCwjpzXc7RQfWmGMa82UVoEwKE@d5adfJNtcdZW2XwK85JAAU8nXAs9JCPYn2RNvDLZn4e").unwrap(),
            ),
        },
        ExitPoint::Gateway {
            identity: NodeIdentity::from_base58_string(
                "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42",
            )
            .unwrap(),
        },
        ExitPoint::Country {
            two_letter_iso_country_code: "CH".to_owned(),
        },
        ExitPoint::Region {
            region: "Zurich".to_owned(),
        },
        ExitPoint::Random,
        ExitPoint::Auto {
            exclude_entry_point_country: true,
            exclude_user_country: false,
        },
    ]
}

fn enable_requests() -> Vec<EnableSocks5Request> {
    exit_points()
        .into_iter()
        .map(|exit_point| EnableSocks5Request {
            socks5_settings: Socks5Settings {
                listen_address: Some("127.0.0.1:1080".parse().unwrap()),
            },
            http_rpc_settings: HttpRpcSettings {
                listen_address: None,
            },
            exit_point,
        })
        .collect()
}

fn statuses() -> Vec<Socks5Status> {
    [
        Socks5State::Disabled,
        Socks5State::Idle,
        Socks5State::Connected,
        Socks5State::Error,
    ]
    .into_iter()
    .map(|state| Socks5Status {
        state,
        socks5_settings: Socks5Settings {
            listen_address: Some("[::1]:1080".parse().unwrap()),
        },
        http_rpc_settings: HttpRpcSettings {
            listen_address: Some("127.0.0.1:8545".parse().unwrap()),
        },
        error_message: (state == Socks5State::Error).then(|| "boom".to_owned()),
        active_connections: 3,
    })
    .collect()
}

fn assert_status_eq(left: &Socks5Status, right: &Socks5Status) {
    assert_eq!(left.state, right.state);
    assert_eq!(left.socks5_settings, right.socks5_settings);
    assert_eq!(left.http_rpc_settings, right.http_rpc_settings);
    assert_eq!(left.error_message, right.error_message);
    assert_eq!(left.active_connections, right.active_connections);
}

#[test]
fn round_trip() {
    for request in enable_requests() {
        let bytes = minicbor::to_vec(&request).unwrap();
        assert_eq!(
            minicbor::decode::<EnableSocks5Request>(&bytes).unwrap(),
            request
        );
    }
    for status in statuses() {
        let bytes = minicbor::to_vec(&status).unwrap();
        assert_status_eq(&minicbor::decode::<Socks5Status>(&bytes).unwrap(), &status);
    }
}

#[test]
fn encoding_matches_snapshot() {
    let mut rendered = String::new();
    for request in enable_requests() {
        let bytes = minicbor::to_vec(&request).unwrap();
        writeln!(rendered, "{}", minicbor::display(&bytes)).unwrap();
    }
    for status in statuses() {
        let bytes = minicbor::to_vec(&status).unwrap();
        writeln!(rendered, "{}", minicbor::display(&bytes)).unwrap();
    }

    if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
        std::fs::write(SNAPSHOT_PATH, &rendered).unwrap();
    }
    assert_eq!(rendered, std::fs::read_to_string(SNAPSHOT_PATH).unwrap());
}

#[derive(Debug, Encode, Decode)]
struct Socks5SettingsNext {
    #[n(0)]
    listen_address: Option<SocketAddr>,
    #[n(1)]
    username: Option<String>,
}

#[test]
fn new_optional_field_is_compatible() {
    let newer = Socks5SettingsNext {
        listen_address: Some("127.0.0.1:1080".parse().unwrap()),
        username: Some("alice".to_owned()),
    };
    let decoded: Socks5Settings = minicbor::decode(&minicbor::to_vec(&newer).unwrap()).unwrap();
    assert_eq!(decoded.listen_address, newer.listen_address);

    let older = Socks5Settings {
        listen_address: Some("127.0.0.1:1080".parse().unwrap()),
    };
    let decoded: Socks5SettingsNext = minicbor::decode(&minicbor::to_vec(&older).unwrap()).unwrap();
    assert_eq!(decoded.listen_address, older.listen_address);
    assert_eq!(decoded.username, None);
}

#[derive(Debug, Encode, Decode)]
#[cbor(index_only)]
enum Socks5StateNext {
    #[n(0)]
    Disabled,
    #[n(4)]
    Degraded,
}

#[derive(Debug, Encode, Decode)]
struct Socks5StatusNext {
    #[n(0)]
    state: Socks5StateNext,
    #[n(1)]
    socks5_settings: Socks5Settings,
    #[n(2)]
    http_rpc_settings: HttpRpcSettings,
    #[n(3)]
    error_message: Option<String>,
    #[n(4)]
    active_connections: u32,
}

#[test]
fn unknown_enum_variant_is_rejected() {
    let status = |state| Socks5StatusNext {
        state,
        socks5_settings: Socks5Settings {
            listen_address: None,
        },
        http_rpc_settings: HttpRpcSettings {
            listen_address: None,
        },
        error_message: None,
        active_connections: 0,
    };

    let known = minicbor::to_vec(status(Socks5StateNext::Disabled)).unwrap();
    assert_eq!(
        minicbor::decode::<Socks5Status>(&known).unwrap().state,
        Socks5State::Disabled
    );

    let unknown = minicbor::to_vec(status(Socks5StateNext::Degraded)).unwrap();
    assert!(minicbor::decode::<Socks5Status>(&unknown).is_err());
}

#[derive(Encode)]
enum ExitPointRaw {
    #[n(1)]
    Gateway {
        #[n(0)]
        identity: String,
    },
}

#[test]
fn malformed_gateway_identity_is_a_decode_error() {
    let bytes = minicbor::to_vec(ExitPointRaw::Gateway {
        identity: "not-a-key".to_owned(),
    })
    .unwrap();
    assert!(minicbor::decode::<ExitPoint>(&bytes).is_err());
}
