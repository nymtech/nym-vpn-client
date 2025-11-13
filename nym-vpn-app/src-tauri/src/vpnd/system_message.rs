use nym_vpn_lib_types as lib;
use serde::Serialize;
use std::collections::HashMap;
use ts_rs::TS;

#[derive(Clone, Serialize, TS)]
#[ts(export, export_to = "tauri.ts")]
pub struct SystemMessage {
    pub name: String,
    pub message: String,
    pub display_from: Option<i64>, // unix timestamp
    pub display_until: Option<i64>,
    pub properties: Option<HashMap<String, String>>,
}

impl From<lib::SystemMessage> for SystemMessage {
    fn from(msg: lib::SystemMessage) -> Self {
        SystemMessage {
            name: msg.name,
            message: msg.message,
            display_from: msg.display_from.map(|dt| dt.unix_timestamp()),
            display_until: msg.display_until.map(|dt| dt.unix_timestamp()),
            properties: msg.properties,
        }
    }
}
