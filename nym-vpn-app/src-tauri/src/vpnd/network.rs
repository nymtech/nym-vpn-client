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
            app_update_policy: compat.app_update_policy,
        }
    }
}
