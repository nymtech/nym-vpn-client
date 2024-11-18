use thiserror::Error;

use crate::error::BackendError;

#[derive(Error, Debug)]
pub enum VpndError {
    #[error("gRPC call error")]
    GrpcError(#[from] tonic::Status),
    #[error("failed to connect to daemon using HTTP transport")]
    FailedToConnectHttp(#[from] tonic::transport::Error),
    #[error("failed to connect to daemon using IPC transport")]
    FailedToConnectIpc(#[from] anyhow::Error),
    #[error("call response error {0}")]
    Response(#[from] BackendError),
}

impl VpndError {
    pub fn internal(err: &str) -> Self {
        VpndError::GrpcError(tonic::Status::internal(err))
    }
}
