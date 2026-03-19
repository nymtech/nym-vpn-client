use nym_vpn_lib_types as lib;
use nym_vpn_lib_types::FlagValue;
use serde::Serialize;
use ts_rs::TS;

const KEY_QUIC: &str = "quic";
const KEY_DOMAIN_FRONTING: &str = "domain_fronting";
const KEY_ZKNYMS: &str = "zkNyms";
const KEY_MIXNET_TUNING: &str = "mixnet_tuning";

#[derive(Clone, Serialize, TS)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "camelCase")]
pub struct FeatureFlags {
    pub quic: bool,
    pub domain_fronting: bool,
    pub zknym_credential: bool,
    pub mixnet_tuning: bool,
}

impl From<lib::FeatureFlags> for FeatureFlags {
    fn from(fflags: lib::FeatureFlags) -> Self {
        FeatureFlags {
            quic: get_group_flag(&fflags, KEY_QUIC, "enabled").unwrap_or(false),
            domain_fronting: get_group_flag(&fflags, KEY_DOMAIN_FRONTING, "enabled")
                .unwrap_or(false),
            zknym_credential: get_group_flag(&fflags, KEY_ZKNYMS, "credentialMode")
                .unwrap_or(false),
            mixnet_tuning: get_group_flag(&fflags, KEY_MIXNET_TUNING, "enabled").unwrap_or(false),
        }
    }
}

fn get_group_flag(fflags: &lib::FeatureFlags, group_name: &str, flag_name: &str) -> Option<bool> {
    if let Some(FlagValue::Group(group)) = fflags.flags.get(group_name)
        && let Some(value) = group.get(flag_name)
    {
        Some(value == "true")
    } else {
        None
    }
}
