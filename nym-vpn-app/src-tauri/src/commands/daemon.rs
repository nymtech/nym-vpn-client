use crate::env::NETWORK_ENV_SELECT;
use crate::error::BackendError;
use crate::grpc::client::{FeatureFlags, GrpcClient, SystemMessage, VpndStatus};
use crate::states::SharedAppState;
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{debug, info, instrument, warn};
use ts_rs::TS;

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub struct DaemonInfo {
    version: String,
    network: String,
}

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
    grpc_client: State<'_, GrpcClient>,
) -> Result<VpndStatus, BackendError> {
    debug!("daemon_status");
    let status = grpc_client
        .check(app_state.inner())
        .await
        .inspect_err(|e| {
            warn!("failed to check daemon status: {:?}", e);
        })
        .unwrap_or(VpndStatus::NotOk);
    debug!("daemon status: {:?}", status);
    Ok(status)
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn daemon_info(grpc_client: State<'_, GrpcClient>) -> Result<DaemonInfo, BackendError> {
    debug!("daemon_info");
    let res = grpc_client.vpnd_info().await.inspect_err(|e| {
        warn!("failed to get daemon info: {:?}", e);
    })?;

    let network = res
        .nym_network
        .map(|network| network.network_name)
        .ok_or_else(|| BackendError::new_internal("missing network details", None))
        .inspect_err(|e| {
            warn!("daemon info response missing network details: {:?}", e);
        })?;

    Ok(DaemonInfo {
        version: res.version,
        network,
    })
}

#[instrument(skip(grpc_client))]
#[tauri::command]
pub async fn set_network(
    grpc_client: State<'_, GrpcClient>,
    network: NetworkEnv,
) -> Result<(), BackendError> {
    debug!("set_network");
    if !*NETWORK_ENV_SELECT {
        warn!("network env selector is disabled");
        return Err(BackendError::new_internal("nope", None));
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
    debug!("system_messages");
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
    debug!("feature_flags");
    grpc_client
        .feature_flags()
        .await
        .inspect_err(|e| {
            warn!("failed to get feature flags: {:?}", e);
        })
        .map_err(|e| e.into())
}
