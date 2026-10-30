// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::IpAddr;

/// Converts the custom DNS flag and plain list of DNS server addresses stored by config versions
/// before v15.
pub fn from_legacy_ips(enabled: bool, ips: Vec<IpAddr>) -> nym_vpn_lib_types::CustomDnsSettings {
    nym_vpn_lib_types::CustomDnsSettings {
        enabled,
        dns_servers: ips
            .into_iter()
            .map(|address| nym_vpn_lib_types::DnsServer::Plain {
                address,
                port: None,
                ip_proto: nym_vpn_lib_types::PlainDnsProtocol::UdpAndTcp,
            })
            .collect(),
    }
}

pub(crate) mod v15 {
    use std::net::IpAddr;

    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    pub struct CustomDnsSettings {
        pub enabled: bool,
        pub dns_servers: Vec<DnsServer>,
    }

    #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum PlainDnsProtocol {
        Udp,
        Tcp,
        UdpAndTcp,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum DnsServer {
        Plain {
            address: IpAddr,
            port: Option<u16>,
            ip_proto: PlainDnsProtocol,
        },
        Tls {
            address: IpAddr,
            port: Option<u16>,
            server_name: String,
        },
        Https {
            address: IpAddr,
            port: Option<u16>,
            server_name: String,
            query_path: Option<String>,
        },
        Quic {
            address: IpAddr,
            port: Option<u16>,
            server_name: String,
        },
        Http3 {
            address: IpAddr,
            port: Option<u16>,
            server_name: String,
            query_path: Option<String>,
        },
    }

    impl From<CustomDnsSettings> for nym_vpn_lib_types::CustomDnsSettings {
        fn from(value: CustomDnsSettings) -> Self {
            Self {
                enabled: value.enabled,
                dns_servers: value.dns_servers.into_iter().map(Into::into).collect(),
            }
        }
    }

    impl From<PlainDnsProtocol> for nym_vpn_lib_types::PlainDnsProtocol {
        fn from(value: PlainDnsProtocol) -> Self {
            match value {
                PlainDnsProtocol::Udp => Self::Udp,
                PlainDnsProtocol::Tcp => Self::Tcp,
                PlainDnsProtocol::UdpAndTcp => Self::UdpAndTcp,
            }
        }
    }

    impl From<DnsServer> for nym_vpn_lib_types::DnsServer {
        fn from(value: DnsServer) -> Self {
            match value {
                DnsServer::Plain {
                    address,
                    port,
                    ip_proto,
                } => Self::Plain {
                    address,
                    port,
                    ip_proto: ip_proto.into(),
                },
                DnsServer::Tls {
                    address,
                    port,
                    server_name,
                } => Self::Tls {
                    address,
                    port,
                    server_name,
                },
                DnsServer::Https {
                    address,
                    port,
                    server_name,
                    query_path,
                } => Self::Https {
                    address,
                    port,
                    server_name,
                    query_path,
                },
                DnsServer::Quic {
                    address,
                    port,
                    server_name,
                } => Self::Quic {
                    address,
                    port,
                    server_name,
                },
                DnsServer::Http3 {
                    address,
                    port,
                    server_name,
                    query_path,
                } => Self::Http3 {
                    address,
                    port,
                    server_name,
                    query_path,
                },
            }
        }
    }

    // This is only required for the latest version of the external representation
    impl From<&nym_vpn_lib_types::CustomDnsSettings> for CustomDnsSettings {
        fn from(value: &nym_vpn_lib_types::CustomDnsSettings) -> Self {
            Self {
                enabled: value.enabled,
                dns_servers: value.dns_servers.iter().map(Into::into).collect(),
            }
        }
    }

    impl From<nym_vpn_lib_types::PlainDnsProtocol> for PlainDnsProtocol {
        fn from(value: nym_vpn_lib_types::PlainDnsProtocol) -> Self {
            match value {
                nym_vpn_lib_types::PlainDnsProtocol::Udp => Self::Udp,
                nym_vpn_lib_types::PlainDnsProtocol::Tcp => Self::Tcp,
                nym_vpn_lib_types::PlainDnsProtocol::UdpAndTcp => Self::UdpAndTcp,
            }
        }
    }

    impl From<&nym_vpn_lib_types::DnsServer> for DnsServer {
        fn from(value: &nym_vpn_lib_types::DnsServer) -> Self {
            match value.clone() {
                nym_vpn_lib_types::DnsServer::Plain {
                    address,
                    port,
                    ip_proto,
                } => Self::Plain {
                    address,
                    port,
                    ip_proto: ip_proto.into(),
                },
                nym_vpn_lib_types::DnsServer::Tls {
                    address,
                    port,
                    server_name,
                } => Self::Tls {
                    address,
                    port,
                    server_name,
                },
                nym_vpn_lib_types::DnsServer::Https {
                    address,
                    port,
                    server_name,
                    query_path,
                } => Self::Https {
                    address,
                    port,
                    server_name,
                    query_path,
                },
                nym_vpn_lib_types::DnsServer::Quic {
                    address,
                    port,
                    server_name,
                } => Self::Quic {
                    address,
                    port,
                    server_name,
                },
                nym_vpn_lib_types::DnsServer::Http3 {
                    address,
                    port,
                    server_name,
                    query_path,
                } => Self::Http3 {
                    address,
                    port,
                    server_name,
                    query_path,
                },
            }
        }
    }
}
