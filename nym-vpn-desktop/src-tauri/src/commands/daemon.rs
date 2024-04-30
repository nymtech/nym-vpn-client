use crate::error::CmdError;
use crate::grpc::client::{GrpcClient, VpndStatus};
use std::sync::Arc;
use tauri::State;
use tracing::{debug, instrument};

#[instrument(skip_all)]
#[tauri::command]
pub async fn vpnd_check(grpc_client: State<'_, Arc<GrpcClient>>) -> Result<VpndStatus, CmdError> {
    debug!("vpnd_check");

    Ok(grpc_client.status())
}
