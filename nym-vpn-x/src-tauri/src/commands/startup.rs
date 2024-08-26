use tracing::{debug, instrument, trace};

use crate::startup::{StartupError, STARTUP_ERROR};

#[instrument(skip_all)]
#[tauri::command]
pub fn startup_error() -> Option<StartupError> {
    debug!("startup_error");
    STARTUP_ERROR.get().cloned().inspect(|e| {
        trace!("{:#?}", e);
    })
}
