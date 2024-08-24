use tracing::{debug, instrument};

use crate::error::BackendError;
use crate::startup::STARTUP_ERROR;

#[instrument(skip_all)]
#[tauri::command]
pub fn startup_error() -> Result<String, BackendError> {
    debug!("startup_error");

    Ok(STARTUP_ERROR
        .get()
        .cloned()
        .unwrap_or_else(|| "No startup error found".to_string()))
}
