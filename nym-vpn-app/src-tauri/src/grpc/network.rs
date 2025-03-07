#[derive(Debug, Default, Clone)]
pub struct NetworkCompatVersions {
    pub core: String,
    pub tauri: String,
}

impl From<nym_vpn_proto::NetworkCompatibility> for NetworkCompatVersions {
    fn from(compat: nym_vpn_proto::NetworkCompatibility) -> Self {
        NetworkCompatVersions {
            core: compat.core,
            tauri: compat.tauri,
        }
    }
}
