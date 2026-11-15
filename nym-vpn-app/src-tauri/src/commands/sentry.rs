use std::time::Duration;
use tauri::State;
use tracing::{debug, error, info, instrument, warn};

use crate::error::BackendError;
use crate::state::{SharedAppConfig, SharedAppState};
use crate::vpnd::client::VpndClient;

#[instrument(skip_all)]
#[tauri::command]
pub async fn enable_sentry(
    app_config: State<'_, SharedAppConfig>,
    vpnd: State<'_, VpndClient>,
) -> Result<(), BackendError> {
    info!("enabling vpnd sentry monitoring");
    vpnd.enable_sentry().await?;

    let mut config_guard = app_config.lock().await;
    let saved = config_guard.read().and_then(|mut config| {
        config.sentry_monitoring = true;
        config_guard.data = config;
        config_guard.write()
    });
    drop(config_guard);

    if let Err(e) = saved {
        // vpnd already took the change, put it back so it matches the saved setting
        warn!("failed to save sentry monitoring setting: {e}, reverting vpnd");
        vpnd.disable_sentry()
            .await
            .inspect_err(|e| {
                error!("failed to revert vpnd sentry monitoring, it may still be enabled: {e}")
            })
            .ok();
        return Err(e.into());
    }
    info!("sentry monitoring enabled, app restart required");

    Ok(())
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn disable_sentry(
    app_config: State<'_, SharedAppConfig>,
    app_state: State<'_, SharedAppState>,
    vpnd: State<'_, VpndClient>,
) -> Result<(), BackendError> {
    info!("disabling vpnd sentry monitoring");
    vpnd.disable_sentry().await?;

    let mut config_guard = app_config.lock().await;
    let saved = config_guard.read().and_then(|mut config| {
        config.sentry_monitoring = false;
        config_guard.data = config;
        config_guard.write()
    });
    drop(config_guard);

    if let Err(e) = saved {
        // vpnd already took the change, put it back so it matches the saved setting
        warn!("failed to save sentry monitoring setting: {e}, reverting vpnd");
        vpnd.enable_sentry()
            .await
            .inspect_err(|e| {
                error!("failed to revert vpnd sentry monitoring, it may still be disabled: {e}")
            })
            .ok();
        return Err(e.into());
    }

    let state_guard = app_state.lock().await;
    if let Some(client) = state_guard.sentry_client.0.as_ref() {
        // do not block UI for too long
        client.close(Some(Duration::from_millis(200)));
        info!("sentry client closed");
    }
    info!("sentry monitoring disabled ⚠ app restart required ⚠");

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
