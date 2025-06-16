use super::NetstackRequest;

pub mod binding {
    #![allow(warnings)]
    rust2go::r2g_include_binding!();
}

// Types for the netstack ping functionality
// Note: These are kept minimal since we use direct C function calls to avoid callback issues

#[derive(Clone, Debug, PartialEq)]
pub struct NetstackRequestGo {
    pub wg_ip: String,
    pub private_key: String,
    pub public_key: String,
    pub endpoint: String,
    pub dns: String,
    pub ip_version: u8,
    pub ping_hosts: Vec<String>,
    pub ping_ips: Vec<String>,
    pub num_ping: u8,
    pub send_timeout_sec: u64,
    pub recv_timeout_sec: u64,
    pub download_timeout_sec: u64,
    pub awg_args: String,
}

#[derive(Clone, Debug, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_netstack_request() -> NetstackRequest {
        use crate::NetstackArgs;
        let args = NetstackArgs {
            netstack_download_timeout_sec: 30,
            netstack_v4_dns: "1.1.1.1".to_string(),
            netstack_v6_dns: "2001:4860:4860::8888".to_string(),
            netstack_num_ping: 3,
            netstack_send_timeout_sec: 5,
            netstack_recv_timeout_sec: 5,
            netstack_ping_hosts_v4: vec!["example.com".to_string()],
            netstack_ping_ips_v4: vec!["8.8.8.8".to_string()],
            netstack_ping_hosts_v6: vec!["ipv6.google.com".to_string()],
            netstack_ping_ips_v6: vec!["2001:4860:4860::8888".to_string()],
        };
        
        NetstackRequest::new(
            "10.0.0.1",
            "fc00::1",
            "test_private_key",
            "test_public_key",
            "192.168.1.1:51820",
            30,
            "",
            args,
        )
    }

    #[test]
    fn test_netstack_request_go_from_rust_v4() {
        let request = create_test_netstack_request();
        let go_request = NetstackRequestGo::from_rust_v4(&request);

        assert_eq!(go_request.wg_ip, "10.0.0.1");
        assert_eq!(go_request.private_key, "test_private_key");
        assert_eq!(go_request.public_key, "test_public_key");
        assert_eq!(go_request.endpoint, "192.168.1.1:51820");
        assert_eq!(go_request.dns, "1.1.1.1");
        assert_eq!(go_request.ip_version, 4);
        assert_eq!(go_request.ping_hosts, vec!["example.com"]);
        assert_eq!(go_request.ping_ips, vec!["8.8.8.8"]);
        assert_eq!(go_request.num_ping, 3);
        assert_eq!(go_request.send_timeout_sec, 5);
        assert_eq!(go_request.recv_timeout_sec, 5);
        assert_eq!(go_request.download_timeout_sec, 30);
        assert_eq!(go_request.awg_args, "");
    }

    #[test]
    fn test_netstack_request_go_from_rust_v6() {
        let request = create_test_netstack_request();
        let go_request = NetstackRequestGo::from_rust_v6(&request);

        assert_eq!(go_request.wg_ip, "fc00::1");
        assert_eq!(go_request.private_key, "test_private_key");
        assert_eq!(go_request.public_key, "test_public_key");
        assert_eq!(go_request.endpoint, "192.168.1.1:51820");
        assert_eq!(go_request.dns, "2001:4860:4860::8888");
        assert_eq!(go_request.ip_version, 6);
        assert_eq!(go_request.ping_hosts, vec!["ipv6.google.com"]);
        assert_eq!(go_request.ping_ips, vec!["2001:4860:4860::8888"]);
        assert_eq!(go_request.num_ping, 3);
        assert_eq!(go_request.send_timeout_sec, 5);
        assert_eq!(go_request.recv_timeout_sec, 5);
        assert_eq!(go_request.download_timeout_sec, 30);
        assert_eq!(go_request.awg_args, "");
    }

    #[test]
    fn test_netstack_response_creation() {
        let response = NetstackResponse {
            can_handshake: true,
            sent_ips: 5,
            received_ips: 4,
            sent_hosts: 3,
            received_hosts: 2,
            can_resolve_dns: true,
            downloaded_file: "https://example.com/test.dat".to_string(),
            download_duration_sec: 10,
            download_error: "".to_string(),
        };

        assert!(response.can_handshake);
        assert_eq!(response.sent_ips, 5);
        assert_eq!(response.received_ips, 4);
        assert_eq!(response.sent_hosts, 3);
        assert_eq!(response.received_hosts, 2);
        assert!(response.can_resolve_dns);
        assert_eq!(response.downloaded_file, "https://example.com/test.dat");
        assert_eq!(response.download_duration_sec, 10);
        assert_eq!(response.download_error, "");
    }

    #[test]
    fn test_netstack_request_go_with_amnezia_args() {
        let mut request = create_test_netstack_request();
        request.awg_args = "jc=4 jmin=10 jmax=100".to_string();
        
        let go_request = NetstackRequestGo::from_rust_v4(&request);
        assert_eq!(go_request.awg_args, "jc=4 jmin=10 jmax=100");
    }

    #[test]
    fn test_netstack_request_go_with_multiple_hosts_and_ips() {
        let mut request = create_test_netstack_request();
        request.v4_ping_config.ping_hosts = vec![
            "example.com".to_string(),
            "google.com".to_string(),
            "cloudflare.com".to_string(),
        ];
        request.v4_ping_config.ping_ips = vec![
            "8.8.8.8".to_string(),
            "1.1.1.1".to_string(),
            "9.9.9.9".to_string(),
        ];
        
        let go_request = NetstackRequestGo::from_rust_v4(&request);
        assert_eq!(go_request.ping_hosts.len(), 3);
        assert_eq!(go_request.ping_ips.len(), 3);
        assert!(go_request.ping_hosts.contains(&"google.com".to_string()));
        assert!(go_request.ping_ips.contains(&"1.1.1.1".to_string()));
    }

    #[test]
    fn test_netstack_request_go_debug_format() {
        let request = create_test_netstack_request();
        let go_request = NetstackRequestGo::from_rust_v4(&request);
        
        let debug_str = format!("{:?}", go_request);
        assert!(debug_str.contains("NetstackRequestGo"));
        assert!(debug_str.contains("10.0.0.1"));
        assert!(debug_str.contains("test_private_key"));
    }

    #[test]
    fn test_netstack_response_debug_format() {
        let response = NetstackResponse {
            can_handshake: true,
            sent_ips: 5,
            received_ips: 4,
            sent_hosts: 3,
            received_hosts: 2,
            can_resolve_dns: true,
            downloaded_file: "test.dat".to_string(),
            download_duration_sec: 10,
            download_error: "".to_string(),
        };
        
        let debug_str = format!("{:?}", response);
        assert!(debug_str.contains("NetstackResponse"));
        assert!(debug_str.contains("can_handshake: true"));
        assert!(debug_str.contains("sent_ips: 5"));
    }

    #[test]
    fn test_netstack_response_equality() {
        let response1 = NetstackResponse {
            can_handshake: true,
            sent_ips: 5,
            received_ips: 4,
            sent_hosts: 3,
            received_hosts: 2,
            can_resolve_dns: true,
            downloaded_file: "test.dat".to_string(),
            download_duration_sec: 10,
            download_error: "".to_string(),
        };

        let response2 = NetstackResponse {
            can_handshake: true,
            sent_ips: 5,
            received_ips: 4,
            sent_hosts: 3,
            received_hosts: 2,
            can_resolve_dns: true,
            downloaded_file: "test.dat".to_string(),
            download_duration_sec: 10,
            download_error: "".to_string(),
        };

        let response3 = NetstackResponse {
            can_handshake: false, // Different value
            sent_ips: 5,
            received_ips: 4,
            sent_hosts: 3,
            received_hosts: 2,
            can_resolve_dns: true,
            downloaded_file: "test.dat".to_string(),
            download_duration_sec: 10,
            download_error: "".to_string(),
        };

        assert_eq!(response1, response2);
        assert_ne!(response1, response3);
    }
}
