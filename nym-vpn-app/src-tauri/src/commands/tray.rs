use crate::{
  error::{BackendError, ErrorKey},
};
use tracing::{debug, info, instrument, warn};
use tauri::{Manager, State};
use crate::tray::TrayManager;

#[instrument(skip_all)]
#[tauri::command]
pub async fn update_tray_mode(tray: State<'_, TrayManager>, mode: String) -> Result<(), BackendError> {
    tray.update_tray_mode(mode).await;
    Ok(())
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn update_tray_state(tray: State<'_, TrayManager>, state: String) -> Result<(), BackendError> {
    tray.update_tray_state(state).await;
    Ok(())
}