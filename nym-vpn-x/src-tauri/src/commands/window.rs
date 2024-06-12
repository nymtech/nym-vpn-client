use tauri::Manager;
use tracing::{debug, error, info, instrument};

use crate::{error::BackendError, MAIN_WINDOW_LABEL};

#[instrument(skip_all)]
#[tauri::command]
pub fn show_main_window(window: tauri::Window) -> Result<(), BackendError> {
    debug!("show_window");
    let main_window = window
        .get_window(MAIN_WINDOW_LABEL)
        .ok_or(BackendError::new_internal(
            "Failed to get the app window",
            None,
        ))?;
    let is_visible = main_window.is_visible().map_err(|e| {
        error!("Failed to get `main` window visibility: {}", e);
        BackendError::new_internal("Failed to show app window", None)
    })?;

    if is_visible {
        debug!("`main` window is already visible");
        return Ok(());
    }

    info!("showing `main` window");
    main_window.show().map_err(|e| {
        error!("Failed to show `main` window: {}", e);
        BackendError::new_internal("Failed to show app window", None)
    })?;
    Ok(())
}
