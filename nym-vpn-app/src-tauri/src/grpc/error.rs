use thiserror::Error;

use crate::error::BackendError;

#[derive(Error, Debug)]
pub enum VpndError {
    #[error("gRPC call error")]
    GrpcError(#[from] tonic::Status),
    #[error("failed to connect to daemon")]
    FailedToConnectIpc(#[from] anyhow::Error),
    #[error("call response error {0}")]
    Response(#[from] BackendError),
}

impl VpndError {
    pub fn internal(err: &str) -> Self {
        VpndError::GrpcError(tonic::Status::internal(err))
    }
}
