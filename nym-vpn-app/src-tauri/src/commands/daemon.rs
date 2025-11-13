use crate::env::DEV_MODE;
use crate::error::BackendError;
use crate::state::SharedAppState;
use crate::state::app::NetworkCompat;
use crate::vpnd::client::{FeatureFlags, SystemMessage, VpndClient, VpndStatus};
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{debug, info, instrument, warn};
use ts_rs::TS;

#[cfg(unix)]
const DEFAULT_VPND_LOG_DIR: &str = "/var/log/nym-vpnd";
#[cfg(windows)]
const DEFAULT_VPND_LOG_DIR: &str = "C:\\ProgramData\\nym-vpnd\\log";

#[derive(strum::AsRefStr, Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
#[ts(export, export_to = "tauri.ts")]
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

#[instrument(skip(vpnd))]
#[tauri::command]
pub async fn set_network(
    vpnd: State<'_, VpndClient>,
    network: NetworkEnv,
) -> Result<(), BackendError> {
    if !*DEV_MODE {
        warn!("not in dev mode");
        return Err(BackendError::internal("nope", None));
    }
    vpnd.set_network(network.as_ref())
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
    vpnd: State<'_, VpndClient>,
) -> Result<Vec<SystemMessage>, BackendError> {
    vpnd.system_messages()
        .await
        .inspect_err(|e| {
            warn!("failed to get system messages: {:?}", e);
        })
        .map_err(|e| e.into())
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn feature_flags(vpnd: State<'_, VpndClient>) -> Result<FeatureFlags, BackendError> {
    vpnd.feature_flags()
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

#[instrument(skip_all)]
#[tauri::command]
pub async fn vpnd_log_dir(
    app_state: State<'_, SharedAppState>,
    vpnd: State<'_, VpndClient>,
) -> Result<String, BackendError> {
    let state = app_state.lock().await;
    if state.vpnd_status == VpndStatus::Down {
        warn!("vpnd is down, fallback to default log dir");
        return Ok(DEFAULT_VPND_LOG_DIR.to_string());
    }

    let path = vpnd.vpnd_log_path().await?.to_string_lossy().to_string();
    Ok(path)
}
