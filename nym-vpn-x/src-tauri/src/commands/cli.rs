use tauri::State;
use tracing::{debug, instrument};

use crate::{
    cli::{Cli, ManagedCli},
    error::BkdError,
};

#[instrument(skip_all)]
#[tauri::command]
pub fn cli_args(cli: State<'_, ManagedCli>) -> Result<&Cli, BkdError> {
    debug!("cli_args");
    Ok(cli.inner())
}
