use tauri::State;
use tracing::{debug, instrument};

use crate::{
    cli::{Cli, ManagedCli},
    error::CmdError,
};

#[instrument(skip_all)]
#[tauri::command]
pub fn has_admin() -> Result<bool, CmdError> {
    debug!("Checking for admin privileges");

    #[cfg(unix)]
    return Ok(nym_vpn_lib::util::unix_has_root().is_ok());

    #[cfg(windows)]
    return Ok(nym_vpn_lib::util::win_has_admin().is_ok());

    // Assume we're all good on unknown platforms
    debug!("Platform not supported for root privilege check");
    Ok(true)
}
