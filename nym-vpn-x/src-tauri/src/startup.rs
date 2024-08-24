use anyhow::{anyhow, Result};
use once_cell::sync::OnceCell;
use tauri::Manager;
use tracing::{error, info, warn};

use crate::commands::startup;

pub static STARTUP_ERROR: OnceCell<String> = OnceCell::new();
const ERROR_WIN_LABEL: &str = "error";

pub fn set_error(error: String) {
    STARTUP_ERROR
        .set(error)
        .inspect_err(|_| {
            warn!("failed to set startup error: already set");
        })
        .ok();
}

pub fn show_error_window() -> Result<()> {
    let context = tauri::generate_context!();

    info!("Starting tauri app");
    tauri::Builder::default()
        .setup(move |app| {
            info!("app setup");

            let window = app
                .get_window(ERROR_WIN_LABEL)
                .ok_or_else(|| anyhow!("failed to get window {}", ERROR_WIN_LABEL))?;

            window
                .show()
                .inspect_err(|e| error!("failed to show window {}: {}", ERROR_WIN_LABEL, e))?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![startup::startup_error])
        .run(context)
        .expect("error while running tauri application");

    Ok(())
}
