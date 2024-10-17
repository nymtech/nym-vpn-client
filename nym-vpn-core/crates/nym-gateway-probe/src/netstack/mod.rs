pub mod ffi;

pub const V4_DNS: &str = "1.1.1.1";
pub const V6_DNS: &str = "2606:4700:4700::1111";

pub struct NetstackRequest {
    private_key: String,
    public_key: String,
    endpoint: String,
    v4_ping_config: PingConfig,
    v6_ping_config: PingConfig,
    awg_args: String,
}

pub struct PingConfig {
    self_ip: String,
    dns: String,
    ping_hosts: Vec<String>,
    ping_ips: Vec<String>,
    num_ping: u8,
    send_timeout_sec: u64,
    recv_timeout_sec: u64,
}

impl PingConfig {
    pub fn default_v4(self_ip: &str, dns_v4: Option<String>) -> Self {
        Self {
            self_ip: self_ip.to_string(),
            dns: dns_v4.unwrap_or(V4_DNS.to_string()),
            ping_hosts: vec!["nymtech.net".to_string()],
            ping_ips: vec!["1.1.1.1".to_string()],
            ..Default::default()
        }
    }

    pub fn default_v6(self_ip: &str, dns_v6: Option<String>) -> Self {
        Self {
            self_ip: self_ip.to_string(),
            dns: dns_v6.unwrap_or(V6_DNS.to_string()),
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
            self_ip: Default::default(),
            dns: Default::default(),
            ping_hosts: vec![],
            ping_ips: vec![],
            num_ping: 10,
            send_timeout_sec: 3,
            recv_timeout_sec: 3,
        }
    }
}

impl NetstackRequest {
    pub fn new(
        wg_ip4: &str,
        wg_ip6: &str,
        private_key: &str,
        public_key: &str,
        endpoint: &str,
        dns_v4: Option<String>,
        dns_v6: Option<String>,
        awg_args: &str,
    ) -> Self {
        Self {
            private_key: private_key.to_string(),
            public_key: public_key.to_string(),
            endpoint: endpoint.to_string(),
            awg_args: awg_args.to_string(),
            v4_ping_config: PingConfig::default_v4(wg_ip4, dns_v4),
            v6_ping_config: PingConfig::default_v6(wg_ip6, dns_v6),
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
            private_key: Default::default(),
            public_key: Default::default(),
            endpoint: Default::default(),
            v4_ping_config: PingConfig::default_v4(Default::default(), None),
            v6_ping_config: PingConfig::default_v6(Default::default(), None),
            awg_args: Default::default(),
        }
    }
}
