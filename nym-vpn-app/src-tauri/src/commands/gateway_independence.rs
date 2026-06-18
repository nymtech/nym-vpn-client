use tauri::State;
use tracing::instrument;

use crate::{
    error::BackendError,
    vpnd::{client::VpndClient, tentative_gateways::TentativeGateways},
};

#[instrument(skip(vpnd))]
#[tauri::command]
pub async fn get_tentative_gateways(
    vpnd: State<'_, VpndClient>,
) -> Result<TentativeGateways, BackendError> {
    let tentative = vpnd.get_tentative_gateways().await?;
    Ok(tentative)
}

#[instrument(skip(vpnd))]
#[tauri::command]
pub async fn set_gateway_independence(
    vpnd: State<'_, VpndClient>,
    enabled: bool,
) -> Result<(), BackendError> {
    vpnd.set_gateway_independence(enabled).await?;
    Ok(())
}

#[instrument(skip(vpnd))]
#[tauri::command]
pub async fn set_gateway_independence_notifications(
    vpnd: State<'_, VpndClient>,
    enabled: bool,
) -> Result<(), BackendError> {
    vpnd.set_gateway_independence_notifications(enabled).await?;
    Ok(())
}
