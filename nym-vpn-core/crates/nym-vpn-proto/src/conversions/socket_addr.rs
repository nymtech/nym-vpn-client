// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{conversions::ConversionError, proto};

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

impl From<SocketAddr> for proto::SocketAddr {
    fn from(addr: SocketAddr) -> Self {
        proto::SocketAddr {
            addr: Some(proto::IpAddr::from(addr.ip())),
            port: addr.port() as u32,
        }
    }
}

impl TryFrom<proto::SocketAddr> for SocketAddr {
    type Error = ConversionError;

    fn try_from(value: proto::SocketAddr) -> Result<Self, Self::Error> {
        let ip_addr = value
            .addr
            .ok_or(ConversionError::NoValueSet("SocketAddr.addr"))?;
        Ok(Self::new(IpAddr::try_from(ip_addr)?, value.port as u16))
    }
}

impl From<IpAddr> for proto::IpAddr {
    fn from(value: IpAddr) -> Self {
        match value {
            IpAddr::V4(v) => Self {
                ip_addr: Some(proto::ip_addr::IpAddr::V4(u32::from_be_bytes(v.octets()))),
            },
            IpAddr::V6(v) => Self {
                ip_addr: Some(proto::ip_addr::IpAddr::V6(v.octets().to_vec())),
            },
        }
    }
}

impl TryFrom<proto::IpAddr> for IpAddr {
    type Error = ConversionError;

    fn try_from(addr: proto::IpAddr) -> Result<Self, Self::Error> {
        let ip_addr = addr
            .ip_addr
            .ok_or(ConversionError::NoValueSet("IpAddr.ip_addr"))?;

        Ok(match ip_addr {
            proto::ip_addr::IpAddr::V4(v) => Self::V4(Ipv4Addr::from(v)),
            proto::ip_addr::IpAddr::V6(v) => {
                if v.len() < 16 {
                    return Err(ConversionError::generic("invalid ipv6 address"));
                }
                let addr = u128::from_be_bytes(v[..16].try_into().unwrap());

                Self::V6(Ipv6Addr::from(addr))
            }
        })
    }
}

impl From<Vec<IpAddr>> for proto::IpAddrList {
    fn from(value: Vec<IpAddr>) -> Self {
        let ips: Vec<proto::IpAddr> = value.into_iter().map(proto::IpAddr::from).collect();
        proto::IpAddrList { ips }
    }
}

impl TryFrom<proto::IpAddrList> for Vec<IpAddr> {
    type Error = ConversionError;

    fn try_from(value: proto::IpAddrList) -> Result<Self, Self::Error> {
        value
            .ips
            .into_iter()
            .map(IpAddr::try_from)
            .collect::<Result<Vec<_>, _>>()
    }
}
