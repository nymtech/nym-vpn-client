use tauri::window::Color;
use tauri::{AppHandle, Manager};
use tracing::{debug, error, instrument};

use crate::{error::BackendError, MAIN_WINDOW_LABEL};

#[instrument(skip_all)]
#[tauri::command]
pub fn show_main_window(app: AppHandle) -> Result<(), BackendError> {
    let main_window = app
        .get_webview_window(MAIN_WINDOW_LABEL)
        .ok_or(BackendError::internal("Failed to get the app window", None))?;
    let is_visible = main_window.is_visible().map_err(|e| {
        error!("failed to get `main` window visibility: {}", e);
        BackendError::internal("Failed to show app window", None)
    })?;

    if is_visible {
        debug!("`main` window is already visible");
        return Ok(());
    }

    debug!("showing `main` window");
    main_window.show().map_err(|e| {
        error!("failed to show `main` window: {}", e);
        BackendError::internal("Failed to show app window", None)
    })?;
    Ok(())
}

#[instrument(skip(webview))]
#[tauri::command]
pub async fn set_background_color(
    webview: tauri::WebviewWindow,
    hex_color: String,
) -> Result<(), BackendError> {
    let color: Color = hex_color.parse().map_err(|e| {
        error!("failed to parse color [{}]: {}", &hex_color, e);
        BackendError::internal("failed to parse color", None)
    })?;
    webview.set_background_color(Some(color)).map_err(|e| {
        error!("failed to get `main` window visibility: {}", e);
        BackendError::internal("failed to set webview background color", None)
    })?;
    Ok(())
}
