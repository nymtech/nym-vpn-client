use serde::{Deserialize, Serialize};
use ts_rs::TS;

// these types are only used on Windows but need to be
// TS exported anyway

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, TS, Clone)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "tauri.ts")]
pub struct UpdateMetadata {
    pub version: String,
    pub current_version: String,
}

#[allow(dead_code)]
#[derive(Clone, Serialize, TS)]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "kebab-case", rename_all_fields = "camelCase")]
#[ts(export, export_to = "tauri.ts")]
pub enum DownloadUpdateEvent {
    Started { content_length: u64 },
    Progress { chunk_length: usize },
    Finished,
}
