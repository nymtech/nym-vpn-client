use nym_vpn_proto::GatewayType;
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{debug, instrument};
use ts_rs::TS;

use crate::grpc::client::GrpcClient;
use crate::{
    country::Country,
    error::{BackendError, ErrorKey},
    states::app::VpnMode,
};

#[derive(Debug, Serialize, Deserialize, TS, Clone)]
pub enum NodeType {
    Entry,
    Exit,
}

#[instrument(skip(grpc))]
#[tauri::command]
pub async fn get_countries(
    vpn_mode: VpnMode,
    node_type: Option<NodeType>,
    grpc: State<'_, GrpcClient>,
) -> Result<Vec<Country>, BackendError> {
    debug!("get_countries");
    let gw_type = match vpn_mode {
        VpnMode::Mixnet => match node_type.ok_or_else(|| {
            BackendError::new_internal("node type must be provided for Mixnet mode", None)
        })? {
            NodeType::Entry => GatewayType::MixnetEntry,
            NodeType::Exit => GatewayType::MixnetExit,
        },
        VpnMode::TwoHop => GatewayType::Wg,
    };
    grpc.countries(gw_type).await.map_err(|e| {
        BackendError::new_with_details(
            &format!("failed to get countries for {:?}", gw_type),
            ErrorKey::from(gw_type),
            e.to_string(),
        )
    })
}
