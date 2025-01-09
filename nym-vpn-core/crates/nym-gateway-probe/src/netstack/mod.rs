use crate::NetstackArgs;

pub mod ffi;

pub struct NetstackRequest {
    private_key: String,
    public_key: String,
    endpoint: String,
    v4_ping_config: PingConfig,
    v6_ping_config: PingConfig,
    download_timeout_sec: u64,
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
    pub fn from_netstack_args_v4(wg_ip4: &str, args: &NetstackArgs) -> Self {
        Self {
            self_ip: wg_ip4.to_string(),
            dns: args.netstack_v4_dns.clone(),
            ping_hosts: args.netstack_ping_hosts_v4.clone(),
            ping_ips: args.netstack_ping_ips_v4.clone(),
            num_ping: args.netstack_num_ping,
            send_timeout_sec: args.netstack_send_timeout_sec,
            recv_timeout_sec: args.netstack_recv_timeout_sec,
        }
    }

    pub fn from_netstack_args_v6(wg_ip6: &str, args: &NetstackArgs) -> Self {
        Self {
            self_ip: wg_ip6.to_string(),
            dns: args.netstack_v6_dns.clone(),
            ping_hosts: args.netstack_ping_hosts_v6.clone(),
            ping_ips: args.netstack_ping_ips_v6.clone(),
            num_ping: args.netstack_num_ping,
            send_timeout_sec: args.netstack_send_timeout_sec,
            recv_timeout_sec: args.netstack_recv_timeout_sec,
        }
    }
}

impl NetstackRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        wg_ip4: &str,
        wg_ip6: &str,
        private_key: &str,
        public_key: &str,
        endpoint: &str,
        download_timeout_sec: u64,
        awg_args: &str,
        netstack_args: NetstackArgs,
    ) -> Self {
        Self {
            private_key: private_key.to_string(),
            public_key: public_key.to_string(),
            endpoint: endpoint.to_string(),
            awg_args: awg_args.to_string(),
            v4_ping_config: PingConfig::from_netstack_args_v4(wg_ip4, &netstack_args),
            v6_ping_config: PingConfig::from_netstack_args_v6(wg_ip6, &netstack_args),
            download_timeout_sec,
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
