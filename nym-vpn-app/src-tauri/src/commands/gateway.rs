use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use tauri::State;
use tracing::{debug, info, instrument, trace};
use ts_rs::TS;

use crate::country::Country;
use crate::error::{BackendError, ErrorKey};
use crate::grpc::client::GrpcClient;
use crate::grpc::gateway::{Gateway, GatewayType};

#[derive(Debug, Serialize, Deserialize, TS, Clone)]
pub enum NodeType {
    Entry,
    Exit,
}

#[derive(Debug, Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub struct GatewaysByCountry {
    pub country: Country,
    pub gateways: Vec<Gateway>,
    #[serde(rename = "type")]
    pub kind: GatewayType,
}

fn group_by_country(gateways: Vec<Gateway>, gw_type: GatewayType) -> Vec<GatewaysByCountry> {
    gateways
        .into_iter()
        .fold(
            HashMap::<String, GatewaysByCountry>::new(),
            |mut map, gateway| {
                let country_code = &gateway.country.code;
                if let Some(gw_by_country) = map.get_mut(country_code) {
                    gw_by_country.gateways.push(gateway);
                } else {
                    map.insert(
                        country_code.clone(),
                        GatewaysByCountry {
                            country: gateway.country.clone(),
                            gateways: vec![gateway],
                            kind: gw_type,
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

fn sort_by_perf(mut gw_by_countries: Vec<GatewaysByCountry>) -> Vec<GatewaysByCountry> {
    for group in gw_by_countries.iter_mut() {
        group.gateways.sort_by(|a, b| match a.kind {
            GatewayType::Wg => a.wg_score.cmp(&b.wg_score).reverse(),
            _ => a.mx_score.cmp(&b.mx_score).reverse(),
        });
    }
    gw_by_countries
}

#[instrument(skip(grpc))]
#[tauri::command]
pub async fn get_gateways(
    node_type: GatewayType,
    grpc: State<'_, GrpcClient>,
) -> Result<Vec<GatewaysByCountry>, BackendError> {
    info!("fetching gateways");
    let gateways = grpc
        .gateways(node_type)
        .await
        .map_err(|e| {
            BackendError::with_details(
                &format!("failed to get gateways for {}", node_type),
                ErrorKey::from(node_type),
                e.to_string(),
            )
        })
        .inspect(|gateways| {
            info!("gateways #{}", gateways.len());
        });

    gateways
        .map(|gws| group_by_country(gws, node_type))
        .map(sort_by_perf)
        .inspect(|list| {
            debug!("countries #{}", list.len());
            for gateways in list {
                trace!("{}", gateways);
                gateways.gateways.iter().for_each(|gw| match gw.kind {
                    GatewayType::Wg => trace!("wg {:?}", gw.wg_score),
                    _ => trace!("mx {:?}", gw.mx_score),
                });
            }
        })
}

impl fmt::Display for GatewaysByCountry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {}: #{}",
            self.country.code,
            self.country.name,
            self.gateways.len()
        )
    }
}
