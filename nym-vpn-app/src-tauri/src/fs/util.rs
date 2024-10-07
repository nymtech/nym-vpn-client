use anyhow::{Context, Result};
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use tracing::{debug, error};

/// Check if a directory exists, if not create it including all
/// parent components
pub fn check_dir(path: &PathBuf) -> Result<()> {
    if !path.is_dir() {
        debug!("directory `{}` does not exist, creating it", path.display());
        return fs::create_dir_all(path)
            .inspect_err(|e| error!("Failed to create directory `{}`: {e}", path.display()))
            .context(format!("Failed to create directory `{}`", path.display()));
    }
    Ok(())
}

/// Check if a file exists, if not create it
pub fn check_file(path: &PathBuf) -> Result<()> {
    if !Path::try_exists(path)
        .inspect_err(|e| error!("Failed to check if path exists `{}`: {e}", path.display()))
        .context(format!(
            "Failed to check if path exists `{}`",
            path.display()
        ))?
    {
        debug!("file `{}` does not exist, creating it", path.display());
        File::create(path)
            .inspect_err(|e| error!("Failed to create file `{}`: {e}", path.display()))
            .context(format!("Failed to create file `{}`", path.display()))?;
    }
    Ok(())
}
