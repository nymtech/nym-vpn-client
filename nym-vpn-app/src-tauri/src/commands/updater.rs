use crate::env::UPDATER_ENABLED;
use crate::error::BackendError;
use crate::state::updater::PendingUpdate;
use crate::updater;

use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;
use tauri::{AppHandle, State};
use tracing::{error, instrument};
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, TS, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMetadata {
    version: String,
    current_version: String,
}

#[derive(Clone, Serialize, TS)]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "kebab-case", rename_all_fields = "camelCase")]
#[ts(export)]
pub enum DownloadEvent {
    Started { content_length: Option<u64> },
    Progress { chunk_length: usize },
    Finished,
}

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
    on_event: Channel<DownloadEvent>,
) -> Result<(), BackendError> {
    if !*UPDATER_ENABLED {
        error!("updater is disabled for this build");
        return Err(BackendError::internal("updater is disabled", None));
    }
    let Some(update) = pending_update.0.lock().await.take() else {
        return Err(BackendError::internal("no update available", None));
    };

    let mut started = false;
    update
        .download_and_install(
            |chunk_length, content_length| {
                if !started {
                    let _ = on_event.send(DownloadEvent::Started { content_length });
                    started = true;
                }
                let _ = on_event.send(DownloadEvent::Progress { chunk_length });
            },
            || {
                let _ = on_event.send(DownloadEvent::Finished);
            },
        )
        .await?;
    Ok(())
}
