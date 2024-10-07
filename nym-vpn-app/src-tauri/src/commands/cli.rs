use tauri::State;
use tracing::{debug, instrument};

use crate::{cli::Cli, error::BackendError};

#[instrument(skip_all)]
#[tauri::command]
pub fn cli_args(cli: State<'_, Cli>) -> Result<&Cli, BackendError> {
    debug!("cli_args");
    Ok(cli.inner())
}
