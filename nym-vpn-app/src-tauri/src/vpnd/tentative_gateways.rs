use nym_vpn_lib_types as lib;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Only the discriminant is needed by the UI to decide the connect flow, so the
/// `Selected` entry/exit payload is intentionally dropped.
#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export, export_to = "tauri.ts")]
pub enum TentativeGateways {
    Selected,
    NeedsRelaxedIndependenceCriteria,
    NoGatewaysAvailable,
}

impl From<lib::TentativeGateways> for TentativeGateways {
    fn from(value: lib::TentativeGateways) -> Self {
        match value {
            lib::TentativeGateways::Selected { .. } => TentativeGateways::Selected,
            lib::TentativeGateways::NeedsRelaxedIndependenceCriteria => {
                TentativeGateways::NeedsRelaxedIndependenceCriteria
            }
            lib::TentativeGateways::NoGatewaysAvailable => TentativeGateways::NoGatewaysAvailable,
        }
    }
}
