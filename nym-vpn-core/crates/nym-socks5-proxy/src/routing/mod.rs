// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod domain;
pub mod ip;

#[cfg(test)]
mod tests;

pub use domain::DomainSet;
pub use ip::GeoIpDatabase;

use std::{
    io::Cursor,
    net::{IpAddr, SocketAddr},
    path::Path,
};

use anyhow::{Context, Result};
use async_compression::tokio::bufread::GzipDecoder;
use nym_socks5_proxy_ipc::InterfaceAddresses;
use tokio::io::AsyncReadExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingDecision {
    VpnTunnelInterface,
    DefaultInterface,
    Reject,
}

pub struct RoutingDatabase {
    pub geo_ip: GeoIpDatabase,
    pub domain: DomainSet,
}

impl RoutingDatabase {
    pub async fn load(excluded_countries: &[String], data_dir: &Path) -> Result<Self> {
        let geo_ip = GeoIpDatabase::load(excluded_countries, data_dir)
            .await
            .context("Failed to load GeoIP database")?;
        let domain = DomainSet::load(excluded_countries, data_dir)
            .await
            .context("Failed to load domain exclusion list")?;
        Ok(Self { geo_ip, domain })
    }
}

pub fn is_excluded_domain(host: &str, domain: &DomainSet) -> bool {
    domain.is_excluded(host)
}

pub fn decide_route_for_addrs(
    addrs: &[SocketAddr],
    tunnel_addrs: &InterfaceAddresses,
    db: &GeoIpDatabase,
) -> RoutingDecision {
    if addrs.iter().any(|sa| db.is_excluded(sa.ip())) {
        return RoutingDecision::DefaultInterface;
    }
    let tunnel_can_reach_any = addrs.iter().any(|sa| match sa.ip() {
        IpAddr::V4(_) => tunnel_addrs.v4_addr.is_some(),
        IpAddr::V6(_) => tunnel_addrs.v6_addr.is_some(),
    });
    if tunnel_can_reach_any {
        RoutingDecision::VpnTunnelInterface
    } else {
        RoutingDecision::Reject
    }
}

pub(crate) async fn decompress_gz(gz_bytes: &[u8]) -> anyhow::Result<String> {
    let mut decoder = GzipDecoder::new(Cursor::new(gz_bytes));
    let mut out = Vec::new();
    decoder
        .read_to_end(&mut out)
        .await
        .context("Gzip decompression failed")?;
    String::from_utf8(out).context("Decompressed data is not valid UTF-8")
}
