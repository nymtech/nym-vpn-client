use nym_vpn_lib_types as lib;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use tracing::{error, info, instrument};

use crate::{
    commands::fs::export_logs_archive,
    error::BackendError,
    state::SharedAppState,
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

/// Runs the diagnostic tool and lets the user save the resulting report, along with
/// the app and vpnd logs, as a single zip archive.
///
/// Returns `true` if the archive was created successfully, `false` if the user
/// cancelled the save dialog.
#[instrument(skip_all)]
#[tauri::command]
pub async fn share_diagnostics_and_logs(
    app: AppHandle,
    app_state: State<'_, SharedAppState>,
    vpnd: State<'_, VpndClient>,
) -> Result<bool, BackendError> {
    let params = lib::DiagnosticRunParams {
        gateway: None,
        skip_dns: false,
        skip_http: false,
        skip_hybrid_transport: false,
    };
    let report = vpnd.run_diagnostic(params).await.map_err(|e| {
        error!("failed to run diagnostic: {e}");
        BackendError::from(e)
    })?;
    let report = DiagnosticReport::from(report);

    let json = serde_json::to_string_pretty(&report).map_err(|e| {
        error!("failed to serialize diagnostic report: {e}");
        BackendError::internal(&format!("failed to serialize diagnostic report: {e}"), None)
    })?;

    export_logs_archive(
        &app,
        &app_state,
        &vpnd,
        "nymvpn-diagnostics.zip",
        Some(("diagnostic-report.json".to_owned(), json.into_bytes())),
    )
    .await
}
