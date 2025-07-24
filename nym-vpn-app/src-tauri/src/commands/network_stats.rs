use crate::error::BackendError;
use crate::grpc::client::GrpcClient;

use tauri::State;
use tracing::instrument;

#[instrument(skip_all)]
#[tauri::command]
pub async fn enable_netstats(grpc: State<'_, GrpcClient>) -> Result<(), BackendError> {
    grpc.enable_netstats().await?;
    Ok(())
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn disable_netstats(grpc: State<'_, GrpcClient>) -> Result<(), BackendError> {
    grpc.disable_netstats().await?;
    Ok(())
}
