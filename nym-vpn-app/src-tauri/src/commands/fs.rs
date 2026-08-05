use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;
use tracing::{debug, error, info, instrument, warn};
use walkdir::WalkDir;
use zip::ZipWriter;
use zip::write::FileOptions;

use crate::commands::daemon::DEFAULT_VPND_LOG_DIR;
use crate::error::BackendError;
use crate::fs::path::APP_LOG_DIR;
use crate::state::SharedAppState;
use crate::vpnd::client::{VpndClient, VpndStatus};

const ZIP_APP_LOGS_PREFIX: &str = "app-logs";
const ZIP_VPND_LOGS_PREFIX: &str = "vpnd-logs";

#[instrument]
#[tauri::command]
pub async fn log_dir() -> Result<String, BackendError> {
    let log_path = APP_LOG_DIR.clone().ok_or_else(|| {
        let err = "Failed to get log directory path";
        error!(err);
        BackendError::internal(err, None)
    })?;
    let log_dir = log_path.to_str().ok_or_else(|| {
        let err = "Failed to get log directory path";
        error!(err);
        BackendError::internal(err, None)
    })?;

    debug!("log directory: {}", log_dir);
    Ok(log_dir.into())
}

#[instrument]
#[tauri::command]
pub async fn delete_app_logs() -> Result<(), BackendError> {
    let log_path = APP_LOG_DIR.clone().ok_or_else(|| {
        let err = "Failed to get log directory path";
        error!(err);
        BackendError::internal(err, None)
    })?;

    debug!("deleting all contents of log directory: {:?}", log_path);

    let entries = fs::read_dir(&log_path).map_err(|e| {
        let err = format!("Failed to read log directory: {}", e);
        error!(err);
        BackendError::internal(&err, None)
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            let err = format!("Failed to read directory entry: {}", e);
            error!(err);
            BackendError::internal(&err, None)
        })?;

        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(&path).map_err(|e| {
                let err = format!("Failed to remove directory {:?}: {}", path, e);
                error!(err);
                BackendError::internal(&err, None)
            })?;
        } else {
            fs::remove_file(&path).map_err(|e| {
                let err = format!("Failed to remove file {:?}: {}", path, e);
                error!(err);
                BackendError::internal(&err, None)
            })?;
        }
    }

    info!("successfully deleted all contents of log directory");
    Ok(())
}

/// Recursively adds all files from a directory to a zip archive.
///
/// Files are added under the specified `prefix` directory within the archive.
/// Uses forward slashes for archive paths to ensure cross-platform compatibility.
fn add_directory_to_zip<W: Write + io::Seek>(
    zip: &mut ZipWriter<W>,
    dir_path: &Path,
    prefix: &str,
) -> io::Result<()> {
    let options = FileOptions::<()>::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    for entry in WalkDir::new(dir_path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
    {
        let path = entry.path();
        let relative_path = path
            .strip_prefix(dir_path)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Use forward slashes for cross-platform zip compatibility
        let archive_path = format!(
            "{}/{}",
            prefix,
            relative_path.to_string_lossy().replace('\\', "/")
        );

        zip.start_file(&archive_path, options)?;
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        io::copy(&mut reader, zip)?;
    }

    Ok(())
}

/// Creates a zip archive containing logs from both the app and vpnd, plus an
/// optional extra file (e.g. a diagnostic report) added at the root of the archive.
///
/// Returns `true` if the archive was created successfully, `false` if the user
/// cancelled the save dialog.
fn create_logs_archive(
    output_path: &Path,
    app_log_dir: &Path,
    vpnd_log_dir: &Path,
    extra_file: Option<(String, Vec<u8>)>,
) -> io::Result<()> {
    let file = File::create(output_path)?;
    let writer = BufWriter::new(file);
    let mut zip = ZipWriter::new(writer);

    if vpnd_log_dir.exists() {
        add_directory_to_zip(&mut zip, vpnd_log_dir, ZIP_VPND_LOGS_PREFIX)?;
    }

    if app_log_dir.exists() {
        add_directory_to_zip(&mut zip, app_log_dir, ZIP_APP_LOGS_PREFIX)?;
    }

    if let Some((name, contents)) = extra_file {
        let options = FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o644);
        zip.start_file(&name, options)?;
        zip.write_all(&contents)?;
    }

    zip.finish()?;
    Ok(())
}

/// Resolves the vpnd log directory path.
///
/// Attempts to get the path from the running daemon, falls back to the default
/// path if the daemon is unavailable or returns an error.
pub(crate) async fn resolve_vpnd_log_dir(
    app_state: &State<'_, SharedAppState>,
    vpnd: &State<'_, VpndClient>,
) -> PathBuf {
    let is_vpnd_down = app_state.lock().await.vpnd_status == VpndStatus::Down;

    if is_vpnd_down {
        warn!("vpnd is down, using default log dir");
        return PathBuf::from(DEFAULT_VPND_LOG_DIR);
    }

    let log_dir = vpnd
        .vpnd_log_path()
        .await
        .inspect_err(|e| warn!("failed to get vpnd log path: {e:?}, using default"))
        .ok()
        .flatten();

    log_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_VPND_LOG_DIR))
}

/// Prompts the user to choose a save location, then builds a zip archive containing
/// the app and vpnd logs, plus an optional extra file (e.g. a diagnostic report).
///
/// Returns `true` if the archive was created successfully, `false` if the user
/// cancelled the save dialog.
pub(crate) async fn export_logs_archive(
    app: &AppHandle,
    app_state: &State<'_, SharedAppState>,
    vpnd: &State<'_, VpndClient>,
    default_file_name: &str,
    extra_file: Option<(String, Vec<u8>)>,
) -> Result<bool, BackendError> {
    let vpnd_log_dir = resolve_vpnd_log_dir(app_state, vpnd).await;
    let app_log_dir = APP_LOG_DIR
        .clone()
        .ok_or_else(|| BackendError::internal("failed to get app log directory path", None))?;

    debug!(vpnd_log_dir = %vpnd_log_dir.display(), app_log_dir = %app_log_dir.display());

    let Some(file_path) = app
        .dialog()
        .file()
        .add_filter("Zip files", &["zip"])
        .set_file_name(default_file_name)
        .blocking_save_file()
    else {
        info!("user cancelled save dialog");
        return Ok(false);
    };

    let output_path = file_path
        .as_path()
        .ok_or_else(|| BackendError::internal("failed to get save path", None))?
        .to_path_buf();

    info!(output = %output_path.display(), "creating logs archive");

    // Run blocking I/O on a dedicated thread to avoid blocking the async runtime
    tokio::task::spawn_blocking(move || {
        create_logs_archive(&output_path, &app_log_dir, &vpnd_log_dir, extra_file)
    })
    .await
    .map_err(|e| {
        error!("task join error: {e}");
        BackendError::internal("failed to create logs archive", None)
    })?
    .map_err(|e| {
        error!("failed to create logs archive: {e}");
        BackendError::internal_with_detail("failed to create logs archive", e.to_string())
    })?;

    info!("logs archive created successfully");
    Ok(true)
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn zip_logs(
    app: AppHandle,
    app_state: State<'_, SharedAppState>,
    vpnd: State<'_, VpndClient>,
) -> Result<bool, BackendError> {
    export_logs_archive(&app, &app_state, &vpnd, "nymvpn-logs.zip", None).await
}
