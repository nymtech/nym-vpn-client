use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    /// Unix socket path of gRPC endpoint in IPC mode
    pub grpc_socket_endpoint: Option<PathBuf>,
    /// Enable HTTP transport for gRPC connection
    pub grpc_http_mode: Option<bool>,
    /// Address of gRPC endpoint in HTTP mode
    pub grpc_http_endpoint: Option<String>,
    /// IP address of the DNS server to use when connected to the VPN
    pub dns_server: Option<String>,
}
