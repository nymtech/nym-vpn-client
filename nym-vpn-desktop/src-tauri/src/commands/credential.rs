use nym_vpn_lib::credentials::import_credential_base58;
use tracing::{debug, error, instrument};

use crate::{
    error::{CmdError, CmdErrorSource},
    fs::path::BACKEND_DATA_PATH,
};

#[instrument(skip_all)]
#[tauri::command]
pub async fn add_credential(credential: String) -> Result<(), CmdError> {
    debug!("add_credential");

    debug!(
        "using path for credential: {}",
        BACKEND_DATA_PATH.to_string_lossy()
    );
    import_credential_base58(&credential, BACKEND_DATA_PATH.clone())
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

    Ok(())
}
