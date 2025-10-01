use crate::env::UPDATER_ENABLED;
use crate::error::BackendError;
use crate::updater;
use crate::updater::PendingUpdate;

use super::updater_types::{DownloadUpdateEvent, UpdateMetadata};
use tauri::ipc::Channel;
use tauri::{AppHandle, State};
use tracing::{debug, error, instrument, trace};

#[tauri::command]
#[instrument(skip_all)]
pub async fn fetch_update(
    app: AppHandle,
    pending_update: State<'_, PendingUpdate>,
) -> Result<Option<UpdateMetadata>, BackendError> {
    if !*UPDATER_ENABLED {
        error!("updater is disabled for this build");
        return Err(BackendError::internal("updater is disabled", None));
    }
    let update = updater::check(app)
        .await
        .map_err(|_| BackendError::internal("updater failed to check for update", None))?;

    match update {
        Some(update) => {
            let metadata = UpdateMetadata {
                version: update.version.clone(),
                current_version: update.current_version.clone(),
            };
            pending_update.0.lock().await.replace(update);
            Ok(Some(metadata))
        }
        None => Ok(None),
    }
}

// Based on https://v2.tauri.app/plugin/updater/#checking-for-updates
#[tauri::command]
#[instrument(skip_all)]
pub async fn install_update(
    pending_update: State<'_, PendingUpdate>,
    on_event: Channel<DownloadUpdateEvent>,
) -> Result<(), BackendError> {
    if !*UPDATER_ENABLED {
        error!("updater is disabled for this build");
        return Err(BackendError::internal("updater is disabled", None));
    }
    let Some(update) = pending_update.0.lock().await.take() else {
        // calling this function without a pending update is an error
        error!("no update available");
        return Err(BackendError::internal("no update available", None));
    };

    let mut started = false;
    update
        .download_and_install(
            |chunk_length, content_length| {
                trace!("downloaded chunk: {chunk_length}, content length: {content_length:?}");
                if !started {
                    debug!(
                        "update download started, content length: {:?}",
                        content_length
                    );
                    let _ = on_event.send(DownloadUpdateEvent::Started {
                        content_length: content_length.unwrap_or(20_000_000), // default to 20MB
                    });
                    started = true;
                }
                let _ = on_event.send(DownloadUpdateEvent::Progress { chunk_length });
            },
            || {
                debug!("update download finished");
                let _ = on_event.send(DownloadUpdateEvent::Finished);
            },
        )
        .await
        .inspect_err(|e| error!("download and install failed: {}", e))?;
    Ok(())
}
