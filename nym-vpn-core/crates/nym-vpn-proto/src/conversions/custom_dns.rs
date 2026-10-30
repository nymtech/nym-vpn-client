// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::IpAddr;

use crate::{
    conversions::ConversionError,
    proto::{
        self,
        dns_server::{DnsOverHttp3, DnsOverHttps, DnsOverQuic, DnsOverTls, PlainDns, Server},
    },
};

impl TryFrom<proto::CustomDnsSettings> for nym_vpn_lib_types::CustomDnsSettings {
    type Error = ConversionError;

    fn try_from(value: proto::CustomDnsSettings) -> Result<Self, Self::Error> {
        let dns_servers = value
            .dns_servers
            .into_iter()
            .map(nym_vpn_lib_types::DnsServer::try_from)
            .collect::<Result<_, _>>()?;

        Ok(Self { dns_servers })
    }
}

impl From<nym_vpn_lib_types::CustomDnsSettings> for proto::CustomDnsSettings {
    fn from(value: nym_vpn_lib_types::CustomDnsSettings) -> Self {
        Self {
            dns_servers: value
                .dns_servers
                .into_iter()
                .map(proto::DnsServer::from)
                .collect(),
        }
    }
}

impl TryFrom<proto::DnsServerList> for Vec<nym_vpn_lib_types::DnsServer> {
    type Error = ConversionError;

    fn try_from(value: proto::DnsServerList) -> Result<Self, Self::Error> {
        value
            .servers
            .into_iter()
            .map(nym_vpn_lib_types::DnsServer::try_from)
            .collect()
    }
}

impl From<Vec<nym_vpn_lib_types::DnsServer>> for proto::DnsServerList {
    fn from(value: Vec<nym_vpn_lib_types::DnsServer>) -> Self {
        Self {
            servers: value.into_iter().map(proto::DnsServer::from).collect(),
        }
    }
}

impl TryFrom<proto::DnsServer> for nym_vpn_lib_types::DnsServer {
    type Error = ConversionError;

    fn try_from(value: proto::DnsServer) -> Result<Self, Self::Error> {
        let server = value
            .server
            .ok_or(ConversionError::NoValueSet("DnsServer.server"))?;

        Ok(match server {
            Server::Plain(plain) => Self::Plain {
                address: decode_address(plain.address)?,
                port: decode_port(plain.port)?,
                ip_proto: proto::PlainDnsProtocol::try_from(plain.ip_proto)
                    .map_err(|e| ConversionError::Decode("DnsServer.plain.ip_proto", e))?
                    .into(),
            },
            Server::Tls(tls) => Self::Tls {
                address: decode_address(tls.address)?,
                port: decode_port(tls.port)?,
                server_name: tls.server_name,
            },
            Server::Https(https) => Self::Https {
                address: decode_address(https.address)?,
                port: decode_port(https.port)?,
                server_name: https.server_name,
                query_path: https.query_path,
            },
            Server::Quic(quic) => Self::Quic {
                address: decode_address(quic.address)?,
                port: decode_port(quic.port)?,
                server_name: quic.server_name,
            },
            Server::Http3(http3) => Self::Http3 {
                address: decode_address(http3.address)?,
                port: decode_port(http3.port)?,
                server_name: http3.server_name,
                query_path: http3.query_path,
            },
        })
    }
}

impl From<nym_vpn_lib_types::DnsServer> for proto::DnsServer {
    fn from(value: nym_vpn_lib_types::DnsServer) -> Self {
        let server = match value {
            nym_vpn_lib_types::DnsServer::Plain {
                address,
                port,
                ip_proto,
            } => Server::Plain(PlainDns {
                address: Some(address.into()),
                port: port.map(u32::from),
                ip_proto: proto::PlainDnsProtocol::from(ip_proto).into(),
            }),
            nym_vpn_lib_types::DnsServer::Tls {
                address,
                port,
                server_name,
            } => Server::Tls(DnsOverTls {
                address: Some(address.into()),
                port: port.map(u32::from),
                server_name,
            }),
            nym_vpn_lib_types::DnsServer::Https {
                address,
                port,
                server_name,
                query_path,
            } => Server::Https(DnsOverHttps {
                address: Some(address.into()),
                port: port.map(u32::from),
                server_name,
                query_path,
            }),
            nym_vpn_lib_types::DnsServer::Quic {
                address,
                port,
                server_name,
            } => Server::Quic(DnsOverQuic {
                address: Some(address.into()),
                port: port.map(u32::from),
                server_name,
            }),
            nym_vpn_lib_types::DnsServer::Http3 {
                address,
                port,
                server_name,
                query_path,
            } => Server::Http3(DnsOverHttp3 {
                address: Some(address.into()),
                port: port.map(u32::from),
                server_name,
                query_path,
            }),
        };

        Self {
            server: Some(server),
        }
    }
}

impl From<proto::PlainDnsProtocol> for nym_vpn_lib_types::PlainDnsProtocol {
    fn from(value: proto::PlainDnsProtocol) -> Self {
        match value {
            proto::PlainDnsProtocol::Udp => Self::Udp,
            proto::PlainDnsProtocol::Tcp => Self::Tcp,
            proto::PlainDnsProtocol::UdpAndTcp => Self::UdpAndTcp,
        }
    }
}

impl From<nym_vpn_lib_types::PlainDnsProtocol> for proto::PlainDnsProtocol {
    fn from(value: nym_vpn_lib_types::PlainDnsProtocol) -> Self {
        match value {
            nym_vpn_lib_types::PlainDnsProtocol::Udp => Self::Udp,
            nym_vpn_lib_types::PlainDnsProtocol::Tcp => Self::Tcp,
            nym_vpn_lib_types::PlainDnsProtocol::UdpAndTcp => Self::UdpAndTcp,
        }
    }
}

fn decode_address(address: Option<proto::IpAddr>) -> Result<IpAddr, ConversionError> {
    address
        .ok_or(ConversionError::NoValueSet("DnsServer.address"))
        .and_then(IpAddr::try_from)
}

fn decode_port(port: Option<u32>) -> Result<Option<u16>, ConversionError> {
    port.map(|port| {
        u16::try_from(port)
            .map_err(|_| ConversionError::generic(format!("DnsServer.port out of range: {port}")))
    })
    .transpose()
}
