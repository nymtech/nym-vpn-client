use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use tracing::{debug, error, warn};

const MAX_PROBE_RESULT_AGE_MINUTES: i64 = 60;

// TODO: these should have their own crate shared with the harbourmaster service

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PagedResult<T> {
    pub page: u32,
    pub size: u32,
    pub total: i32,
    pub items: Vec<T>,
}

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

// TODO: this should be a shared crate with nym-gateway-probe

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResult {
    pub gateway: String,
    pub outcome: ProbeOutcome,
}

impl ProbeResult {
    pub fn is_fully_operational_entry(&self) -> bool {
        self.outcome.as_entry.can_connect && self.outcome.as_entry.can_route
    }

    pub fn is_fully_operational_exit(&self) -> bool {
        self.outcome.as_entry.can_connect
            && self.outcome.as_entry.can_route
            && self.outcome.as_exit.as_ref().map_or(false, |exit| {
                exit.can_connect
                    && exit.can_route_ip_v4
                    && exit.can_route_ip_external_v4
                    && exit.can_route_ip_v6
                    && exit.can_route_ip_external_v6
            })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeOutcome {
    pub as_entry: Entry,
    pub as_exit: Option<Exit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub can_connect: bool,
    pub can_route: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exit {
    pub can_connect: bool,
    pub can_route_ip_v4: bool,
    pub can_route_ip_external_v4: bool,
    pub can_route_ip_v6: bool,
    pub can_route_ip_external_v6: bool,
}

impl Gateway {
    pub fn is_fully_operational_entry(&self) -> bool {
        if !is_recently_updated(&self.last_updated_utc) {
            debug!(
                "Gateway {} has not been updated recently",
                self.gateway_identity_key
            );
            return false;
        }

        let Some(ref last_probe_result) = self.last_probe_result else {
            debug!("Gateway {} has no probe result", self.gateway_identity_key);
            return false;
        };
        let probe_outcome: Result<ProbeResult, _> =
            serde_json::from_value(last_probe_result.clone());
        match probe_outcome {
            Ok(gateway) => {
                let is_fully_operational = gateway.is_fully_operational_entry();
                if !is_fully_operational {
                    debug!(
                        "Gateway {} is not fully operational as entry node",
                        self.gateway_identity_key
                    );
                }
                is_fully_operational
            }
            Err(err) => {
                warn!("Failed to parse probe result: {:?}", err);
                false
            }
        }
    }

    pub fn is_fully_operational_exit(&self) -> bool {
        if !is_recently_updated(&self.last_updated_utc) {
            debug!(
                "Gateway {} has not been updated recently",
                self.gateway_identity_key
            );
            return false;
        }

        let Some(last_probe_result) = self.last_probe_result.as_ref() else {
            debug!("Gateway {} has no probe result", self.gateway_identity_key);
            return false;
        };
        let probe_outcome: Result<ProbeResult, _> =
            serde_json::from_value(last_probe_result.clone());
        match probe_outcome {
            Ok(gateway) => {
                let is_fully_operational = gateway.is_fully_operational_exit();
                if !is_fully_operational {
                    debug!(
                        "Gateway {} is not fully operational as exit node",
                        self.gateway_identity_key
                    );
                }
                is_fully_operational
            }
            Err(err) => {
                warn!("failed to parse probe result: {:?}", err);
                false
            }
        }
    }
}

fn is_recently_updated(last_updated_utc: &str) -> bool {
    if let Ok(last_updated) = last_updated_utc.parse::<chrono::DateTime<chrono::Utc>>() {
        let now = chrono::Utc::now();
        let duration = now - last_updated;
        duration.num_minutes() < MAX_PROBE_RESULT_AGE_MINUTES
    } else {
        false
    }
}
