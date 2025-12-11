use tauri::State;
use tracing::{info, instrument};

use crate::error::BackendError;
use crate::vpnd::client::Node;
use crate::vpnd::client::VpndClient;
use crate::vpnd::socks5::{HttpRpcSettings, Socks5Settings, Socks5Status};

#[instrument(skip_all)]
#[tauri::command]
pub async fn enable_socks5(
    vpnd: State<'_, VpndClient>,
    socks5_settings: Socks5Settings,
    http_rpc_settings: HttpRpcSettings,
    exit: Node,
) -> Result<(), BackendError> {
    info!("Enabling SOCKS5 proxy with exit_node: {exit}");
    vpnd.enable_socks5(socks5_settings, http_rpc_settings, exit)
        .await?;
    Ok(())
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn disable_socks5(vpnd: State<'_, VpndClient>) -> Result<(), BackendError> {
    info!("Disabling SOCKS5 proxy");
    vpnd.disable_socks5().await?;
    Ok(())
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn get_socks5_status(vpnd: State<'_, VpndClient>) -> Result<Socks5Status, BackendError> {
    let status = vpnd.get_socks5_status().await?;
    Ok(status)
}
