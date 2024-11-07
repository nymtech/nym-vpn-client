use serde_json::Value as JsonValue;
use tauri::State;
use tracing::{debug, error, info, instrument};

use crate::{error::BackendError, grpc::client::GrpcClient};

#[instrument(skip_all)]
#[tauri::command]
pub async fn add_account(
    mnemonic: String,
    grpc: State<'_, GrpcClient>,
) -> Result<(), BackendError> {
    debug!("add_account");
    grpc.store_account(mnemonic)
        .await
        .map_err(|e| {
            error!("failed to add account: {}", e);
            e.into()
        })
        .inspect(|_| {
            info!("account added successfully");
        })
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn delete_account(grpc: State<'_, GrpcClient>) -> Result<(), BackendError> {
    debug!("delete_account");
    grpc.remove_account()
        .await
        .map_err(|e| {
            error!("failed to remove account: {}", e);
            e.into()
        })
        .inspect(|_| {
            info!("account removed successfully");
        })
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn is_account_stored(grpc: State<'_, GrpcClient>) -> Result<bool, BackendError> {
    debug!("is_account_stored");

    grpc.is_account_stored()
        .await
        .map_err(|e| {
            error!("failed to check stored account: {e}");
            e.into()
        })
        .inspect(|stored| {
            info!("account stored: {stored}");
        })
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn get_account_info(grpc: State<'_, GrpcClient>) -> Result<JsonValue, BackendError> {
    debug!("get_account_info");

    grpc.get_account_summary()
        .await
        .map_err(|e| {
            error!("failed to get account info: {e}");
            e.into()
        })
        .map(|s| {
            info!("account info: {}", s);
            s.parse::<JsonValue>()
                .inspect_err(|e| error!("failed to parse json value: {e}"))
                .unwrap_or(JsonValue::Null)
        })
}
