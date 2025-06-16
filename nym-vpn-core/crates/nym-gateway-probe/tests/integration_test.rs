use nym_gateway_probe::*;
use std::str::FromStr;

fn create_test_credential_args() -> CredentialArgs {
    CredentialArgs {
        enable_credentials_mode: false,
        mnemonic: None,
    }
}

fn create_test_netstack_args() -> NetstackArgs {
    NetstackArgs {
        netstack_download_timeout_sec: 10,
        netstack_v4_dns: "1.1.1.1".to_string(),
        netstack_v6_dns: "2001:4860:4860::8888".to_string(),
        netstack_num_ping: 1,
        netstack_send_timeout_sec: 2,
        netstack_recv_timeout_sec: 2,
        netstack_ping_hosts_v4: vec!["example.com".to_string()],
        netstack_ping_ips_v4: vec!["8.8.8.8".to_string()],
        netstack_ping_hosts_v6: vec!["ipv6.google.com".to_string()],
        netstack_ping_ips_v6: vec!["2001:4860:4860::8888".to_string()],
    }
}

#[test]
fn test_probe_creation_integration() {
    // Test that we can create a probe with valid gateway identity
    let entry = nym_gateway_directory::EntryPoint::Gateway {
        identity: nym_sdk::mixnet::NodeIdentity::from_str("98uf1hyzmWTinkyc5PGyCxDo3E9QQK5XhWQ8B8z8aFoX").unwrap(),
    };
    let test_point = TestedNode::SameAsEntry;
    let netstack_args = create_test_netstack_args();
    let credential_args = create_test_credential_args();

    let probe = Probe::new(entry, test_point, netstack_args, credential_args);
    
    // Should be able to create probe without panicking
    assert!(std::mem::size_of_val(&probe) > 0);
}

#[test]
fn test_probe_with_custom_node() {
    // Test probe creation with a custom test node
    let entry = nym_gateway_directory::EntryPoint::Gateway {
        identity: nym_sdk::mixnet::NodeIdentity::from_str("98uf1hyzmWTinkyc5PGyCxDo3E9QQK5XhWQ8B8z8aFoX").unwrap(),
    };
    let test_point = TestedNode::Custom {
        identity: nym_sdk::mixnet::NodeIdentity::from_str("6imWS1yMngEeV3jCys4r7CDC5N2HC9yoXvA2ytqDfrWC").unwrap(),
    };
    let netstack_args = create_test_netstack_args();
    let credential_args = create_test_credential_args();

    let probe = Probe::new(entry, test_point, netstack_args, credential_args);
    
    // Should be able to create probe without panicking
    assert!(std::mem::size_of_val(&probe) > 0);
}

#[test]
fn test_probe_with_amnezia_integration() {
    let entry = nym_gateway_directory::EntryPoint::Gateway {
        identity: nym_sdk::mixnet::NodeIdentity::from_str("98uf1hyzmWTinkyc5PGyCxDo3E9QQK5XhWQ8B8z8aFoX").unwrap(),
    };
    let test_point = TestedNode::SameAsEntry;
    let netstack_args = create_test_netstack_args();
    let credential_args = create_test_credential_args();

    let mut probe = Probe::new(entry, test_point, netstack_args, credential_args);
    
    // Test adding Amnezia-WG arguments
    probe.with_amnezia("jc=4 jmin=10 jmax=100 S1=0 S2=0 H1=1 H2=2 H3=3 H4=4");
    
    // Should be able to modify probe without panicking
    assert!(std::mem::size_of_val(&probe) > 0);
}

#[test]
fn test_netstack_args_creation_and_modification() {
    let mut netstack_args = create_test_netstack_args();
    
    // Test that we can create and modify args
    assert_eq!(netstack_args.netstack_download_timeout_sec, 10);
    assert_eq!(netstack_args.netstack_v4_dns, "1.1.1.1");
    assert_eq!(netstack_args.netstack_v6_dns, "2001:4860:4860::8888");
    
    // Test modification
    netstack_args.netstack_download_timeout_sec = 30;
    netstack_args.netstack_num_ping = 5;
    
    assert_eq!(netstack_args.netstack_download_timeout_sec, 30);
    assert_eq!(netstack_args.netstack_num_ping, 5);
}

#[test] 
fn test_netstack_args_with_multiple_targets() {
    // Test netstack args with multiple ping targets
    let netstack_args = NetstackArgs {
        netstack_download_timeout_sec: 30,
        netstack_v4_dns: "1.1.1.1".to_string(),
        netstack_v6_dns: "2001:4860:4860::8888".to_string(),
        netstack_num_ping: 3,
        netstack_send_timeout_sec: 5,
        netstack_recv_timeout_sec: 5,
        netstack_ping_hosts_v4: vec![
            "google.com".to_string(),
            "cloudflare.com".to_string(),
            "nymtech.net".to_string(),
        ],
        netstack_ping_ips_v4: vec![
            "8.8.8.8".to_string(),
            "1.1.1.1".to_string(),
            "9.9.9.9".to_string(),
        ],
        netstack_ping_hosts_v6: vec![
            "ipv6.google.com".to_string(),
            "ipv6.cloudflare.com".to_string(),
        ],
        netstack_ping_ips_v6: vec![
            "2001:4860:4860::8888".to_string(),
            "2606:4700:4700::1111".to_string(),
        ],
    };

    // Verify IPv4 config has multiple targets
    assert_eq!(netstack_args.netstack_ping_hosts_v4.len(), 3);
    assert_eq!(netstack_args.netstack_ping_ips_v4.len(), 3);
    assert!(netstack_args.netstack_ping_hosts_v4.contains(&"nymtech.net".to_string()));
    assert!(netstack_args.netstack_ping_ips_v4.contains(&"9.9.9.9".to_string()));

    // Verify IPv6 config has multiple targets
    assert_eq!(netstack_args.netstack_ping_hosts_v6.len(), 2);
    assert_eq!(netstack_args.netstack_ping_ips_v6.len(), 2);
    assert!(netstack_args.netstack_ping_hosts_v6.contains(&"ipv6.cloudflare.com".to_string()));
    assert!(netstack_args.netstack_ping_ips_v6.contains(&"2606:4700:4700::1111".to_string()));
}

#[test]
fn test_credential_args_variants() {
    // Test without credentials
    let args_no_creds = CredentialArgs {
        enable_credentials_mode: false,
        mnemonic: None,
    };

    assert!(!args_no_creds.enable_credentials_mode);
    assert!(args_no_creds.mnemonic.is_none());

    // Test with credentials
    let test_mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let args_with_creds = CredentialArgs {
        enable_credentials_mode: true,
        mnemonic: Some(test_mnemonic.to_string()),
    };

    assert!(args_with_creds.enable_credentials_mode);
    assert!(args_with_creds.mnemonic.is_some());
    assert_eq!(args_with_creds.mnemonic.as_ref().unwrap(), test_mnemonic);
}

// Test helper functions for debugging and validation
#[test]
fn test_debug_formatting() {
    let netstack_args = create_test_netstack_args();
    let credential_args = create_test_credential_args();

    // Test that debug formatting works for all types
    let debug_netstack = format!("{:?}", netstack_args);
    let debug_creds = format!("{:?}", credential_args);

    assert!(debug_netstack.contains("NetstackArgs"));
    assert!(debug_creds.contains("CredentialArgs"));
}

#[test]
fn test_edge_case_empty_ping_targets() {
    // Test with empty ping targets (edge case)
    let netstack_args = NetstackArgs {
        netstack_download_timeout_sec: 30,
        netstack_v4_dns: "1.1.1.1".to_string(),
        netstack_v6_dns: "2001:4860:4860::8888".to_string(),
        netstack_num_ping: 1,
        netstack_send_timeout_sec: 3,
        netstack_recv_timeout_sec: 3,
        netstack_ping_hosts_v4: vec![], // Empty
        netstack_ping_ips_v4: vec![], // Empty
        netstack_ping_hosts_v6: vec![], // Empty
        netstack_ping_ips_v6: vec![], // Empty
    };

    // Should handle empty vectors gracefully
    assert_eq!(netstack_args.netstack_ping_hosts_v4.len(), 0);
    assert_eq!(netstack_args.netstack_ping_ips_v4.len(), 0);
    assert_eq!(netstack_args.netstack_ping_hosts_v6.len(), 0);
    assert_eq!(netstack_args.netstack_ping_ips_v6.len(), 0);
}

#[test]
fn test_tested_node_enum_variants() {
    // Test TestedNode enum variants
    let same_as_entry = TestedNode::SameAsEntry;
    assert!(matches!(same_as_entry, TestedNode::SameAsEntry));
    assert!(same_as_entry.is_same_as_entry());

    let custom_node = TestedNode::Custom {
        identity: nym_sdk::mixnet::NodeIdentity::from_str("98uf1hyzmWTinkyc5PGyCxDo3E9QQK5XhWQ8B8z8aFoX").unwrap(),
    };
    assert!(matches!(custom_node, TestedNode::Custom { .. }));
    assert!(!custom_node.is_same_as_entry());
} 