use crate::error::BkdError;
use crate::grpc::client::{GrpcClient, VpndStatus};
use crate::states::SharedAppState;
use std::sync::Arc;
use tauri::State;
use tracing::{debug, instrument, warn};

#[instrument(skip_all)]
#[tauri::command]
pub async fn daemon_status(
    app_state: State<'_, SharedAppState>,
    grpc_client: State<'_, Arc<GrpcClient>>,
) -> Result<VpndStatus, BkdError> {
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
