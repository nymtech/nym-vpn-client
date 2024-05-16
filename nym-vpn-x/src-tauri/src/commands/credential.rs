use std::sync::Arc;

use tauri::State;
use tracing::{debug, info, instrument, warn};

use crate::{
    error::{CmdError, CmdErrorSource, I18nKey},
    grpc::client::GrpcClient,
};

#[instrument(skip_all)]
#[tauri::command]
pub async fn add_credential(
    credential: String,
    grpc: State<'_, Arc<GrpcClient>>,
) -> Result<(), CmdError> {
    debug!("add_credential");

    let bytes = bs58::decode(credential).into_vec().map_err(|e| {
        info!("failed to decode base58 credential: {:?}", e);
        CmdError::new(CmdErrorSource::CallerError, "bad credential format")
    })?;

    let res = grpc.import_credential(bytes).await?;
    if res.success {
        info!("successfully imported credential");
        Ok(())
    } else {
        warn!("failed to import credential");
        let error = res.error.map(|e| e.kind().into()).unwrap_or_else(|| {
            CmdError::new_with_local(
                CmdErrorSource::InternalError,
                "failed to import credential",
                I18nKey::UnknownError,
            )
        });
        Err(error)
    }
}
