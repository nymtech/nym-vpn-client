use crate::error::BackendError;
use crate::tray::TrayManager;
use tauri::State;
use tracing::instrument;

#[instrument(skip_all)]
#[tauri::command]
pub async fn update_tray_mode(
    tray: State<'_, TrayManager>,
    mode: String,
) -> Result<(), BackendError> {
    tray.update_tray_mode(mode).await;
    Ok(())
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn update_tray_state(
    tray: State<'_, TrayManager>,
    state: String,
) -> Result<(), BackendError> {
    tray.update_tray_state(state).await;
    Ok(())
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn update_tray_entry(
    tray: State<'_, TrayManager>,
    entry: String,
) -> Result<(), BackendError> {
    tray.update_tray_entry(entry).await;
    Ok(())
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn update_tray_exit(
    tray: State<'_, TrayManager>,
    exit: String,
) -> Result<(), BackendError> {
    tray.update_tray_exit(exit).await;
    Ok(())
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn update_tray_entry_visible(
    tray: State<'_, TrayManager>,
    visible: bool,
) -> Result<(), BackendError> {
    tray.update_tray_entry_visible(visible).await;
    Ok(())
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn update_tray_show_hide(
    tray: State<'_, TrayManager>,
    show_hide: String,
) -> Result<(), BackendError> {
    tray.update_tray_show_hide(show_hide).await;
    Ok(())
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn update_tray_quit(
    tray: State<'_, TrayManager>,
    quit: String,
) -> Result<(), BackendError> {
    tray.update_tray_quit(quit).await;
    Ok(())
}
