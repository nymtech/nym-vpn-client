// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use hickory_resolver::config::{NameServerConfig, ProtocolConfig};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use nym_firewall::{Endpoint, TransportProtocol};
use nym_vpn_lib_types::{DnsServer, PlainDnsProtocol};

pub trait DnsServerExt {
    /// Returns `NameServerConfig` compatible with hickory
    fn to_name_server_config(&self) -> NameServerConfig;

    /// Returns endpoints that can be used to configure firewall exceptions
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    fn firewall_endpoints(&self) -> Vec<Endpoint>;
}

impl DnsServerExt for DnsServer {
    fn to_name_server_config(&self) -> NameServerConfig {
        match self {
            Self::Plain {
                address,
                port,
                ip_proto,
            } => {
                let mut config = match ip_proto {
                    PlainDnsProtocol::Tcp => NameServerConfig::tcp(*address),
                    PlainDnsProtocol::Udp => NameServerConfig::udp(*address),
                    PlainDnsProtocol::UdpAndTcp => NameServerConfig::udp_and_tcp(*address),
                };
                if let Some(port) = port {
                    set_port(&mut config, *port);
                }
                config
            }
            Self::Tls {
                address,
                port,
                server_name,
            } => {
                let mut config = NameServerConfig::tls(*address, Arc::from(server_name.as_str()));
                if let Some(port) = port {
                    set_port(&mut config, *port);
                }
                config
            }
            Self::Https {
                address,
                port,
                server_name,
                query_path,
            } => {
                let mut config = NameServerConfig::https(
                    *address,
                    Arc::from(server_name.as_str()),
                    query_path.as_deref().map(Arc::from),
                );
                if let Some(port) = port {
                    set_port(&mut config, *port);
                }
                config
            }
            Self::Quic {
                address,
                port,
                server_name,
            } => {
                let mut config = NameServerConfig::quic(*address, Arc::from(server_name.as_str()));
                if let Some(port) = port {
                    set_port(&mut config, *port);
                }
                config
            }
            Self::Http3 {
                address,
                port,
                server_name,
                query_path,
            } => {
                let mut config = NameServerConfig::h3(
                    *address,
                    Arc::from(server_name.as_str()),
                    query_path.as_deref().map(Arc::from),
                );
                if let Some(port) = port {
                    set_port(&mut config, *port);
                }
                config
            }
        }
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    fn firewall_endpoints(&self) -> Vec<Endpoint> {
        let ns = self.to_name_server_config();

        ns.connections
            .iter()
            .map(|conn| {
                let proto = if conn.protocol.to_protocol().is_datagram() {
                    TransportProtocol::Udp
                } else {
                    TransportProtocol::Tcp
                };
                Endpoint::new(ns.ip, conn.port, proto)
            })
            .collect()
    }
}

/// Converts a hickory name server config into custom DNS servers, one per connection.
/// UDP and TCP connections on the same port are merged into a single plain server.
pub fn dns_servers_from_hickory_config(ns: &NameServerConfig) -> Vec<DnsServer> {
    let mut servers: Vec<DnsServer> = Vec::with_capacity(ns.connections.len());

    for conn in &ns.connections {
        let address = ns.ip;
        let port = Some(conn.port);

        match &conn.protocol {
            ProtocolConfig::Udp | ProtocolConfig::Tcp => {
                let ip_proto = if matches!(conn.protocol, ProtocolConfig::Udp) {
                    PlainDnsProtocol::Udp
                } else {
                    PlainDnsProtocol::Tcp
                };
                let existing = servers.iter_mut().find_map(|server| match server {
                    DnsServer::Plain {
                        port: existing_port,
                        ip_proto,
                        ..
                    } if *existing_port == port => Some(ip_proto),
                    _ => None,
                });
                match existing {
                    Some(existing) if *existing != ip_proto => {
                        *existing = PlainDnsProtocol::UdpAndTcp;
                    }
                    Some(_) => {}
                    None => servers.push(DnsServer::Plain {
                        address,
                        port,
                        ip_proto,
                    }),
                }
            }
            ProtocolConfig::Tls { server_name } => servers.push(DnsServer::Tls {
                address,
                port,
                server_name: server_name.as_ref().to_owned(),
            }),
            ProtocolConfig::Https { server_name, path } => servers.push(DnsServer::Https {
                address,
                port,
                server_name: server_name.as_ref().to_owned(),
                query_path: Some(path.as_ref().to_owned()),
            }),
            ProtocolConfig::Quic { server_name } => servers.push(DnsServer::Quic {
                address,
                port,
                server_name: server_name.as_ref().to_owned(),
            }),
            ProtocolConfig::H3 {
                server_name, path, ..
            } => servers.push(DnsServer::Http3 {
                address,
                port,
                server_name: server_name.as_ref().to_owned(),
                query_path: Some(path.as_ref().to_owned()),
            }),
        }
    }

    servers
}

fn set_port(config: &mut NameServerConfig, port: u16) {
    if port != 0 {
        for conn in config.connections.iter_mut() {
            conn.port = port;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::IpAddr;

    use super::*;

    const ADDRESS: IpAddr = IpAddr::V4(std::net::Ipv4Addr::new(9, 9, 9, 9));

    #[test]
    fn merges_udp_and_tcp_into_one_plain_server() {
        assert_eq!(
            dns_servers_from_hickory_config(&NameServerConfig::udp_and_tcp(ADDRESS)),
            vec![DnsServer::Plain {
                address: ADDRESS,
                port: Some(53),
                ip_proto: PlainDnsProtocol::UdpAndTcp,
            }]
        );
        assert_eq!(
            dns_servers_from_hickory_config(&NameServerConfig::tcp(ADDRESS)),
            vec![DnsServer::Plain {
                address: ADDRESS,
                port: Some(53),
                ip_proto: PlainDnsProtocol::Tcp,
            }]
        );
    }

    #[test]
    fn round_trips_through_name_server_config() {
        let servers = [
            DnsServer::Plain {
                address: ADDRESS,
                port: Some(5353),
                ip_proto: PlainDnsProtocol::Udp,
            },
            DnsServer::Tls {
                address: ADDRESS,
                port: Some(853),
                server_name: "dns.quad9.net".to_owned(),
            },
            DnsServer::Https {
                address: ADDRESS,
                port: Some(443),
                server_name: "dns.quad9.net".to_owned(),
                query_path: Some("/dns-query".to_owned()),
            },
            DnsServer::Quic {
                address: ADDRESS,
                port: Some(853),
                server_name: "dns.quad9.net".to_owned(),
            },
            DnsServer::Http3 {
                address: ADDRESS,
                port: Some(443),
                server_name: "dns.quad9.net".to_owned(),
                query_path: Some("/dns-query".to_owned()),
            },
        ];

        for server in servers {
            assert_eq!(
                dns_servers_from_hickory_config(&server.to_name_server_config()),
                vec![server]
            );
        }
    }
}
