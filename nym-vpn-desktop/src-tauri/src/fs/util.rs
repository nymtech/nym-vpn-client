use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::fs;
use tracing::debug;

/// Check if a directory exists, if not create it including all
/// parent components
pub async fn check_dir(path: &PathBuf) -> Result<()> {
    if !fs::try_exists(&path).await.context(format!(
        "Failed to check if directory exists {}",
        path.display()
    ))? {
        debug!("directory {} does not exist, creating it", path.display());
        return fs::create_dir_all(&path).await.context(format!(
            "Failed to create data directory {}",
            path.display()
        ));
    }
    Ok(())
}
