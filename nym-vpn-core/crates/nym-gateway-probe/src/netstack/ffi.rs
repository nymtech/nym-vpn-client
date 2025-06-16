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

// Helper functions to use the callback-free approach
impl NetstackResponse {
    /// Get the last response stored globally in Go
    pub fn get_last_response() -> Self {
        unsafe {
            let response_ref = binding::CNetstackCall_get_last_response();
            
            // Convert the C strings safely
            let downloaded_file = if response_ref.downloaded_file.ptr.is_null() || response_ref.downloaded_file.len == 0 {
                String::new()
            } else {
                let bytes = std::slice::from_raw_parts(response_ref.downloaded_file.ptr, response_ref.downloaded_file.len);
                String::from_utf8_lossy(bytes).into_owned()
            };
            
            let download_error = if response_ref.download_error.ptr.is_null() || response_ref.download_error.len == 0 {
                String::new()
            } else {
                let bytes = std::slice::from_raw_parts(response_ref.download_error.ptr, response_ref.download_error.len);
                String::from_utf8_lossy(bytes).into_owned()
            };
            
            Self {
                can_handshake: response_ref.can_handshake,
                sent_ips: response_ref.sent_ips,
                received_ips: response_ref.received_ips,
                sent_hosts: response_ref.sent_hosts,
                received_hosts: response_ref.received_hosts,
                can_resolve_dns: response_ref.can_resolve_dns,
                downloaded_file,
                download_duration_sec: response_ref.download_duration_sec,
                download_error,
            }
        }
    }
}

// Helper functions to call C functions directly, bypassing rust2go trait issues
impl NetstackRequestGo {
    /// Call ping directly via C function, bypassing the problematic trait implementation
    pub fn call_ping_direct(&self) -> NetstackResponse {
        unsafe {
            // Convert strings to C string refs
            let wg_ip_bytes = self.wg_ip.as_bytes();
            let private_key_bytes = self.private_key.as_bytes();
            let public_key_bytes = self.public_key.as_bytes();
            let endpoint_bytes = self.endpoint.as_bytes();
            let dns_bytes = self.dns.as_bytes();
            let awg_args_bytes = self.awg_args.as_bytes();
            
            // Convert Vec<String> to Vec<&[u8]> for hosts and ips
            let ping_hosts_bytes: Vec<&[u8]> = self.ping_hosts.iter().map(|s| s.as_bytes()).collect();
            let ping_ips_bytes: Vec<&[u8]> = self.ping_ips.iter().map(|s| s.as_bytes()).collect();
            
            // Create C NetstackRequestGoRef
            let req_ref = binding::NetstackRequestGoRef {
                wg_ip: binding::StringRef {
                    ptr: wg_ip_bytes.as_ptr(),
                    len: wg_ip_bytes.len(),
                },
                private_key: binding::StringRef {
                    ptr: private_key_bytes.as_ptr(),
                    len: private_key_bytes.len(),
                },
                public_key: binding::StringRef {
                    ptr: public_key_bytes.as_ptr(),
                    len: public_key_bytes.len(),
                },
                endpoint: binding::StringRef {
                    ptr: endpoint_bytes.as_ptr(),
                    len: endpoint_bytes.len(),
                },
                dns: binding::StringRef {
                    ptr: dns_bytes.as_ptr(),
                    len: dns_bytes.len(),
                },
                ip_version: self.ip_version,
                ping_hosts: binding::ListRef {
                    ptr: if ping_hosts_bytes.is_empty() { std::ptr::null() } else { ping_hosts_bytes.as_ptr() as *const std::ffi::c_void },
                    len: ping_hosts_bytes.len(),
                },
                ping_ips: binding::ListRef {
                    ptr: if ping_ips_bytes.is_empty() { std::ptr::null() } else { ping_ips_bytes.as_ptr() as *const std::ffi::c_void },
                    len: ping_ips_bytes.len(),
                },
                num_ping: self.num_ping,
                send_timeout_sec: self.send_timeout_sec,
                recv_timeout_sec: self.recv_timeout_sec,
                download_timeout_sec: self.download_timeout_sec,
                awg_args: binding::StringRef {
                    ptr: awg_args_bytes.as_ptr(),
                    len: awg_args_bytes.len(),
                },
            };
            
            // Call the ping function directly (stores result globally, no callback)
            binding::CNetstackCall_ping(
                req_ref,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            
            // Get the stored result
            NetstackResponse::get_last_response()
        }
    }
}
