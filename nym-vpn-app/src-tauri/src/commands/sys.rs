use crate::error::BackendError;
use crate::state::SharedAppState;
use crate::sys::OsInfo;

use tauri::State;
use tracing::instrument;

#[instrument]
#[tauri::command]
pub async fn os_info(app_state: State<'_, SharedAppState>) -> Result<OsInfo, BackendError> {
    let state = app_state.lock().await;
    Ok(state.os_info.clone())
}
