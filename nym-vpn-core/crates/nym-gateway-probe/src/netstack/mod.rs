pub mod ffi;

pub const V4_DNS: &str = "1.1.1.1";
pub const V6_DNS: &str = "2606:4700:4700::1111";

pub struct NetstackRequest {
    pub wg_ip: String,
    pub private_key: String,
    pub public_key: String,
    pub endpoint: String,
    pub dns: String,
    pub ip_version: u8,
    pub v4_ping_config: PingConfig,
    pub v6_ping_config: PingConfig,
}

pub struct PingConfig {
    pub ping_hosts: Vec<String>,
    pub ping_ips: Vec<String>,
    pub num_ping: u8,
    pub send_timeout_sec: u64,
    pub recv_timeout_sec: u64,
}

impl PingConfig {
    pub fn default_v4() -> Self {
        Self {
            ping_hosts: vec!["nymtech.net".to_string()],
            ping_ips: vec!["1.1.1.1".to_string()],
            ..Default::default()
        }
    }

    pub fn default_v6() -> Self {
        Self {
            ping_hosts: vec!["ipv6.google.com".to_string()],
            ping_ips: vec![
                "2001:4860:4860::8888".to_string(), // google DNS
                "2606:4700:4700::1111".to_string(), // cloudflare DNS
                "2620:fe::fe".to_string(),          //Quad9 DNS
            ],
            ..Default::default()
        }
    }
}

impl Default for PingConfig {
    fn default() -> Self {
        Self {
            ping_hosts: vec![],
            ping_ips: vec![],
            num_ping: 3,
            send_timeout_sec: 1,
            recv_timeout_sec: 2,
        }
    }
}

impl NetstackRequest {
    pub fn new(
        wg_ip: &str,
        private_key: &str,
        public_key: &str,
        endpoint: &str,
        dns: &str,
        ip_version: u8,
    ) -> Self {
        Self {
            wg_ip: wg_ip.to_string(),
            private_key: private_key.to_string(),
            public_key: public_key.to_string(),
            endpoint: endpoint.to_string(),
            dns: dns.to_string(),
            ip_version,
            v4_ping_config: PingConfig::default_v4(),
            v6_ping_config: PingConfig::default_v6(),
        }
    }

    #[allow(dead_code)]
    pub fn set_v4_config(&mut self, config: PingConfig) {
        self.v4_ping_config = config;
    }

    #[allow(dead_code)]
    pub fn set_v6_config(&mut self, config: PingConfig) {
        self.v6_ping_config = config;
    }
}

impl Default for NetstackRequest {
    fn default() -> Self {
        Self {
            wg_ip: Default::default(),
            private_key: Default::default(),
            public_key: Default::default(),
            endpoint: Default::default(),
            dns: V4_DNS.to_string(),
            ip_version: 4,
            v4_ping_config: PingConfig::default_v4(),
            v6_ping_config: PingConfig::default_v6(),
        }
    }
}