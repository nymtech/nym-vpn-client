use std::sync::Arc;

use nym_vpn_proto::ImportUserCredentialRequest;
use tauri::State;
use tonic::Request;
use tracing::{debug, error, info, instrument};

use crate::{
    error::{CmdError, CmdErrorSource},
    grpc::client::GrpcClient,
};

#[instrument(skip_all)]
#[tauri::command]
pub async fn add_credential(
    credential: String,
    grpc_client_state: State<'_, Arc<GrpcClient>>,
) -> Result<(), CmdError> {
    debug!("add_credential");

    let mut grpc_client = grpc_client_state.client().map_err(|_| {
        error!("not connected to nym daemon");
        CmdError::new(CmdErrorSource::DaemonError, "not connected to nym daemon")
    })?;

    let request = Request::new(ImportUserCredentialRequest {
        credential: bs58::decode(credential).into_vec().map_err(|e| {
            info!("failed to decode base58 credential: {:?}", e);
            CmdError::new(CmdErrorSource::CallerError, "bad credential format")
        })?,
    });
    let response = grpc_client
        .import_user_credential(request)
        .await
        .map_err(|e| {
            error!("grpc error: {}", e);
            CmdError::new(
                CmdErrorSource::DaemonError,
                &format!("failed to import user credential: {e}"),
            )
        })?;

    match response.get_ref().success {
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
