use std::sync::Arc;
use std::time::SystemTime;

use tauri::State;
use tracing::{debug, info, instrument, warn};

use crate::{
    error::{BackendError, ErrorKey},
    grpc::client::GrpcClient,
};

#[instrument(skip_all)]
#[tauri::command]
pub async fn add_credential(
    credential: String,
    grpc: State<'_, Arc<GrpcClient>>,
) -> Result<Option<i64>, BackendError> {
    debug!("add_credential");

    let bytes = bs58::decode(credential).into_vec().map_err(|e| {
        info!("failed to decode base58 credential: {:?}", e);
        BackendError::new("bad credential format", ErrorKey::CredentialInvalid)
    })?;
    let res = grpc.import_credential(bytes).await?;
    if res.success {
        info!("successfully imported credential");
        Ok(res.expiry.map(|t| {
            t.seconds
        }))
    } else {
        warn!("failed to import credential");
        let error = res.error.map(|e| e.into()).unwrap_or_else(|| {
            BackendError::new("failed to import credential", ErrorKey::UnknownError)
        });
        Err(error)
    }
}
