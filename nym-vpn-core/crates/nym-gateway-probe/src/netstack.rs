pub mod binding {
    #![allow(warnings)]
    rust2go::r2g_include_binding!();
}

#[derive(rust2go::R2G, Clone)]
pub struct NetstackRequest {
    pub wg_ip: String,
    pub private_key: String,
    pub public_key: String,
    pub endpoint: String,
    pub dns: String,
    pub ping_hosts: Vec<String>,
    pub ping_ips: Vec<String>,
    pub num_ping: u8,
    pub send_timeout_sec: u64,
    pub recv_timeout_sec: u64,
}

impl Default for NetstackRequest {
    fn default() -> Self {
        Self {
            wg_ip: Default::default(),
            private_key: Default::default(),
            public_key: Default::default(),
            endpoint: Default::default(),
            dns: "1.1.1.1".to_string(),
            ping_hosts: vec!["google.com".to_string()],
            ping_ips: vec!["1.1.1.1".to_string()],
            num_ping: 3,
            send_timeout_sec: 1,
            recv_timeout_sec: 5,
        }
    }
}

#[derive(rust2go::R2G, Clone, Debug)]
pub struct NetstackResponse {
    pub can_handshake: bool,
    pub sent_ips: u16,
    pub received_ips: u16,
    pub sent_hosts: u16,
    pub received_hosts: u16,
    pub can_resolve_dns: bool,
}

#[rust2go::r2g]
pub trait NetstackCall {
    fn ping(req: &NetstackRequest) -> NetstackResponse;
}
