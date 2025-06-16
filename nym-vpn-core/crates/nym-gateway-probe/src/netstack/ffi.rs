use super::NetstackRequest;

pub mod binding {
    #![allow(warnings)]
    rust2go::r2g_include_binding!();
}

#[derive(rust2go::R2G, Clone)]
pub struct NetstackRequestGo {
    pub wg_ip: String,
    private_key: String,
    public_key: String,
    endpoint: String,
    pub dns: String,
    ip_version: u8,
    ping_hosts: Vec<String>,
    ping_ips: Vec<String>,
    num_ping: u8,
    send_timeout_sec: u64,
    recv_timeout_sec: u64,
    download_timeout_sec: u64,
    awg_args: String,
}

impl NetstackRequestGo {
    pub fn from_rust_v4(req: &NetstackRequest) -> Self {
        NetstackRequestGo {
            wg_ip: req.v4_ping_config.self_ip.clone(),
            private_key: req.private_key.clone(),
            public_key: req.public_key.clone(),
            endpoint: req.endpoint.clone(),
            dns: req.v4_ping_config.dns.clone(),
            ip_version: 4,
            ping_hosts: req.v4_ping_config.ping_hosts.clone(),
            ping_ips: req.v4_ping_config.ping_ips.clone(),
            num_ping: req.v4_ping_config.num_ping,
            send_timeout_sec: req.v4_ping_config.send_timeout_sec,
            recv_timeout_sec: req.v4_ping_config.recv_timeout_sec,
            download_timeout_sec: req.download_timeout_sec,
            awg_args: req.awg_args.clone(),
        }
    }

    pub fn from_rust_v6(req: &NetstackRequest) -> Self {
        NetstackRequestGo {
            wg_ip: req.v6_ping_config.self_ip.clone(),
            private_key: req.private_key.clone(),
            public_key: req.public_key.clone(),
            endpoint: req.endpoint.clone(),
            dns: req.v6_ping_config.dns.clone(),
            ip_version: 6,
            ping_hosts: req.v6_ping_config.ping_hosts.clone(),
            ping_ips: req.v6_ping_config.ping_ips.clone(),
            num_ping: req.v6_ping_config.num_ping,
            send_timeout_sec: req.v6_ping_config.send_timeout_sec,
            recv_timeout_sec: req.v6_ping_config.recv_timeout_sec,
            download_timeout_sec: req.download_timeout_sec,
            awg_args: req.awg_args.clone(),
        }
    }
}

#[rust2go::r2g]
pub trait NetstackCall {
    fn ping(req: &NetstackRequestGo) -> NetstackResponse;
    fn get_last_response() -> NetstackResponse;
}

#[derive(rust2go::R2G, Clone, Debug)]
pub struct NetstackResponse {
    pub can_handshake: bool,
    pub sent_ips: u16,
    pub received_ips: u16,
    pub sent_hosts: u16,
    pub received_hosts: u16,
    pub can_resolve_dns: bool,
    pub downloaded_file: String,
    pub download_duration_sec: u64,
    pub download_error: String,
}

impl NetstackCall for binding::NetstackCallImpl {
    fn ping(req: &NetstackRequestGo) -> NetstackResponse {
        // Call the Go function that stores the result globally (no callback)
        unsafe {
            binding::CNetstackCall_ping(
                binding::NetstackRequestGoRef::from(req),
                std::ptr::null_mut(), // slot - not used anymore
                std::ptr::null_mut(), // callback - not used anymore
            );
        }
        
        Self::get_last_response()
    }
    
    fn get_last_response() -> NetstackResponse {
        // Call the Go function to retrieve the last stored response
        unsafe {
            let response_ref = binding::CNetstackCall_get_last_response();
            NetstackResponse::from(response_ref)
        }
    }
}
