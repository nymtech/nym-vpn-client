use thiserror::Error;

use crate::error::BackendError;

#[derive(Error, Debug)]
pub enum VpndError {
    #[error("can't connect to daemon without authentication")]
    AuthenticationRequired,
    #[error("failed to connect to daemon")]
    FailedToConnectIpc(#[from] anyhow::Error),
    #[error(transparent)]
    RpcClient(#[from] nym_vpn_proto::rpc_client::Error),
    #[error("call response error {0}")]
    Response(#[from] BackendError),
}
