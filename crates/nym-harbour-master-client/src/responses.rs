use serde::{Deserialize, Serialize};

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
        // TODO: check if result is outdated
        let Some(ref last_probe_result) = self.last_probe_result else {
            return false;
        };
        let probe_outcome: ProbeResult = serde_json::from_value(last_probe_result.clone()).unwrap();
        probe_outcome.is_fully_operational_entry()
    }

    pub fn is_fully_operational_exit(&self) -> bool {
        // TODO: check if result is outdated
        let Some(last_probe_result) = self.last_probe_result.as_ref() else {
            return false;
        };
        let probe_outcome: ProbeResult = serde_json::from_value(last_probe_result.clone()).unwrap();
        probe_outcome.is_fully_operational_exit()
    }
}
