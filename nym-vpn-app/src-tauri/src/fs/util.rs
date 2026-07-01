use crate::APP_DIR;
use anyhow::{Context, Result};
use itertools::Itertools;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use tracing::{debug, error};

// `identifier` property in tauri.config.json
const APP_ID: &str = "net.nymtech.vpn";

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

/// Remove all app local files
pub fn clean_local_files() {
    let paths = [
        dirs::config_dir().map(|mut p| {
            p.push(APP_DIR);
            p
        }),
        dirs::config_dir().map(|mut p| {
            p.push(APP_ID);
            p
        }),
        dirs::data_dir().map(|mut p| {
            p.push(APP_DIR);
            p
        }),
        dirs::data_dir().map(|mut p| {
            p.push(APP_ID);
            p
        }),
        dirs::cache_dir().map(|mut p| {
            p.push(APP_DIR);
            p
        }),
        dirs::cache_dir().map(|mut p| {
            p.push(APP_ID);
            p
        }),
        #[cfg(target_os = "linux")]
        dirs::state_dir().map(|mut p| {
            p.push(APP_DIR);
            p
        }),
        #[cfg(target_os = "linux")]
        dirs::state_dir().map(|mut p| {
            p.push(APP_ID);
            p
        }),
    ];

    for path in paths.iter().flatten().unique() {
        if path.exists() {
            fs::remove_dir_all(path)
                .inspect_err(|e| eprintln!("failed to remove {}: {e}", path.display()))
                .inspect(|_| println!("removed: {}", path.display()))
                .ok();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    // Builds a unique path under the OS temp dir so parallel test runs don't collide.
    fn unique_temp_path(suffix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let mut path = std::env::temp_dir();
        path.push(format!("nym-vpn-app-test-{nanos}-{suffix}"));
        path
    }

    #[test]
    fn check_dir_creates_a_missing_directory() {
        let dir = unique_temp_path("dir");
        assert!(!dir.is_dir());
        check_dir(&dir).expect("check_dir should create the directory");
        assert!(dir.is_dir());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_dir_is_ok_for_an_existing_directory() {
        let dir = unique_temp_path("existing");
        fs::create_dir_all(&dir).unwrap();
        assert!(check_dir(&dir).is_ok());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_file_creates_a_missing_file() {
        let dir = unique_temp_path("filedir");
        fs::create_dir_all(&dir).unwrap();
        let mut file = dir.clone();
        file.push("marker.txt");
        assert!(!file.exists());
        check_file(&file).expect("check_file should create the file");
        assert!(file.exists());
        let _ = fs::remove_dir_all(&dir);
    }
}
