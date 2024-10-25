use crate::env::NETWORK_ENV_SELECT;
use serde::Serialize;
use tracing::{debug, instrument};
use ts_rs::TS;

#[derive(Serialize, Debug, Clone, TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[ts(export)]
pub struct Env {
    network_env_select: bool,
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn env() -> Env {
    debug!("env");
    Env {
        network_env_select: *NETWORK_ENV_SELECT,
    }
}
