// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::Path,
};

use anyhow::{Context, Result};
use ipnet::{Ipv4Net, Ipv6Net};
use iprange::IpRange;
use serde::Deserialize;

static EMBEDDED_GEOIP: &[(&str, &[u8])] = &[("CN", include_bytes!("../../builtin/CN-ip.json.gz"))];

pub struct GeoIpDatabase {
    pub(super) excluded_countries: HashMap<String, CountryIpSet>,
}

impl GeoIpDatabase {
    pub async fn load(excluded_countries: &[String], data_dir: &Path) -> Result<Self> {
        let mut countries = HashMap::new();

        for code in excluded_countries {
            let upper = code.to_uppercase();
            match load_country_ip_set(&upper, data_dir).await {
                Ok(set) => {
                    let (v4_ranges, v6_ranges) = set.len();
                    tracing::info!(
                        country = %upper,
                        v4_ranges,
                        v6_ranges,
                        "Loaded GeoIP data for country",
                    );
                    countries.insert(upper, set);
                }
                Err(err) => {
                    tracing::warn!(
                        "Failed to load GeoIP data for {upper}: {err:#}.  \
                         This country will not be excluded."
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

#[derive(Default)]
pub(super) struct CountryIpSet {
    pub(super) v4: IpRange<Ipv4Net>,
    pub(super) v6: IpRange<Ipv6Net>,
}

impl CountryIpSet {
    pub(super) fn contains(&self, ip: IpAddr) -> bool {
        match ip {
            IpAddr::V4(v4) => self.v4.contains(&host_net_v4(v4)),
            IpAddr::V6(v6) => self.v6.contains(&host_net_v6(v6)),
        }
    }

    pub(super) fn len(&self) -> (usize, usize) {
        (self.v4.iter().count(), self.v6.iter().count())
    }
}

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

    let json = super::decompress_gz(&gz_bytes)
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

pub(super) fn parse_ipv4_cidrs(cidrs: &[String]) -> Result<IpRange<Ipv4Net>> {
    let mut range = IpRange::new();
    for s in cidrs {
        let net: Ipv4Net = s.trim().parse().context("Invalid IPv4 CIDR entry")?;
        range.add(net);
    }
    Ok(range)
}

pub(super) fn parse_ipv6_cidrs(cidrs: &[String]) -> Result<IpRange<Ipv6Net>> {
    let mut range = IpRange::new();
    for s in cidrs {
        let net: Ipv6Net = s.trim().parse().context("Invalid IPv6 CIDR entry")?;
        range.add(net);
    }
    Ok(range)
}

#[inline]
fn host_net_v4(ip: Ipv4Addr) -> Ipv4Net {
    Ipv4Net::new(ip, 32).expect("32 is a valid IPv4 prefix length")
}

#[inline]
fn host_net_v6(ip: Ipv6Addr) -> Ipv6Net {
    Ipv6Net::new(ip, 128).expect("128 is a valid IPv6 prefix length")
}
