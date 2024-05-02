use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    /// Address of NymVpn daemon to connect to (gRPC server endpoint)
    pub daemon_address: Option<String>,
    /// IP address of the DNS server to use when connected to the VPN
    pub dns_server: Option<String>,
    /// Path pointing to an env configuration file describing the network
    pub env_config_file: Option<PathBuf>,
}
