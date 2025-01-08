use crate::env::DEV_MODE;
use serde::Serialize;
use tracing::instrument;
use ts_rs::TS;

#[derive(Serialize, Debug, Clone, TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[ts(export)]
pub struct Env {
    dev_mode: bool,
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn env() -> Env {
    Env {
        dev_mode: *DEV_MODE,
    }
}
