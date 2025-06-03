use crate::env;
use std::thread::sleep;

use anyhow::Result;
use tauri::AppHandle;
use tauri_plugin_updater::{Update, UpdaterExt};
use tokio::sync::Mutex;
use tracing::{debug, error, info, instrument, trace};

pub struct PendingUpdate(pub Mutex<Option<Update>>);

// Based on https://v2.tauri.app/plugin/updater/#checking-for-updates
#[instrument(skip(app))]
pub async fn check(app: AppHandle) -> Result<Option<Update>> {
    let builder = app.updater_builder().on_before_exit(|| {
        // sleep for a short duration to allow the UI to finish rendering progress
        sleep(std::time::Duration::from_millis(400));
    });
    let update = if let Some(endpoint) = env::UPDATER_ENDPOINT {
        debug!("using endpoint: {}", endpoint);
        let url = url::Url::parse(endpoint)
            .inspect_err(|e| error!("failed to parse URL endpoint: {}", e))?;
        builder
            .endpoints(vec![url])
            .inspect_err(|e| error!("endpoint failed: {}", e))?
            .build()
            .inspect_err(|e| error!("build failed: {}", e))?
            .check()
            .await
            .inspect_err(|e| error!("check update failed: {}", e))?
    } else {
        builder
            .build()
            .inspect_err(|e| error!("build failed: {}", e))?
            .check()
            .await
            .inspect_err(|e| error!("check update failed: {}", e))?
    };
    match &update {
        Some(update) => {
            info!("new update available: {}", update.version);
        }
        None => trace!("no update available"),
    }

    Ok(update)
}
