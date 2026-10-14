use nym_vpn_lib_types as lib;

#[derive(Debug, Default, Clone)]
pub struct NetworkCompatVersions {
    pub core: String,
    pub tauri: String,
    pub app_update_policy: String,
}

impl From<lib::NetworkCompatibility> for NetworkCompatVersions {
    fn from(compat: lib::NetworkCompatibility) -> Self {
        NetworkCompatVersions {
            core: compat.core,
            tauri: compat.tauri,
            app_update_policy: compat
                .app_update_policies
                .get("tauri")
                .cloned()
                .unwrap_or(compat.app_update_policy),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn compat(policies: &[(&str, &str)]) -> lib::NetworkCompatibility {
        lib::NetworkCompatibility {
            core: "2026.12.4".to_owned(),
            ios: "2026.12.4".to_owned(),
            macos: "2026.12.4".to_owned(),
            tauri: "2026.12.4".to_owned(),
            android: "2026.12.4".to_owned(),
            app_update_policy: "dismissible".to_owned(),
            app_update_policies: policies
                .iter()
                .map(|(platform, policy)| (platform.to_string(), policy.to_string()))
                .collect::<HashMap<_, _>>(),
        }
    }

    #[test]
    fn desktop_uses_its_own_policy_and_falls_back_to_the_global_one() {
        let own = NetworkCompatVersions::from(compat(&[("tauri", "required"), ("android", "x")]));
        assert_eq!(own.app_update_policy, "required");
        let global = NetworkCompatVersions::from(compat(&[("android", "required")]));
        assert_eq!(global.app_update_policy, "dismissible");
    }
}
