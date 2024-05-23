use tauri::Manager;
use tracing::{debug, error, info, instrument};

use crate::error::BkdError;

#[instrument(skip_all)]
#[tauri::command]
pub fn show_main_window(window: tauri::Window) -> Result<(), BkdError> {
    debug!("show_window");
    let main_window = window
        .get_window("main")
        .ok_or(BkdError::new_internal("Failed to get the app window", None))?;
    let is_visible = main_window.is_visible().map_err(|e| {
        error!("Failed to get `main` window visibility: {}", e);
        BkdError::new_internal("Failed to show app window", None)
    })?;

    if is_visible {
        debug!("`main` window is already visible");
        return Ok(());
    }

    info!("showing `main` window");
    main_window.show().map_err(|e| {
        error!("Failed to show `main` window: {}", e);
        BkdError::new_internal("Failed to show app window", None)
    })?;
    Ok(())
}
