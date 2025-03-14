use crate::env::DEV_MODE;
use crate::error::BackendError;
use crate::grpc::client::{FeatureFlags, GrpcClient, SystemMessage, VpndStatus};
use crate::state::app::NetworkCompat;
use crate::state::SharedAppState;
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{debug, info, instrument, warn};
use ts_rs::TS;

#[derive(strum::AsRefStr, Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
#[ts(export)]
pub enum NetworkEnv {
    Mainnet,
    Canary,
    QA,
    Sandbox,
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn daemon_status(
    app_state: State<'_, SharedAppState>,
) -> Result<VpndStatus, BackendError> {
    let status = app_state.lock().await.vpnd_status.clone();
    debug!("daemon status: {:?}", status);
    Ok(status)
}

#[instrument(skip(grpc_client))]
#[tauri::command]
pub async fn set_network(
    grpc_client: State<'_, GrpcClient>,
    network: NetworkEnv,
) -> Result<(), BackendError> {
    if !*DEV_MODE {
        warn!("not in dev mode");
        return Err(BackendError::internal("nope", None));
    }
    grpc_client
        .set_network(network.as_ref())
        .await
        .map_err(|e| {
            warn!("failed to set network {}: {:?}", network.as_ref(), e);
            e.into()
        })
        .inspect(|_| {
            info!("vpnd network set to {} ⚠ restart vpnd!", network.as_ref());
        })
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn system_messages(
    grpc_client: State<'_, GrpcClient>,
) -> Result<Vec<SystemMessage>, BackendError> {
    grpc_client
        .system_messages()
        .await
        .inspect_err(|e| {
            warn!("failed to get system messages: {:?}", e);
        })
        .map_err(|e| e.into())
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn feature_flags(
    grpc_client: State<'_, GrpcClient>,
) -> Result<FeatureFlags, BackendError> {
    grpc_client
        .feature_flags()
        .await
        .inspect_err(|e| {
            warn!("failed to get feature flags: {:?}", e);
        })
        .map_err(|e| e.into())
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn network_compat(
    app_state: State<'_, SharedAppState>,
) -> Result<Option<NetworkCompat>, BackendError> {
    Ok(app_state.lock().await.network_compat.clone())
}
