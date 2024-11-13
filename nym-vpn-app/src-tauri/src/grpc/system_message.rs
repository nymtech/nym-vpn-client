use serde::Serialize;
use std::collections::HashMap;
use ts_rs::TS;

#[derive(Clone, Serialize, TS)]
#[ts(export)]
pub struct SystemMessage {
    pub name: String,
    pub message: String,
    pub properties: HashMap<String, String>,
}

impl From<&nym_vpn_proto::SystemMessage> for SystemMessage {
    fn from(msg: &nym_vpn_proto::SystemMessage) -> Self {
        SystemMessage {
            name: msg.name.clone(),
            message: msg.message.clone(),
            properties: msg.properties.clone(),
        }
    }
}
