use crate::env::DEV_MODE;
use crate::error::BackendError;
use crate::states::SharedAppState;
use tauri::State;
use tracing::{instrument, warn};

#[instrument(skip(state))]
#[tauri::command]
pub async fn get_credentials_mode(state: State<'_, SharedAppState>) -> Result<bool, BackendError> {
    let state = state.lock().await;
    Ok(state.credentials_mode)
}

#[instrument(skip(state))]
#[tauri::command]
pub async fn set_credentials_mode(
    state: State<'_, SharedAppState>,
    enabled: bool,
) -> Result<(), BackendError> {
    if !*DEV_MODE {
        warn!("not in dev mode");
        return Err(BackendError::internal("nope", None));
    }
    let mut state = state.lock().await;
    state.credentials_mode = enabled;
    Ok(())
}
