use nym_vpn_lib_types as lib;
use nym_vpn_lib_types::FlagValue;
use serde::Serialize;
use ts_rs::TS;

const KEY_QUIC: &str = "quic";
const KEY_DOMAIN_FRONTING: &str = "domain_fronting";
const KEY_ZKNYMS: &str = "zkNyms";

#[derive(Clone, Serialize, TS)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "camelCase")]
pub struct FeatureFlags {
    pub quic: bool,
    pub domain_fronting: bool,
    pub zknym_credential: bool,
}

impl From<lib::FeatureFlags> for FeatureFlags {
    fn from(fflags: lib::FeatureFlags) -> Self {
        FeatureFlags {
            quic: get_group_flag(&fflags, KEY_QUIC, "enabled").unwrap_or(false),
            domain_fronting: get_group_flag(&fflags, KEY_DOMAIN_FRONTING, "enabled")
                .unwrap_or(false),
            zknym_credential: get_group_flag(&fflags, KEY_ZKNYMS, "credentialMode")
                .unwrap_or(false),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn group(entries: &[(&str, &str)]) -> FlagValue {
        FlagValue::Group(
            entries
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        )
    }

    fn lib_flags(groups: &[(&str, FlagValue)]) -> lib::FeatureFlags {
        lib::FeatureFlags {
            flags: groups
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
        }
    }

    #[test]
    fn maps_group_flags_to_booleans() {
        let ff: FeatureFlags = lib_flags(&[
            (KEY_QUIC, group(&[("enabled", "true")])),
            (KEY_DOMAIN_FRONTING, group(&[("enabled", "false")])),
            (KEY_ZKNYMS, group(&[("credentialMode", "true")])),
        ])
        .into();
        assert!(ff.quic);
        assert!(!ff.domain_fronting);
        assert!(ff.zknym_credential);
    }

    #[test]
    fn defaults_missing_flags_to_false() {
        let ff: FeatureFlags = lib_flags(&[]).into();
        assert!(!ff.quic);
        assert!(!ff.domain_fronting);
        assert!(!ff.zknym_credential);
    }

    #[test]
    fn treats_a_non_group_flag_value_as_absent() {
        let ff: FeatureFlags =
            lib_flags(&[(KEY_QUIC, FlagValue::Value("true".to_string()))]).into();
        assert!(!ff.quic);
    }

    #[test]
    fn only_the_literal_true_counts_as_enabled() {
        let ff: FeatureFlags = lib_flags(&[(KEY_QUIC, group(&[("enabled", "1")]))]).into();
        assert!(!ff.quic);
    }
}
