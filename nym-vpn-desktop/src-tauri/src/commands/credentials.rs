use nym_vpn_lib::credentials::import_credential_base58;
use tauri::api::path::data_dir;
use tracing::{debug, error, info, instrument};

use crate::{
    error::{CmdError, CmdErrorSource},
    fs::util::check_dir,
    APP_DIR,
};

const CREDENTIAL_DIR: &str = "credential";

#[instrument(skip_all)]
#[tauri::command]
pub async fn add_credential(credential: String) -> Result<(), CmdError> {
    debug!("add_credential");
    let mut path = data_dir().ok_or(CmdError::new(
        CmdErrorSource::InternalError,
        "Failed to retrieve data directory path".to_string(),
    ))?;
    path.push(APP_DIR);
    path.push(CREDENTIAL_DIR);
    info!("using path for credential: {:?}", path);
    check_dir(&path).await.map_err(|e| {
        error!("failed to check directory {:?}: {e}", path);
        CmdError::new(
            CmdErrorSource::InternalError,
            "failed to import credential".to_string(),
        )
    })?;

    import_credential_base58(&credential, path)
        .await
        .map_err(|e| {
            error!("failed to import credential: {:?}", e);
            CmdError::new(
                CmdErrorSource::InternalError,
                "failed to import credential".to_string(),
            )
        })?;
    // TODO handle errors/bad credential

    Ok(())
}
