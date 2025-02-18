use tauri::State;
use tracing::{error, info, instrument, warn};

use crate::grpc::account_links::AccountLinks;
use crate::{error::BackendError, grpc::client::GrpcClient};

#[instrument(skip_all)]
#[tauri::command]
pub async fn add_account(
    mnemonic: String,
    grpc: State<'_, GrpcClient>,
) -> Result<(), BackendError> {
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
pub async fn forget_account(grpc: State<'_, GrpcClient>) -> Result<(), BackendError> {
    grpc.forget_account()
        .await
        .map_err(|e| {
            error!("failed to forget account: {}", e);
            e.into()
        })
        .inspect(|_| {
            info!("account removed successfully");
        })
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn is_account_stored(grpc: State<'_, GrpcClient>) -> Result<bool, BackendError> {
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

#[instrument(skip(grpc))]
#[tauri::command]
pub async fn account_links(
    grpc: State<'_, GrpcClient>,
    locale: String,
) -> Result<AccountLinks, BackendError> {
    grpc.account_links(&locale).await.map_err(|e| {
        error!("failed to get account link: {e}");
        e.into()
    })
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn get_account_id(grpc: State<'_, GrpcClient>) -> Result<Option<String>, BackendError> {
    grpc.account_id()
        .await
        .map_err(|e| {
            warn!("failed to get account id: {e}");
            e.into()
        })
        .inspect(|id| {
            info!("account id: {:?}", id);
        })
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn get_device_id(grpc: State<'_, GrpcClient>) -> Result<String, BackendError> {
    grpc.device_id()
        .await
        .map_err(|e| {
            warn!("failed to get device id: {e}");
            e.into()
        })
        .inspect(|id| {
            info!("device id: {id}");
        })
}
