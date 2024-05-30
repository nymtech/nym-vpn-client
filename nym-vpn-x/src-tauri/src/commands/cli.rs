use tauri::State;
use tracing::{debug, instrument};

use crate::{
    cli::{Cli, ManagedCli},
    error::BackendError,
};

#[instrument(skip_all)]
#[tauri::command]
pub fn cli_args(cli: State<'_, ManagedCli>) -> Result<&Cli, BackendError> {
    debug!("cli_args");
    Ok(cli.inner())
}
