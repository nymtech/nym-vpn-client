use tauri::State;
use tracing::instrument;

use crate::{cli::Cli, error::BackendError};

#[instrument(skip_all)]
#[tauri::command]
pub fn cli_args(cli: State<'_, Cli>) -> Result<&Cli, BackendError> {
    Ok(cli.inner())
}
