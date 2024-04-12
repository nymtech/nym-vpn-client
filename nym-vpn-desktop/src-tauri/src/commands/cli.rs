use tauri::State;
use tracing::{debug, instrument};

use crate::{
    cli::{Cli, ManagedCli},
    error::CmdError,
};

#[instrument(skip_all)]
#[tauri::command]
pub fn cli_args(cli: State<'_, ManagedCli>) -> Result<&Cli, CmdError> {
    debug!("cli_args");
    Ok(cli.inner())
}
