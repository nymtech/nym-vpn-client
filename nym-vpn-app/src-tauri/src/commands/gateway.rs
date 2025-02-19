use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;
use tracing::{debug, instrument};
use ts_rs::TS;

use crate::country::Country;
use crate::grpc::client::GrpcClient;
use crate::grpc::gateway::{Gateway, GatewayType};
use crate::{
    error::{BackendError, ErrorKey},
    states::app::VpnMode,
};

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
        .collect()
}

#[instrument(skip(grpc))]
#[tauri::command]
pub async fn get_gateways(
    vpn_mode: VpnMode,
    node_type: Option<NodeType>,
    grpc: State<'_, GrpcClient>,
) -> Result<Vec<GatewaysByCountry>, BackendError> {
    let gw_type = match vpn_mode {
        VpnMode::Mixnet => match node_type.ok_or_else(|| {
            BackendError::internal("node type must be provided for Mixnet mode", None)
        })? {
            NodeType::Entry => GatewayType::MxEntry,
            NodeType::Exit => GatewayType::MxExit,
        },
        VpnMode::TwoHop => GatewayType::Wg,
    };
    let gateways = grpc.gateways(gw_type).await.map_err(|e| {
        BackendError::with_details(
            &format!("failed to get gateways for {}", gw_type),
            ErrorKey::from(gw_type),
            e.to_string(),
        )
    });
    gateways
        .map(|gws| group_by_country(gws, gw_type))
        .inspect(|list| {
            // TODO remove this
            debug!("gateways by country {:#?}", list);
        })
}
