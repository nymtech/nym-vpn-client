use crate::commands::startup;
use crate::db::DbError;

use anyhow::{anyhow, Result};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use tauri::Manager;
use tracing::{error, info, warn};
use ts_rs::TS;

pub static STARTUP_ERROR: OnceCell<StartupError> = OnceCell::new();
const ERROR_WIN_LABEL: &str = "error";

#[derive(Debug, Serialize, Deserialize, TS, Clone)]
#[ts(export, export_to = "StartupErrorKey.ts")]
pub enum ErrorKey {
    /// At startup, failed to open the embedded db, generic
    StartupOpenDb,
    /// At startup, failed to open the embedded db because it is already locked
    StartupOpenDbLocked,
}

#[derive(Debug, Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub struct StartupError {
    pub key: ErrorKey,
    pub details: Option<String>,
}

impl StartupError {
    pub fn new(key: ErrorKey, details: Option<String>) -> Self {
        Self { key, details }
    }
}

pub fn set_error(key: ErrorKey, details: Option<&str>) {
    STARTUP_ERROR
        .set(StartupError::new(key, details.map(String::from)))
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

impl From<&DbError> for ErrorKey {
    fn from(value: &DbError) -> Self {
        match value {
            DbError::Locked(_) => ErrorKey::StartupOpenDbLocked,
            _ => ErrorKey::StartupOpenDb,
        }
    }
}
