use std::sync::Arc;

use tauri::State;
use tracing::{debug, error, info, instrument};

use crate::{
    error::{CmdError, CmdErrorSource},
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

    match grpc.import_credential(bytes).await? {
        true => {
            info!("successfully imported credential");
            Ok(())
        }
        false => {
            error!("failed to import credential");
            Err(CmdError::new(
                CmdErrorSource::InternalError,
                "failed to import credential",
            ))
        }
    }
}
