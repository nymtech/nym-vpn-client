use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    /// Unix socket path of gRPC endpoint in IPC mode
    pub grpc_socket_endpoint: Option<PathBuf>,
    /// IP address of the DNS server to use when connected to the VPN
    pub dns_server: Option<String>,
}
