use super::files::*;
use tempfile::TempDir;

use std::sync::Once;

static TRACING_INIT: Once = Once::new();

fn init_tracing() {
    TRACING_INIT.call_once(|| {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,nym_vpn_lib=trace")),
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

    for descr in AD_BLOCKING_LIST_DESCRIPTORS.iter() {
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
#[ignore]   // This test is not practical as the easylist_adservers.txt file changes very frequently
async fn test_update_nothing() {
    let temp_dir = init_tests()
        .await
        .expect("Failed to initialize ad-blocking files");
    let data_dir = temp_dir.path();

    let updated = update_files(data_dir)
        .await
        .expect("Failed to update ad-blocking files");

    assert!(
        !updated,
        "ad-blocking files were unexpectedly updated when they should have been up-to-date"
    );
}

#[tokio::test]
async fn test_update_0() {
    update_descr_at_index(0)
        .await
        .expect("Failed to update ad-blocking files for descriptor 0");
}

#[tokio::test]
async fn test_update_1() {
    update_descr_at_index(1)
        .await
        .expect("Failed to update ad-blocking files for descriptor 1");
}

async fn init_tests() -> Result<TempDir, String> {
    init_tracing();

    let temp_dir =
        tempfile::tempdir().map_err(|e| format!("failed to create temporary directory: {e}"))?;
    let data_dir = temp_dir.path();

    init_files(data_dir, false)
        .await
        .map_err(|e| format!("failed to create initial ad-blocking files: {e}"))?;

    Ok(temp_dir)
}

async fn update_descr_at_index(index: usize) -> Result<(), String> {
    let temp_dir = init_tests().await?;
    let data_dir = temp_dir.path();
    let ad_blocking_path = get_ad_blocking_path(data_dir);

    // Open the meta file and change the etag in order to force an update of the data file
    let meta_path = ad_blocking_path.join(AD_BLOCKING_LIST_DESCRIPTORS[index].meta_file_name);
    let mut meta_data = AdBlockListMeta::from_file(&meta_path)
        .await
        .map_err(|e| format!("failed to read ad-blocking meta file: {e}"))?;
    meta_data.etag = "fake-etag".to_string();
    meta_data
        .write_to_file(&meta_path)
        .await
        .map_err(|e| format!("failed to write ad-blocking meta file: {e}"))?;

    let updated = update_files(data_dir)
        .await
        .expect("Failed to update ad-blocking files");

    if updated {
        Ok(())
    } else {
        Err("ad-blocking files were not updated when they should have been".to_string())
    }
}
