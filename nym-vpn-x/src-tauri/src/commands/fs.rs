use tracing::{debug, error, instrument};

use crate::error::BackendError;
use crate::fs::path::LOG_DIR_PATH;

#[instrument]
#[tauri::command]
pub async fn log_dir() -> Result<String, BackendError> {
    debug!("log_dir");
    let log_path = LOG_DIR_PATH.clone().ok_or_else(|| {
        let err = "Failed to get log directory path";
        error!(err);
        BackendError::new_internal(err, None)
    })?;
    let log_dir = log_path.to_str().ok_or_else(|| {
        let err = "Failed to get log directory path";
        error!(err);
        BackendError::new_internal(err, None)
    })?;

    debug!("log directory: {}", log_dir);
    Ok(log_dir.into())
}
