use serde_json::Value as JsonValue;
use tauri::State;
use tracing::{debug, error, info, instrument, warn};

use crate::{
    error::{BackendError, ErrorKey},
    grpc::client::GrpcClient,
};

#[instrument(skip_all)]
#[tauri::command]
pub async fn add_account(
    mnemonic: String,
    grpc: State<'_, GrpcClient>,
) -> Result<(), BackendError> {
    debug!("add_account");

    let res = grpc.store_account(mnemonic).await?;
    if res.success {
        info!("account added successfully");
        Ok(())
    } else {
        warn!("failed to add account");
        let error = res
            .error
            .map(|e| e.into())
            .unwrap_or_else(|| BackendError::new("failed to add account", ErrorKey::UnknownError));
        Err(error)
    }
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn get_account_info(grpc: State<'_, GrpcClient>) -> Result<JsonValue, BackendError> {
    debug!("get_account_info");

    let res = grpc.get_account_summary().await?;
    if let Some(error) = res.error {
        Err(error.into())
    } else {
        info!("account info: {}", res.json);
        Ok(res
            .json
            .parse::<JsonValue>()
            .inspect_err(|e| error!("failed to parse json value: {e}"))
            .unwrap_or(JsonValue::Null))
    }
}
