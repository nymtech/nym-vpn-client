// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::{files::*, task::AdBlockerTask};
use crate::resolver::DnsFilterDecision;
use std::{path::Path, time::Duration};
use tempfile::TempDir;
use tokio::time::{Instant, sleep};
use tokio_util::sync::CancellationToken;

const USER_AGENT: &str = "nym-vpn-ad-blocker-tests/0.1";
const SHOULD_BE_BLOCKED_DOMAIN: &str = "www.0.beer";

#[tokio::test]
#[tracing_test::traced_test]
async fn test_init_files() {
    let temp_dir = init_tests()
        .await
        .expect("Failed to initialize ad-blocker files");
    let data_dir = temp_dir.path();

    for descr in SOURCES.iter() {
        let file_path = get_ad_blocking_path(data_dir).join(descr.file_name);
        assert!(
            file_path.exists(),
            "ad-blocker data file {} was not created",
            file_path.display()
        );

        let meta_file_path = get_ad_blocking_path(data_dir).join(descr.meta_file_name);
        assert!(
            meta_file_path.exists(),
            "ad-blocker meta file {} was not created",
            meta_file_path.display()
        );
    }
}

#[tokio::test]
#[tracing_test::traced_test]
#[ignore] // This test is not practical as the easylist_adservers.txt file changes very frequently
async fn test_update_nothing() {
    let temp_dir = init_tests()
        .await
        .expect("Failed to initialize ad-blocker files");
    let data_dir = temp_dir.path();

    let updated = update_files(data_dir, USER_AGENT)
        .await
        .expect("Failed to update ad-blocker files");

    assert!(
        !updated,
        "ad-blocker files were updated when they should not have been"
    );
}

#[tokio::test]
#[tracing_test::traced_test]
async fn test_update_0() {
    let temp_dir = init_tests()
        .await
        .expect("Failed to initialize ad-blocker files");
    let data_dir = temp_dir.path();
    let ad_blocking_path = get_ad_blocking_path(data_dir);

    write_fake_etag(&ad_blocking_path, 0)
        .await
        .expect("Failed to update ad-blocker metadata 1");

    let updated = update_files(data_dir, USER_AGENT)
        .await
        .expect("Failed to update ad-blocker files");

    assert!(
        updated,
        "ad-blocker files were not updated when they should have been"
    );
}

#[tokio::test]
#[tracing_test::traced_test]
async fn test_update_1() {
    let temp_dir = init_tests()
        .await
        .expect("Failed to initialize ad-blocker files");
    let data_dir = temp_dir.path();
    let ad_blocking_path = get_ad_blocking_path(data_dir);

    write_fake_etag(&ad_blocking_path, 1)
        .await
        .expect("Failed to update ad-blocker metadata 0");

    let updated = update_files(data_dir, USER_AGENT)
        .await
        .expect("Failed to update ad-blocker files");

    assert!(
        updated,
        "ad-blocker files were not updated when they should have been"
    );
}

#[tokio::test]
#[tracing_test::traced_test]
async fn test_load_filterset_default() {
    let temp_dir = init_tests()
        .await
        .expect("Failed to initialize ad-blocker files");
    let data_dir = temp_dir.path();

    let _filter_set = load_filter_set(data_dir)
        .await
        .expect("Failed to load filter set from ad-blocker files");
}

#[tokio::test]
#[tracing_test::traced_test]
async fn test_load_filterset_updated() {
    let temp_dir = init_tests()
        .await
        .expect("Failed to initialize ad-blocker files");
    let data_dir = temp_dir.path();
    let ad_blocking_path = get_ad_blocking_path(data_dir);

    write_fake_etag(&ad_blocking_path, 0)
        .await
        .expect("Failed to update ad-blocker metadata 0");
    write_fake_etag(&ad_blocking_path, 1)
        .await
        .expect("Failed to update ad-blocker metadata 1");

    let _updated = update_files(data_dir, USER_AGENT)
        .await
        .expect("Failed to update ad-blocker files");

    let _filter_set = load_filter_set(data_dir)
        .await
        .expect("Failed to load filter set from ad-blocker files");
}

#[tokio::test]
#[tracing_test::traced_test]
async fn test_task_allows_domains_before_init() {
    let temp_dir = init_tests()
        .await
        .expect("Failed to initialize ad-blocker files");
    let data_dir = temp_dir.path();

    let shutdown_token = CancellationToken::new();
    let (task_handle, join_handle) = AdBlockerTask::spawn(
        data_dir,
        USER_AGENT.to_string(),
        shutdown_token.child_token(),
    )
    .await
    .expect("Failed to spawn AdBlockerTask");

    let dns_filter = task_handle
        .get_dns_filter()
        .await
        .expect("Expected DNS filter");

    // AdBlocker::default() should not block anything.
    let decision = {
        let guard = dns_filter.lock().await;
        guard.should_block(SHOULD_BE_BLOCKED_DOMAIN)
    };

    assert!(matches!(decision, DnsFilterDecision::Pass));

    shutdown_token.cancel();
    let _ = join_handle.await;
}

#[tokio::test]
#[tracing_test::traced_test]
async fn test_task_blocks_domain_after_init() {
    let temp_dir = init_tests()
        .await
        .expect("Failed to initialize ad-blocker files");
    let data_dir = temp_dir.path();

    let shutdown_token = CancellationToken::new();
    let (task_handle, join_handle) = AdBlockerTask::spawn(
        data_dir,
        USER_AGENT.to_string(),
        shutdown_token.child_token(),
    )
    .await
    .expect("Failed to spawn AdBlockerTask");

    let dns_filter = task_handle
        .get_dns_filter()
        .await
        .expect("Expected DNS filter");

    // Kick off initialization.
    task_handle.init_ad_blocker().await;

    // Wait for initialization to complete.
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if task_handle.is_ad_blocker_initted().await {
            break;
        }
        if Instant::now() > deadline {
            panic!("Timed out waiting for ad-blocker initialization");
        }
        sleep(Duration::from_millis(50)).await;
    }

    // After init, it should block this known ad-blocked domain.
    let decision = {
        let guard = dns_filter.lock().await;
        guard.should_block(SHOULD_BE_BLOCKED_DOMAIN)
    };

    assert!(matches!(decision, DnsFilterDecision::Block(_)));

    shutdown_token.cancel();
    let _ = join_handle.await;
}

async fn init_tests() -> Result<TempDir, String> {
    let temp_dir =
        tempfile::tempdir().map_err(|e| format!("failed to create temporary directory: {e}"))?;
    let data_dir = temp_dir.path();

    init_files(data_dir, false)
        .await
        .map_err(|e| format!("failed to create initial ad-blocker files: {e}"))?;

    Ok(temp_dir)
}

// Open the meta file and change the etag in order to force an update of the data file
async fn write_fake_etag(ad_blocking_path: &Path, index: usize) -> Result<(), String> {
    let meta_path = ad_blocking_path.join(SOURCES[index].meta_file_name);
    let mut meta_data = SourceMetaData::from_file(&meta_path)
        .await
        .map_err(|e| format!("failed to read ad-blocker meta file: {e}"))?;
    meta_data.etag = "fake-etag".to_string();
    meta_data
        .write_to_file(&meta_path)
        .await
        .map_err(|e| format!("failed to write ad-blocker meta file: {e}"))?;
    Ok(())
}
