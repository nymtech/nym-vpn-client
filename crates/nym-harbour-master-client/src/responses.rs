use serde::{Deserialize, Serialize};

// TODO: these should have their own crate shared with the harbourmaster service

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PagedResult<T> {
    pub page: u32,
    pub size: u32,
    pub total: i32,
    pub items: Vec<T>,
}

// #[derive(Debug, Clone, Deserialize, Serialize)]
// pub struct Status {
//     pub message: String,
//     pub timestamp: String,
// }

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Gateway {
    pub gateway_identity_key: String,
    pub self_described: Option<serde_json::Value>,
    pub explorer_pretty_bond: Option<serde_json::Value>,
    pub last_probe_result: Option<serde_json::Value>,
    pub last_probe_log: Option<String>,
    pub last_testrun_utc: Option<String>,
    pub last_updated_utc: String,
    pub routing_score: f32,
    pub config_score: u32,
}
