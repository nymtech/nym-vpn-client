use tracing::{instrument, trace};

use crate::startup_error::{StartupError, STARTUP_ERROR};

#[instrument(skip_all)]
#[tauri::command]
pub fn startup_error() -> Option<StartupError> {
    STARTUP_ERROR.get().cloned().inspect(|e| {
        trace!("{:#?}", e);
    })
}
