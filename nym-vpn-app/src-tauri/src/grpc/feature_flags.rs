use nym_vpn_proto::proto::{FeatureFlagGroup, GetFeatureFlagsResponse};
use serde::Serialize;
use std::collections::HashMap;
use ts_rs::TS;

const KEY_VERSIONS: &str = "versions";
const KEY_QUIC: &str = "quic";
const KEY_DOMAIN_FRONTING: &str = "domain_fronting";
const KEY_ZKNYMS: &str = "zkNyms";
const KEY_GW_UPDATE: &str = "gatewayMetadataUpdate";

#[derive(Clone, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct FeatureFlags {
    pub quic: bool,
    pub domain_fronting: bool,
    pub zknym_credential: bool,
    pub gateway_update_version: Option<String>,
    pub flags: HashMap<String, String>,
}

impl From<&GetFeatureFlagsResponse> for FeatureFlags {
    fn from(feature_flags: &GetFeatureFlagsResponse) -> Self {
        let mut flags = HashMap::new();
        for (key, value) in &feature_flags.flags {
            flags.insert(key.clone(), value.clone());
        }

        FeatureFlags {
            quic: get_bool_value(&feature_flags.groups, KEY_QUIC, "enabled"),
            domain_fronting: get_bool_value(&feature_flags.groups, KEY_DOMAIN_FRONTING, "enabled"),
            zknym_credential: get_bool_value(&feature_flags.groups, KEY_ZKNYMS, "credentialMode"),
            gateway_update_version: get_version_value(&feature_flags.groups, KEY_GW_UPDATE),
            flags,
        }
    }
}

fn get_bool_value(map: &HashMap<String, FeatureFlagGroup>, key: &str, subkey: &str) -> bool {
    map.get(key)
        .map(|group| {
            group.map.get(subkey).map(|v| match v.as_str() {
                "true" | "TRUE" => true,
                _ => false,
            })
        })
        .flatten()
        .unwrap_or(false)
}

fn get_version_value(map: &HashMap<String, FeatureFlagGroup>, key: &str) -> Option<String> {
    map.get(KEY_VERSIONS)
        .and_then(|group| group.map.get(key))
        .cloned()
}
