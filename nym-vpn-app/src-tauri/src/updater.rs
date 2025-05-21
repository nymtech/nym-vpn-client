use anyhow::Result;
use tauri::AppHandle;
use tauri_plugin_updater::{Update, UpdaterExt};
use tracing::{info, trace};

const ENDPOINT: &str = "https://raw.githubusercontent.com/nymtech/nym-vpn-client/refs/heads/develop/nym-vpn-app/updater.json";

// Based on https://v2.tauri.app/plugin/updater/#checking-for-updates
pub async fn check(app: AppHandle) -> Result<Option<Update>> {
    let url = url::Url::parse(ENDPOINT).expect("invalid updater endpoint URL");

    let update = app
        .updater_builder()
        .endpoints(vec![url])?
        .build()?
        .check()
        .await?;
    match &update {
        Some(update) => {
            info!("new update available: {}", update.version);
        }
        None => trace!("no update available"),
    }

    Ok(update)
}
