use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use tracing::{error, info, instrument};

use crate::{
    error::BackendError,
    vpnd::{
        client::VpndClient,
        diagnostic::{DiagnosticReport, DiagnosticRunParams},
    },
};

#[instrument(skip(vpnd))]
#[tauri::command]
pub async fn run_diagnostic(
    vpnd: State<'_, VpndClient>,
    params: DiagnosticRunParams,
) -> Result<DiagnosticReport, BackendError> {
    let report = vpnd.run_diagnostic(params.into()).await.map_err(|e| {
        error!("failed to run diagnostic: {e}");
        BackendError::from(e)
    })?;
    Ok(report.into())
}

#[instrument(skip_all)]
#[tauri::command]
pub async fn share_diagnostic(
    app: AppHandle,
    report: DiagnosticReport,
) -> Result<(), BackendError> {
    let json = serde_json::to_string_pretty(&report).map_err(|e| {
        error!("failed to serialize diagnostic report: {e}");
        BackendError::internal(&format!("failed to serialize diagnostic report: {e}"), None)
    })?;

    let Some(file_path) = app
        .dialog()
        .file()
        .add_filter("JSON files", &["json"])
        .set_file_name("diagnostic-report.json")
        .blocking_save_file()
    else {
        info!("user cancelled save dialog");
        return Ok(());
    };

    let output_path = file_path
        .as_path()
        .ok_or_else(|| BackendError::internal("failed to get save path", None))?
        .to_path_buf();

    std::fs::write(output_path, json).map_err(|e| {
        error!("failed to write diagnostic report: {e}");
        BackendError::internal(&format!("failed to write diagnostic report: {e}"), None)
    })?;

    Ok(())
}
