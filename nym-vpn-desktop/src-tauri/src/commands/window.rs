use tauri::Manager;
use tracing::{debug, error, info, instrument};

use crate::error::{CmdError, CmdErrorSource};

#[instrument(skip_all)]
#[tauri::command]
pub fn show_main_window(window: tauri::Window) -> Result<(), CmdError> {
    debug!("show_window");
    let main_window = window.get_window("main").unwrap();
    let is_visible = main_window.is_visible().map_err(|e| {
        error!("Failed to get `main` window visibility: {}", e);
        CmdError::new(
            CmdErrorSource::InternalError,
            "Failed to show app window".to_string(),
        )
    })?;

    if is_visible {
        debug!("`main` window is already visible");
        return Ok(());
    }

    info!("showing `main` window");
    window.get_window("main").unwrap().show().map_err(|e| {
        error!("Failed to show `main` window: {}", e);
        CmdError::new(
            CmdErrorSource::InternalError,
            "Failed to show app window".to_string(),
        )
    })?;
    Ok(())
}
