use std::time::Duration;
use tauri::State;
use tracing::{debug, info, instrument};

use crate::error::BackendError;
use crate::state::{SharedAppConfig, SharedAppState};
use crate::vpnd::client::VpndClient;

#[instrument(skip_all)]
#[tauri::command]
pub async fn enable_sentry(
    app_config: State<'_, SharedAppConfig>,
    vpnd: State<'_, VpndClient>,
) -> Result<(), BackendError> {
    let mut config_guard = app_config.lock().await;
    let mut config = config_guard.read()?;
    config.sentry_monitoring = true;
    config_guard.data = config;
    config_guard.write()?;
    info!("sentry monitoring enabled, app restart required");

    info!("enabling vpnd sentry monitoring");
    vpnd.enable_sentry().await?;

    Ok(())
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn disable_sentry(
    app_config: State<'_, SharedAppConfig>,
    app_state: State<'_, SharedAppState>,
    vpnd: State<'_, VpndClient>,
) -> Result<(), BackendError> {
    let mut config_guard = app_config.lock().await;
    let mut config = config_guard.read()?;
    config.sentry_monitoring = false;
    config_guard.data = config;
    config_guard.write()?;
    drop(config_guard);

    let state_guard = app_state.lock().await;
    if let Some(client) = state_guard.sentry_client.0.as_ref() {
        // do not block UI for too long
        client.close(Some(Duration::from_millis(200)));
        info!("sentry client closed");
    }
    info!("sentry monitoring disabled ⚠ app restart required ⚠");

    info!("disabling vpnd sentry monitoring");
    vpnd.disable_sentry().await?;
    Ok(())
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn sentry_enabled(app_config: State<'_, SharedAppConfig>) -> Result<bool, BackendError> {
    let config_guard = app_config.lock().await;
    let enabled = config_guard.read().is_ok_and(|c| c.sentry_monitoring);
    debug!("sentry monitoring enabled: {}", enabled);

    Ok(enabled)
}
