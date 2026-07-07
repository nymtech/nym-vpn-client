use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{debug, error, info, instrument, trace, warn};
use ts_rs::TS;

use crate::error::BackendError;
use crate::state::{SharedAppConfig, SharedDebugLogging};

#[derive(Debug, Serialize, Deserialize, TS, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[instrument(skip_all, name = "js")]
#[tauri::command]
pub async fn log_js(message: String, level: Option<Level>) -> Result<(), BackendError> {
    match level {
        Some(Level::Trace) => trace!(message),
        Some(Level::Debug) => debug!(message),
        Some(Level::Info) => info!(message),
        Some(Level::Warn) => warn!(message),
        Some(Level::Error) => error!(message),
        None => info!(message),
    }

    Ok(())
}

#[instrument(skip(app_config, control))]
#[tauri::command]
pub async fn set_debug_logging(
    enabled: bool,
    app_config: State<'_, SharedAppConfig>,
    control: State<'_, SharedDebugLogging>,
) -> Result<(), BackendError> {
    let mut config_guard = app_config.lock().await;
    let mut control_guard = control.lock().await;

    control_guard.set(enabled).map_err(|e| {
        error!("failed to apply debug logging state: {e}");
        BackendError::internal_with_detail("failed to apply debug logging state", e.to_string())
    })?;

    let mut config = config_guard.read()?;
    config.debug_logging = enabled;
    config_guard.data = config;
    config_guard.write()?;

    info!(
        "app debug logging {}",
        if enabled { "enabled" } else { "disabled" }
    );
    Ok(())
}

#[instrument(skip(control))]
#[tauri::command]
pub async fn debug_logging_enabled(
    control: State<'_, SharedDebugLogging>,
) -> Result<bool, BackendError> {
    Ok(control.lock().await.enabled())
}
