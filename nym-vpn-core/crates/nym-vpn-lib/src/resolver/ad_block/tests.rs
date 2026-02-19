// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::files::*;
use std::path::Path;
use tempfile::TempDir;

const USER_AGENT: &str = "nym-vpn-ad-blocker-tests/0.1";

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
