use tauri::State;
use tracing::{error, instrument};

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
