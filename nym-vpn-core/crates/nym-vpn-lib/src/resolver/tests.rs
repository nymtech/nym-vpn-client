// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::SocketAddr;

use super::*;
use hickory_server::resolver::{
    TokioResolver,
    config::{NameServerConfigGroup, ResolverConfig},
    name_server::TokioConnectionProvider,
};
use tokio_util::sync::CancellationToken;

/// Test whether we can successfully bind the socket even if the address is already used in
/// different scenarios.
#[tokio::test]
#[serial_test::serial]
async fn test_bind() {
    // Bind a wildcard socket to create potential collisions.
    let _sock = std::net::UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], DNS_LISTEN_PORT)))
        .expect("failed to bind wildcard port");

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

    let shutdown_token = CancellationToken::new();
    let (handle, join_handle) =
        LocalResolver::spawn(temp_dir.path(), false, shutdown_token.child_token())
            .await
            .unwrap();

    let test_resolver = get_test_resolver(handle.listen_addr());
    test_resolver
        .lookup(&ALLOWED_DOMAINS[0], RecordType::A)
        .await
        .expect("lookup should succeed");

    drop(_sock);
    shutdown_token.cancel();
    join_handle.await.unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;

    // bind() succeeds if wildcard address is bound with SO_REUSEADDR etc is platform specific; we
    // just ensure we can start/stop cleanly.
}

#[tokio::test]
#[serial_test::serial]
async fn test_successful_lookup() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let shutdown_token = CancellationToken::new();
    let (handle, join_handle) =
        LocalResolver::spawn(temp_dir.path(), false, shutdown_token.child_token())
            .await
            .unwrap();
    let test_resolver = get_test_resolver(handle.listen_addr());

    for domain in &*ALLOWED_DOMAINS {
        test_resolver
            .lookup(domain, RecordType::A)
            .await
            .expect("domain resolution failed");
    }

    shutdown_token.cancel();
    join_handle.await.unwrap();
}

#[tokio::test]
#[serial_test::serial]
async fn test_failed_lookup() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let shutdown_token = CancellationToken::new();
    let (handle, join_handle) =
        LocalResolver::spawn(temp_dir.path(), false, shutdown_token.child_token())
            .await
            .unwrap();
    let test_resolver = get_test_resolver(handle.listen_addr());

    let captive_portal_domain = LowerName::from(Name::from_str("apple.com").unwrap());
    assert!(
        test_resolver
            .lookup(captive_portal_domain, RecordType::A)
            .await
            .is_err(),
        "Non-whitelisted DNS request should fail"
    );
    shutdown_token.cancel();
    join_handle.await.unwrap();
}

/// Test that we close the socket when shutting down the local resolver.
#[tokio::test]
#[serial_test::serial]
async fn test_unbind_socket_on_stop() {
    // Bind resolver to 127.0.0.1 so we can easily bind to the same address here.
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let shutdown_token = CancellationToken::new();
    let (handle, join_handle) =
        LocalResolver::spawn(temp_dir.path(), false, shutdown_token.child_token())
            .await
            .unwrap();
    let addr = handle.listen_addr();
    assert_eq!(
        addr,
        SocketAddr::from((Ipv4Addr::LOCALHOST, DNS_LISTEN_PORT))
    );
    shutdown_token.cancel();
    join_handle.await.unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;
    std::net::UdpSocket::bind(addr).expect("Failed to bind to a port that should have been freed");
}

fn get_test_resolver(listen_addr: SocketAddr) -> TokioResolver {
    let resolver_config = ResolverConfig::from_parts(
        None,
        vec![],
        NameServerConfigGroup::from_ips_clear(&[listen_addr.ip()], listen_addr.port(), true),
    );
    TokioResolver::builder_with_config(resolver_config, TokioConnectionProvider::default()).build()
}
