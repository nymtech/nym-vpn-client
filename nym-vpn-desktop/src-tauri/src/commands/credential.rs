use nym_vpn_lib::{credentials::import_credential_base58, nym_config::defaults::var_names};
use tauri::{api::path::data_dir, State};
use tracing::{debug, error, info, instrument};

use crate::{
    db::{Db, Key},
    error::{CmdError, CmdErrorSource},
    fs::util::check_dir,
    APP_DIR,
};

const CREDENTIAL_DIR: &str = "credential";

#[instrument(skip_all)]
#[tauri::command]
pub async fn add_credential(credential: String, db: State<'_, Db>) -> Result<(), CmdError> {
    debug!("add_credential");
    let network_name = std::env::var(var_names::NETWORK_NAME)
        .inspect_err(|e| error!("failed to get current network name: {}", e))
        .unwrap_or("unknown".into());
    let mut path = data_dir().ok_or(CmdError::new(
        CmdErrorSource::InternalError,
        "Failed to retrieve data directory path".to_string(),
    ))?;
    path.push(APP_DIR);
    path.push(CREDENTIAL_DIR);
    path.push(network_name);
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
            // TODO improve error handling, distinguish between
            // errors linked to user responsibility or internal errors
            CmdError::new(
                CmdErrorSource::InternalError,
                "failed to import credential".to_string(),
            )
        })?;

    db.insert(Key::ImportedCredential, true)
        .map_err(|_| CmdError::new(CmdErrorSource::InternalError, "Cache error".to_string()))?;

    Ok(())
}
