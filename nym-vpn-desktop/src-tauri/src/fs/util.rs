use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::fs::{self, File};
use tracing::{debug, error};

/// Check if a directory exists, if not create it including all
/// parent components
pub async fn check_dir(path: &PathBuf) -> Result<()> {
    if !fs::try_exists(&path)
        .await
        .inspect_err(|e| error!("Failed to check if path exists `{}`: {e}", path.display()))
        .context(format!(
            "Failed to check if path exists `{}`",
            path.display()
        ))?
    {
        debug!("directory `{}` does not exist, creating it", path.display());
        return fs::create_dir_all(&path)
            .await
            .inspect_err(|e| error!("Failed to create directory `{}`: {e}", path.display()))
            .context(format!("Failed to create directory `{}`", path.display()));
    }
    Ok(())
}

/// Check if a file exists, if not create it
pub async fn check_file(path: &PathBuf) -> Result<()> {
    if !fs::try_exists(&path)
        .await
        .inspect_err(|e| error!("Failed to check if path exists `{}`: {e}", path.display()))
        .context(format!(
            "Failed to check if path exists `{}`",
            path.display()
        ))?
    {
        debug!("file `{}` does not exist, creating it", path.display());
        File::create(&path)
            .await
            .inspect_err(|e| error!("Failed to create file `{}`: {e}", path.display()))
            .context(format!("Failed to create file `{}`", path.display()))?;
        return Ok(());
    }
    Ok(())
}
