// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Based on [v2ray](https://github.com/v2ray/v2ray-core).
//!
//! V2ray stores `(network, shift)` pairs and uses an XOR-shift membership check
//! (`(ip ^ network) >> shift == 0`).  We store `(start, end)` instead because merging
//! adjacent CIDRs can produce ranges that straddle CIDR boundaries (e.g. two adjacent /24s
//! becoming a /23-equivalent span), which cannot be expressed as a single CIDR.  Merging
//! gives us fewer entries to search through.

#[cfg(test)]
mod tests;

use std::{
    collections::HashMap,
    io::Cursor,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    path::Path,
};

use anyhow::{Context, Result};
use async_compression::tokio::bufread::GzipDecoder;
use ipnet::{Ipv4Net, Ipv6Net};
use nym_socks5_proxy_ipc::InterfaceAddresses;
use serde::Deserialize;
use tokio::io::AsyncReadExt;

static EMBEDDED_GEOIP: &[(&str, &[u8])] = &[("CN", include_bytes!("../../builtin/CN.json.gz"))];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingDecision {
    VpnTunnelInterface,
    DefaultInterface,
}

pub struct GeoIpDatabase {
    excluded_countries: HashMap<String, CountryIpSet>,
}

impl GeoIpDatabase {
    /// Load GeoIP data from disk, and if the excluded country file does not exist, use the embedded data.
    /// If that doesn't exist then an error occurs.
    pub async fn load(excluded_countries: &[String], data_dir: &Path) -> Result<Self> {
        let mut countries = HashMap::new();

        for code in excluded_countries {
            let upper = code.to_uppercase();
            match load_country_ip_set(&upper, data_dir).await {
                Ok(set) => {
                    tracing::info!(
                        country = %upper,
                        v4_ranges = set.v4.len(),
                        v6_ranges = set.v6.len(),
                        "Loaded GeoIP data for country",
                    );
                    countries.insert(upper, set);
                }
                Err(err) => {
                    tracing::warn!(
                        "Failed to load GeoIP data for {upper}: {err:#}.  This country will not be excluded."
                    );
                }
            }
        }

        Ok(Self {
            excluded_countries: countries,
        })
    }

    pub fn is_excluded(&self, ip: IpAddr) -> bool {
        self.excluded_countries.values().any(|set| set.contains(ip))
    }
}

#[derive(Deserialize)]
struct CountryGeoData {
    ipv4: Vec<String>,
    ipv6: Vec<String>,
}

#[derive(Debug, Default)]
struct Ipv4RangeSet {
    /// Network start addresses, sorted ascending.  Binary search target.
    starts: Vec<u32>,
    /// Corresponding inclusive end addresses.  Accessed only after binary search.
    ends: Vec<u32>,
}

impl Ipv4RangeSet {
    fn from_cidrs(cidrs: impl Iterator<Item = Ipv4Net>) -> Self {
        // Collect as (start, end) for sorting and merging, then split into SoA.
        let mut pairs: Vec<(u32, u32)> = cidrs
            .map(|net| (u32::from(net.network()), u32::from(net.broadcast())))
            .collect();
        pairs.sort_unstable_by_key(|&(s, _)| s);
        let pairs = merge_v4(pairs);
        let (starts, ends) = pairs.into_iter().unzip();
        Self { starts, ends }
    }

    /// O(log n) membership test.  Binary search runs on `starts` only.
    #[inline]
    fn contains(&self, ip: Ipv4Addr) -> bool {
        let q = u32::from(ip);
        // partition_point → first index where starts[i] > q, i.e. all entries before it
        // have starts[i] <= q.  The candidate is therefore at idx-1.
        let idx = self.starts.partition_point(|&s| s <= q);
        if idx == 0 {
            return false;
        }
        // SAFETY: idx - 1 is always a valid index because we checked idx > 0 and starts and
        // ends have the same length (built together from the same pairs iterator).
        q <= unsafe { *self.ends.get_unchecked(idx - 1) }
    }

    fn len(&self) -> usize {
        self.starts.len()
    }
}

#[derive(Debug, Default)]
struct Ipv6RangeSet {
    starts: Vec<u128>,
    ends: Vec<u128>,
}

impl Ipv6RangeSet {
    fn from_cidrs(cidrs: impl Iterator<Item = Ipv6Net>) -> Self {
        let mut pairs: Vec<(u128, u128)> = cidrs
            .map(|net| (u128::from(net.network()), u128::from(net.broadcast())))
            .collect();
        pairs.sort_unstable_by_key(|&(s, _)| s);
        let pairs = merge_v6(pairs);
        let (starts, ends) = pairs.into_iter().unzip();
        Self { starts, ends }
    }

    #[inline]
    fn contains(&self, ip: Ipv6Addr) -> bool {
        let q = u128::from(ip);
        let idx = self.starts.partition_point(|&s| s <= q);
        if idx == 0 {
            return false;
        }
        q <= unsafe { *self.ends.get_unchecked(idx - 1) }
    }

    fn len(&self) -> usize {
        self.starts.len()
    }
}

fn merge_v4(sorted: Vec<(u32, u32)>) -> Vec<(u32, u32)> {
    let mut out: Vec<(u32, u32)> = Vec::with_capacity(sorted.len());
    for (start, end) in sorted {
        match out.last_mut() {
            Some(last) if start <= last.1.saturating_add(1) => last.1 = last.1.max(end),
            _ => out.push((start, end)),
        }
    }
    out
}

fn merge_v6(sorted: Vec<(u128, u128)>) -> Vec<(u128, u128)> {
    let mut out: Vec<(u128, u128)> = Vec::with_capacity(sorted.len());
    for (start, end) in sorted {
        match out.last_mut() {
            Some(last) if start <= last.1.saturating_add(1) => last.1 = last.1.max(end),
            _ => out.push((start, end)),
        }
    }
    out
}

#[derive(Debug, Default)]
struct CountryIpSet {
    v4: Ipv4RangeSet,
    v6: Ipv6RangeSet,
}

impl CountryIpSet {
    fn contains(&self, ip: IpAddr) -> bool {
        match ip {
            IpAddr::V4(v4) => self.v4.contains(v4),
            IpAddr::V6(v6) => self.v6.contains(v6),
        }
    }
}

async fn load_country_ip_set(country_code: &str, data_dir: &Path) -> Result<CountryIpSet> {
    let geo_data = load_country_geo_data(country_code, data_dir).await?;

    let v4 = parse_ipv4_cidrs(&geo_data.ipv4)
        .with_context(|| format!("Failed to parse IPv4 CIDRs for {country_code}"))?;
    let v6 = parse_ipv6_cidrs(&geo_data.ipv6)
        .with_context(|| format!("Failed to parse IPv6 CIDRs for {country_code}"))?;

    Ok(CountryIpSet { v4, v6 })
}

async fn load_country_geo_data(country_code: &str, data_dir: &Path) -> Result<CountryGeoData> {
    let path = data_dir
        .join("geoip")
        .join(format!("{country_code}.json.gz"));

    let gz_bytes: Vec<u8> = match std::fs::read(&path) {
        Ok(bytes) => {
            tracing::debug!("Loaded updated GeoIP data from '{}'", path.display());
            bytes
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => embedded_geoip_gz(country_code)
            .map(|b| b.to_vec())
            .ok_or_else(|| anyhow::anyhow!("No GeoIP data available for country {country_code}"))?,
        Err(err) => {
            tracing::warn!(
                "Could not read GeoIP from '{}': {err}; trying embedded data",
                path.display(),
            );
            embedded_geoip_gz(country_code)
                .map(|b| b.to_vec())
                .ok_or_else(|| {
                    anyhow::anyhow!("No embedded GeoIP data available for country {country_code}")
                })?
        }
    };

    let json = decompress(&gz_bytes)
        .await
        .context("Failed to decompress GeoIP data")?;
    serde_json::from_str(&json)
        .with_context(|| format!("Failed to parse GeoIP JSON for {country_code}"))
}

fn embedded_geoip_gz(country_code: &str) -> Option<&'static [u8]> {
    EMBEDDED_GEOIP
        .iter()
        .find(|&&(cc, _)| cc == country_code)
        .map(|&(_, data)| data)
}

async fn decompress(gz_bytes: &[u8]) -> Result<String> {
    let mut decoder = GzipDecoder::new(Cursor::new(gz_bytes));
    let mut out = Vec::new();
    decoder
        .read_to_end(&mut out)
        .await
        .context("Gzip decompression failed")?;
    String::from_utf8(out).context("Decompressed GeoIP data is not valid UTF-8")
}

/// Make a routing decision by checking ALL resolved socket addresses.
/// If any address is in an excluded country, the whole connection bypasses the tunnel.
/// If none are excluded but the tunnel can reach at least one address family, use the tunnel.
pub fn decide_route(
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
        RoutingDecision::DefaultInterface
    }
}

fn parse_ipv4_cidrs(cidrs: &[String]) -> Result<Ipv4RangeSet> {
    let nets: Result<Vec<Ipv4Net>, _> = cidrs.iter().map(|s| s.trim().parse::<Ipv4Net>()).collect();
    let nets = nets.context("Invalid IPv4 CIDR entry")?;
    Ok(Ipv4RangeSet::from_cidrs(nets.into_iter()))
}

fn parse_ipv6_cidrs(cidrs: &[String]) -> Result<Ipv6RangeSet> {
    let nets: Result<Vec<Ipv6Net>, _> = cidrs.iter().map(|s| s.trim().parse::<Ipv6Net>()).collect();
    let nets = nets.context("Invalid IPv6 CIDR entry")?;
    Ok(Ipv6RangeSet::from_cidrs(nets.into_iter()))
}
