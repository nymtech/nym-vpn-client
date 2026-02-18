use super::files::*;
use std::path::Path;
use tempfile::TempDir;

use std::sync::Once;
use time::Duration;

const INIT_TRACING: bool = false;
static TRACING_INIT: Once = Once::new();

const DEFAULT_EXPIRED_DURATION: Duration = Duration::seconds(10);

#[allow(dead_code)]
fn init_tracing() {
    TRACING_INIT.call_once(|| {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                    tracing_subscriber::EnvFilter::new("info,nym_vpn_lib=trace")
                }),
            )
            .with_test_writer()
            .compact()
            .try_init();
    });
}

#[tokio::test]
async fn test_init_files() {
    let temp_dir = init_tests()
        .await
        .expect("Failed to initialize ad-blocking files");
    let data_dir = temp_dir.path();

    for descr in SOURCES.iter() {
        let file_path = get_ad_blocking_path(data_dir).join(descr.file_name);
        assert!(
            file_path.exists(),
            "ad-blocking data file {} was not created",
            file_path.display()
        );

        let meta_file_path = get_ad_blocking_path(data_dir).join(descr.meta_file_name);
        assert!(
            meta_file_path.exists(),
            "ad-blocking meta file {} was not created",
            meta_file_path.display()
        );
    }
}

#[tokio::test]
#[ignore] // This test is not practical as the easylist_adservers.txt file changes very frequently
async fn test_update_nothing() {
    let temp_dir = init_tests()
        .await
        .expect("Failed to initialize ad-blocking files");
    let data_dir = temp_dir.path();

    let updated = update_files(data_dir, DEFAULT_EXPIRED_DURATION)
        .await
        .expect("Failed to update ad-blocking files");

    assert!(
        !updated,
        "ad-blocking files were updated when they should not have been"
    );
}

#[tokio::test]
async fn test_update_0() {
    let temp_dir = init_tests()
        .await
        .expect("Failed to initialize ad-blocking files");
    let data_dir = temp_dir.path();
    let ad_blocking_path = get_ad_blocking_path(data_dir);

    write_fake_etag(&ad_blocking_path, 0)
        .await
        .expect("Failed to update ad-blocking metadata 1");

    let updated = update_files(data_dir, DEFAULT_EXPIRED_DURATION)
        .await
        .expect("Failed to update ad-blocking files");

    assert!(
        updated,
        "ad-blocking files were not updated when they should have been"
    );
}

#[tokio::test]
async fn test_update_1() {
    let temp_dir = init_tests()
        .await
        .expect("Failed to initialize ad-blocking files");
    let data_dir = temp_dir.path();
    let ad_blocking_path = get_ad_blocking_path(data_dir);

    write_fake_etag(&ad_blocking_path, 1)
        .await
        .expect("Failed to update ad-blocking metadata 0");

    let updated = update_files(data_dir, DEFAULT_EXPIRED_DURATION)
        .await
        .expect("Failed to update ad-blocking files");

    assert!(
        updated,
        "ad-blocking files were not updated when they should have been"
    );
}

#[tokio::test]
async fn test_update_expired() {
    let temp_dir = init_tests()
        .await
        .expect("Failed to initialize ad-blocking files");
    let data_dir = temp_dir.path();
    let ad_blocking_path = get_ad_blocking_path(data_dir);

    // We need to force both data files to be updated
    write_fake_etag(&ad_blocking_path, 0)
        .await
        .expect("Failed to update ad-blocking metadata 0");
    write_fake_etag(&ad_blocking_path, 1)
        .await
        .expect("Failed to update ad-blocking metadata 1");

    let updated = update_files(data_dir, Duration::seconds(10000))
        .await
        .expect("Failed to update ad-blocking files");

    assert!(
        updated,
        "ad-blocking files were not updated when they should have been"
    );

    // We need to force both data files to be *potentially* updated,
    // however they shouldn't be as their update time will be less than
    // 10 seconds ago.
    write_fake_etag(&ad_blocking_path, 0)
        .await
        .expect("Failed to update ad-blocking metadata 0");
    write_fake_etag(&ad_blocking_path, 1)
        .await
        .expect("Failed to update ad-blocking metadata 1");

    let updated_again = update_files(data_dir, Duration::seconds(10))
        .await
        .expect("Failed to update ad-blocking files");

    assert!(
        !updated_again,
        "ad-blocking files were updated when they not should have been"
    );
}

#[tokio::test]
async fn test_load_filterset_default() {
    let temp_dir = init_tests()
        .await
        .expect("Failed to initialize ad-blocking files");
    let data_dir = temp_dir.path();

    let _filter_set = load_filter_set(data_dir)
        .await
        .expect("Failed to load filter set from ad-blocking files");
}

#[tokio::test]
async fn test_load_filterset_updated() {
    let temp_dir = init_tests()
        .await
        .expect("Failed to initialize ad-blocking files");
    let data_dir = temp_dir.path();
    let ad_blocking_path = get_ad_blocking_path(data_dir);

    write_fake_etag(&ad_blocking_path, 0)
        .await
        .expect("Failed to update ad-blocking metadata 0");
    write_fake_etag(&ad_blocking_path, 1)
        .await
        .expect("Failed to update ad-blocking metadata 1");

    let _updated = update_files(data_dir, DEFAULT_EXPIRED_DURATION)
        .await
        .expect("Failed to update ad-blocking files");

    let _filter_set = load_filter_set(data_dir)
        .await
        .expect("Failed to load filter set from ad-blocking files");
}

async fn init_tests() -> Result<TempDir, String> {
    if INIT_TRACING {
        init_tracing();
    }

    let temp_dir =
        tempfile::tempdir().map_err(|e| format!("failed to create temporary directory: {e}"))?;
    let data_dir = temp_dir.path();

    init_files(data_dir, false)
        .await
        .map_err(|e| format!("failed to create initial ad-blocking files: {e}"))?;

    Ok(temp_dir)
}

// Open the meta file and change the etag in order to force an update of the data file
async fn write_fake_etag(ad_blocking_path: &Path, index: usize) -> Result<(), String> {
    let meta_path = ad_blocking_path.join(SOURCES[index].meta_file_name);
    let mut meta_data = SourceMetaData::from_file(&meta_path)
        .await
        .map_err(|e| format!("failed to read ad-blocking meta file: {e}"))?;
    meta_data.etag = "fake-etag".to_string();
    meta_data
        .write_to_file(&meta_path)
        .await
        .map_err(|e| format!("failed to write ad-blocking meta file: {e}"))?;
    Ok(())
}
