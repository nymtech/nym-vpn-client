use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use tauri::State;
use tracing::{debug, info, instrument, trace};
use ts_rs::TS;

use crate::country::Country;
use crate::error::{BackendError, ErrorKey};
use crate::state::app::VpnMode;
use crate::vpnd::client::VpndClient;
use crate::vpnd::gateway::{Gateway, GatewayType, RecentGateways};

#[derive(Debug, Serialize, Deserialize, TS, Clone)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "kebab-case")]
pub enum Hop {
    Entry,
    Exit,
}

#[derive(Debug, Serialize, Deserialize, TS, Clone)]
#[ts(export, export_to = "tauri.ts")]
pub struct Region {
    pub name: String,
    pub country: Country,
    pub gateways: Vec<Gateway>,
    #[serde(rename = "type")]
    pub kind: GatewayType,
    // whether there is at least 1 quic compatible gateway
    pub quic: bool,
}

#[derive(Debug, Serialize, Deserialize, TS, Clone)]
#[ts(export, export_to = "tauri.ts")]
pub struct GatewaysByCountry {
    pub country: Country,
    pub regions: Vec<Region>,
    pub gateways: Vec<Gateway>,
    #[serde(rename = "type")]
    pub kind: GatewayType,
    // whether there is at least 1 quic compatible gateway
    pub quic: bool,
}

fn group_by_region(
    countries: Vec<GatewaysByCountry>,
    gw_type: GatewayType,
) -> Vec<GatewaysByCountry> {
    countries
        .into_iter()
        .map(|mut country| {
            let by_region = country
                .gateways
                .clone()
                .into_iter()
                .fold(HashMap::<String, Region>::new(), |mut map, gateway| {
                    let region = &gateway.location.region;
                    let is_quic = gateway.quic;
                    if let Some(r) = map.get_mut(region) {
                        r.gateways.push(gateway);
                        if is_quic {
                            r.quic = true;
                        }
                    } else {
                        map.insert(
                            region.clone(),
                            Region {
                                name: region.to_owned(),
                                country: country.country.clone(),
                                gateways: vec![gateway],
                                kind: gw_type,
                                quic: is_quic,
                            },
                        );
                    }
                    map
                })
                .into_values()
                .sorted_by_key(|region| region.name.clone())
                .map(|mut region| {
                    sort_by_perf(&mut region.gateways);
                    region
                })
                .collect();
            country.regions = by_region;
            country
        })
        .collect()
}

fn group_by_country(gateways: Vec<Gateway>, gw_type: GatewayType) -> Vec<GatewaysByCountry> {
    gateways
        .into_iter()
        .fold(
            HashMap::<String, GatewaysByCountry>::new(),
            |mut map, gateway| {
                let country_code = &gateway.country.code;
                let is_quic = gateway.quic;
                if let Some(c) = map.get_mut(country_code) {
                    c.gateways.push(gateway);
                    if is_quic {
                        c.quic = true;
                    }
                } else {
                    map.insert(
                        country_code.clone(),
                        GatewaysByCountry {
                            country: gateway.country.clone(),
                            regions: vec![],
                            gateways: vec![gateway],
                            kind: gw_type,
                            quic: is_quic,
                        },
                    );
                }
                map
            },
        )
        .into_values()
        .sorted_by_key(|g| g.country.name.clone())
        .collect()
}

fn sort_by_perf(gateways: &mut [Gateway]) {
    gateways.sort_by(|a, b| match a.kind {
        GatewayType::Wg => a.wg_score.cmp(&b.wg_score).reverse(),
        _ => a.mx_score.cmp(&b.mx_score).reverse(),
    });
}

fn sort_countries_gw(mut gw_by_countries: Vec<GatewaysByCountry>) -> Vec<GatewaysByCountry> {
    for country in gw_by_countries.iter_mut() {
        sort_by_perf(&mut country.gateways);
    }
    gw_by_countries
}

#[instrument(skip(vpnd))]
#[tauri::command]
pub async fn get_gateways(
    node_type: GatewayType,
    vpnd: State<'_, VpndClient>,
) -> Result<Vec<GatewaysByCountry>, BackendError> {
    info!("fetching gateways");
    let gateways = vpnd
        .gateways(node_type)
        .await
        .map_err(|e| {
            BackendError::with_detail(
                &format!("failed to get gateways for {node_type}"),
                ErrorKey::from(node_type),
                e.to_string(),
            )
        })
        .inspect(|gateways| {
            info!("gateways #{}", gateways.len());
        });

    gateways
        .map(|gws| group_by_country(gws, node_type))
        .map(|countries| group_by_region(countries, node_type))
        .map(sort_countries_gw)
        .inspect(|list| {
            debug!("countries #{}", list.len());
            for country in list {
                trace!("{}", country);
                country
                    .regions
                    .iter()
                    .for_each(|region| trace!("{}", region));
            }
        })
}

#[instrument(skip(vpnd))]
#[tauri::command]
pub async fn get_recent_gateways(
    vpn_mode: VpnMode,
    vpnd: State<'_, VpndClient>,
) -> Result<RecentGateways, BackendError> {
    info!("fetching recent gateways");
    vpnd.recent_gateways(&vpn_mode)
        .await
        .map_err(|e| {
            BackendError::with_detail(
                &format!("failed to get recent gateways for {vpn_mode}"),
                ErrorKey::Internal,
                e.to_string(),
            )
        })
        .inspect(|recents| {
            info!(
                "recent gateways: entry #{}, exit #{}",
                recents.entry.len(),
                recents.exit.len()
            );
        })
}

impl fmt::Display for GatewaysByCountry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {}: gws #{}, regions #{}",
            self.country.code,
            self.country.name,
            self.gateways.len(),
            self.regions.len()
        )
    }
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({}) {}: gws #{}",
            self.country.code,
            self.name,
            self.gateways.len()
        )
    }
}
