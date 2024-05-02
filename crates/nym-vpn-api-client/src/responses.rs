use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

const MAX_PROBE_RESULT_AGE_MINUTES: i64 = 60;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Gateway {
    pub identity_key: String,
    pub location: Location,
    pub last_probe: Option<Probe>,
}

// impl Gateway {
//     pub fn is_fully_operational_entry(&self) -> bool {
//         if !is_recently_updated(&self.last_probe.last_updated_utc) {
//             debug!(
//                 "Gateway {} has not been updated recently",
//                 self.identity_key
//             );
//             return false;
//         }
//
//         let is_fully_operational = self.last_probe.outcome.is_fully_operational_entry();
//         if !is_fully_operational {
//             debug!(
//                 "Gateway {} is not fully operational as entry node",
//                 self.identity_key
//             );
//         }
//         is_fully_operational
//     }
//
//     pub fn is_fully_operational_exit(&self) -> bool {
//         if !is_recently_updated(&self.last_probe.last_updated_utc) {
//             debug!(
//                 "Gateway {} has not been updated recently",
//                 self.identity_key
//             );
//             return false;
//         }
//
//         let is_fully_operational = self.last_probe.outcome.is_fully_operational_exit();
//         if !is_fully_operational {
//             debug!(
//                 "Gateway {} is not fully operational as exit node",
//                 self.identity_key
//             );
//         }
//         is_fully_operational
//     }
// }

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Location {
    pub two_letter_iso_country_code: String,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Probe {
    pub last_updated_utc: String,
    pub outcome: ProbeOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeOutcome {
    pub as_entry: Entry,
    pub as_exit: Option<Exit>,
}

impl ProbeOutcome {
    pub fn is_fully_operational_entry(&self) -> bool {
        self.as_entry.can_connect && self.as_entry.can_route
    }

    pub fn is_fully_operational_exit(&self) -> bool {
        self.as_entry.can_connect
            && self.as_entry.can_route
            && self.as_exit.as_ref().map_or(false, |exit| {
                exit.can_connect
                    && exit.can_route_ip_v4
                    && exit.can_route_ip_external_v4
                    && exit.can_route_ip_v6
                    && exit.can_route_ip_external_v6
            })
    }
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


fn is_recently_updated(last_updated_utc: &str) -> bool {
    if let Ok(last_updated) = last_updated_utc.parse::<chrono::DateTime<chrono::Utc>>() {
        let now = chrono::Utc::now();
        let duration = now - last_updated;
        duration.num_minutes() < MAX_PROBE_RESULT_AGE_MINUTES
    } else {
        false
    }
}
