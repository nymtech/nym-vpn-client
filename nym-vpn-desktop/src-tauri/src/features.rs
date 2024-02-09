use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub enum FeatureFlag {
    DefaultNodeLocation,
    FastestNodeLocation,
}

pub const FEATURE_FLAGS: [FeatureFlag; 1] = [FeatureFlag::DefaultNodeLocation];
