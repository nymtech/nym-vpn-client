use std::fs;

use tracing::{debug, error, info, instrument};

use crate::error::BackendError;
use crate::fs::path::APP_LOG_DIR;

#[instrument]
#[tauri::command]
pub async fn log_dir() -> Result<String, BackendError> {
    let log_path = APP_LOG_DIR.clone().ok_or_else(|| {
        let err = "Failed to get log directory path";
        error!(err);
        BackendError::internal(err, None)
    })?;
    let log_dir = log_path.to_str().ok_or_else(|| {
        let err = "Failed to get log directory path";
        error!(err);
        BackendError::internal(err, None)
    })?;

    debug!("log directory: {}", log_dir);
    Ok(log_dir.into())
}

#[instrument]
#[tauri::command]
pub async fn delete_app_logs() -> Result<(), BackendError> {
    let log_path = APP_LOG_DIR.clone().ok_or_else(|| {
        let err = "Failed to get log directory path";
        error!(err);
        BackendError::internal(err, None)
    })?;

    debug!("deleting all contents of log directory: {:?}", log_path);

    let entries = fs::read_dir(&log_path).map_err(|e| {
        let err = format!("Failed to read log directory: {}", e);
        error!(err);
        BackendError::internal(&err, None)
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            let err = format!("Failed to read directory entry: {}", e);
            error!(err);
            BackendError::internal(&err, None)
        })?;

        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(&path).map_err(|e| {
                let err = format!("Failed to remove directory {:?}: {}", path, e);
                error!(err);
                BackendError::internal(&err, None)
            })?;
        } else {
            fs::remove_file(&path).map_err(|e| {
                let err = format!("Failed to remove file {:?}: {}", path, e);
                error!(err);
                BackendError::internal(&err, None)
            })?;
        }
    }

    info!("successfully deleted all contents of log directory");
    Ok(())
}
