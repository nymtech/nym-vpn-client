use tauri::api::path::cache_dir;
use tracing::{debug, error, instrument};

use crate::{error::BackendError, log::LOG_DIR, APP_DIR};

#[instrument]
#[tauri::command]
pub async fn log_dir() -> Result<String, BackendError> {
    debug!("log_dir");
    let cache_dir = cache_dir().ok_or_else(|| {
        let err = "Failed to get cache directory path";
        error!(err);
        BackendError::new_internal(err, None)
    })?;
    let log_path = cache_dir.join(format!("{}/{}", APP_DIR, LOG_DIR));
    let log_dir = log_path.to_str().ok_or_else(|| {
        let err = "Failed to get log directory path";
        error!(err);
        BackendError::new_internal(err, None)
    })?;
    debug!("log directory: {}", log_dir);
    Ok(log_dir.into())
}
