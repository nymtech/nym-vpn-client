use serde::Serialize;
use std::collections::HashMap;
use ts_rs::TS;

#[derive(Clone, Serialize, TS)]
#[ts(export)]
pub struct FeatureFlags {
    pub flags: HashMap<String, String>,
    pub groups: HashMap<String, FeatureFlagGroup>,
}

#[derive(Clone, Serialize, TS)]
#[ts(export)]
pub struct FeatureFlagGroup(HashMap<String, String>);

impl From<&nym_vpn_proto::GetFeatureFlagsResponse> for FeatureFlags {
    fn from(feature_flags: &nym_vpn_proto::GetFeatureFlagsResponse) -> Self {
        let mut flags = HashMap::new();
        for (key, value) in &feature_flags.flags {
            flags.insert(key.clone(), value.clone());
        }

        let mut groups = HashMap::new();
        for (group_key, group) in &feature_flags.groups {
            let mut group_flags = HashMap::new();
            for (key, value) in &group.map {
                group_flags.insert(key.clone(), value.clone());
            }
            groups.insert(group_key.clone(), FeatureFlagGroup(group_flags));
        }

        FeatureFlags { flags, groups }
    }
}
