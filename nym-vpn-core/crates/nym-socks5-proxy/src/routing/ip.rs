// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! IP-range based exclusion.
//!
//! Stores sorted `(start, end)` pairs for IPv4 and IPv6 separately and uses
//! binary search for O(log n) membership tests.  Adjacent or overlapping
//! ranges are merged on construction so the set stays compact.

use std::{
    collections::HashMap,
    io::Cursor,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::Path,
};

use anyhow::{Context, Result};
use async_compression::tokio::bufread::GzipDecoder;
use ipnet::{Ipv4Net, Ipv6Net};
use serde::Deserialize;
use tokio::io::AsyncReadExt;

static EMBEDDED_GEOIP: &[(&str, &[u8])] = &[("CN", include_bytes!("../../builtin/CN-ip.json.gz"))];

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

pub struct GeoIpDatabase {
    pub(super) excluded_countries: HashMap<String, CountryIpSet>,
}

impl GeoIpDatabase {
    /// Load GeoIP data for each excluded country code.  Falls back to embedded
    /// data when no on-disk file is found.
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

    /// Returns `true` if `ip` falls within any excluded-country range.
    pub fn is_excluded(&self, ip: IpAddr) -> bool {
        self.excluded_countries.values().any(|set| set.contains(ip))
    }
}

// ---------------------------------------------------------------------------
// Internal range sets (pub(super) so tests.rs can reach them)
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
pub(super) struct Ipv4RangeSet {
    /// Network start addresses, sorted ascending.  Binary search target.
    pub(super) starts: Vec<u32>,
    /// Corresponding inclusive end addresses.
    pub(super) ends: Vec<u32>,
}

impl Ipv4RangeSet {
    pub(super) fn from_cidrs(cidrs: impl Iterator<Item = Ipv4Net>) -> Self {
        let mut pairs: Vec<(u32, u32)> = cidrs
            .map(|net| (u32::from(net.network()), u32::from(net.broadcast())))
            .collect();
        pairs.sort_unstable_by_key(|&(s, _)| s);
        let pairs = merge_v4(pairs);
        let (starts, ends) = pairs.into_iter().unzip();
        Self { starts, ends }
    }

    #[inline]
    pub(super) fn contains(&self, ip: Ipv4Addr) -> bool {
        let q = u32::from(ip);
        let idx = self.starts.partition_point(|&s| s <= q);
        if idx == 0 {
            return false;
        }
        q <= unsafe { *self.ends.get_unchecked(idx - 1) }
    }

    pub(super) fn len(&self) -> usize {
        self.starts.len()
    }
}

#[derive(Debug, Default)]
pub(super) struct Ipv6RangeSet {
    pub(super) starts: Vec<u128>,
    pub(super) ends: Vec<u128>,
}

impl Ipv6RangeSet {
    pub(super) fn from_cidrs(cidrs: impl Iterator<Item = Ipv6Net>) -> Self {
        let mut pairs: Vec<(u128, u128)> = cidrs
            .map(|net| (u128::from(net.network()), u128::from(net.broadcast())))
            .collect();
        pairs.sort_unstable_by_key(|&(s, _)| s);
        let pairs = merge_v6(pairs);
        let (starts, ends) = pairs.into_iter().unzip();
        Self { starts, ends }
    }

    #[inline]
    pub(super) fn contains(&self, ip: Ipv6Addr) -> bool {
        let q = u128::from(ip);
        let idx = self.starts.partition_point(|&s| s <= q);
        if idx == 0 {
            return false;
        }
        q <= unsafe { *self.ends.get_unchecked(idx - 1) }
    }

    pub(super) fn len(&self) -> usize {
        self.starts.len()
    }
}

#[derive(Debug, Default)]
pub(super) struct CountryIpSet {
    pub(super) v4: Ipv4RangeSet,
    pub(super) v6: Ipv6RangeSet,
}

impl CountryIpSet {
    pub(super) fn contains(&self, ip: IpAddr) -> bool {
        match ip {
            IpAddr::V4(v4) => self.v4.contains(v4),
            IpAddr::V6(v6) => self.v6.contains(v6),
        }
    }
}

// ---------------------------------------------------------------------------
// Merge helpers
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Loading
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub(super) struct CountryGeoData {
    pub(super) ipv4: Vec<String>,
    pub(super) ipv6: Vec<String>,
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

    let json = decompress_gz(&gz_bytes)
        .await
        .context("Failed to decompress GeoIP data")?;
    serde_json::from_str(&json)
        .with_context(|| format!("Failed to parse GeoIP JSON for {country_code}"))
}

pub(super) fn embedded_geoip_gz(country_code: &str) -> Option<&'static [u8]> {
    EMBEDDED_GEOIP
        .iter()
        .find(|&&(cc, _)| cc == country_code)
        .map(|&(_, data)| data)
}

pub(super) async fn decompress_gz(gz_bytes: &[u8]) -> Result<String> {
    let mut decoder = GzipDecoder::new(Cursor::new(gz_bytes));
    let mut out = Vec::new();
    decoder
        .read_to_end(&mut out)
        .await
        .context("Gzip decompression failed")?;
    String::from_utf8(out).context("Decompressed data is not valid UTF-8")
}

pub(super) fn parse_ipv4_cidrs(cidrs: &[String]) -> Result<Ipv4RangeSet> {
    let nets: Result<Vec<Ipv4Net>, _> = cidrs.iter().map(|s| s.trim().parse::<Ipv4Net>()).collect();
    let nets = nets.context("Invalid IPv4 CIDR entry")?;
    Ok(Ipv4RangeSet::from_cidrs(nets.into_iter()))
}

pub(super) fn parse_ipv6_cidrs(cidrs: &[String]) -> Result<Ipv6RangeSet> {
    let nets: Result<Vec<Ipv6Net>, _> = cidrs.iter().map(|s| s.trim().parse::<Ipv6Net>()).collect();
    let nets = nets.context("Invalid IPv6 CIDR entry")?;
    Ok(Ipv6RangeSet::from_cidrs(nets.into_iter()))
}
