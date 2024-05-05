use std::path::PathBuf;

use anyhow::{anyhow, Result};
use nym_vpn_proto::{
    health_check_response::ServingStatus, health_client::HealthClient,
    nym_vpnd_client::NymVpndClient, DisconnectRequest, HealthCheckRequest, StatusRequest,
};
use nym_vpn_proto::{
    ConnectRequest, ConnectionStatus, Dns, EntryNode, ExitNode, ImportUserCredentialRequest,
};
use parity_tokio_ipc::Endpoint as IpcEndpoint;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use thiserror::Error;
use tokio::sync::mpsc;
use tonic::transport::Endpoint as TonicEndpoint;
use tonic::{transport::Channel, Request};
use tracing::{debug, error, info, instrument, warn};
use ts_rs::TS;

use crate::cli::Cli;
use crate::fs::config::AppConfig;
use crate::{events::AppHandleEventEmitter, states::SharedAppState};

const VPND_SERVICE: &str = "nym.vpn.NymVpnd";
const DEFAULT_SOCKET_PATH: &str = "/var/run/nym-vpn.sock";
const DEFAULT_HTTP_ENDPOINT: &str = "http://[::1]:53181";

#[derive(Clone, Debug)]
enum Transport {
    Http(String),
    Ipc(PathBuf),
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, TS)]
pub enum VpndStatus {
    Ok,
    #[default]
    NotOk,
}

#[derive(Error, Debug)]
pub enum VpndError {
    #[error("rpc error")]
    RpcError(#[from] tonic::Status),
    #[error("not connected to daemon")]
    NotConnected,
}

#[derive(Debug, Default, Clone)]
pub struct GrpcClient(Transport);

impl GrpcClient {
    #[instrument(skip_all)]
    pub fn new(config: &AppConfig, cli: &Cli) -> Self {
        let client = GrpcClient(Transport::from((config, cli)));
        match &client.0 {
            Transport::Http(endpoint) => {
                info!("using grpc HTTP transport: {}", endpoint);
            }
            Transport::Ipc(socket) => {
                info!("using grpc IPC transport: {}", socket.display());
            }
        }
        client
    }

    /// Get the Vpnd service client
    #[instrument(skip_all)]
    pub async fn vpnd(&self) -> Result<NymVpndClient<Channel>> {
        match &self.0 {
            Transport::Http(endpoint) => {
                NymVpndClient::connect(endpoint.clone()).await.map_err(|e| {
                    warn!("failed to connect to the daemon: {}", e);
                    anyhow!("failed to connect to the daemon: {}", e)
                })
            }
            Transport::Ipc(socket) => {
                let channel = get_channel(socket.clone()).await.map_err(|e| {
                    warn!("failed to connect to the daemon: {}", e);
                    anyhow!("failed to connect to the daemon: {}", e)
                })?;
                Ok(NymVpndClient::new(channel))
            }
        }
    }

    /// Get the Health service client
    #[instrument(skip_all)]
    pub async fn health(&self) -> Result<HealthClient<Channel>> {
        match &self.0 {
            Transport::Http(endpoint) => {
                HealthClient::connect(endpoint.clone()).await.map_err(|e| {
                    warn!("failed to connect to the daemon: {}", e);
                    anyhow!("failed to connect to the daemon: {}", e)
                })
            }
            Transport::Ipc(socket) => {
                let channel = get_channel(socket.clone()).await.map_err(|e| {
                    warn!("failed to connect to the daemon: {}", e);
                    anyhow!("failed to connect to the daemon: {}", e)
                })?;
                Ok(HealthClient::new(channel))
            }
        }
    }

    /// Check the connection with the grpc server
    #[instrument(skip_all)]
    pub async fn check(&self, app_state: &SharedAppState) -> Result<VpndStatus> {
        let mut health = self.health().await?;

        let request = Request::new(HealthCheckRequest {
            service: VPND_SERVICE.into(),
        });
        let response = health
            .check(request)
            .await
            .inspect_err(|e| {
                error!("health check failed: {}", e);
            })?
            .into_inner();
        let status = response.status();
        let mut state = app_state.lock().await;
        state.vpnd_status = status.into();

        Ok(status.into())
    }

    #[instrument(skip_all)]
    pub async fn vpn_status(&self) -> Result<ConnectionStatus, VpndError> {
        debug!("vpn_status");
        let mut vpnd = self.vpnd().await.map_err(|_| VpndError::NotConnected)?;

        let request = Request::new(StatusRequest {});
        let response = vpnd.vpn_status(request).await.map_err(|e| {
            error!("grpc vpn_status: {}", e);
            VpndError::RpcError(e)
        })?;
        debug!("grpc response: {:?}", response);

        Ok(response.into_inner().status())
    }

    /// Connect to the VPN
    #[instrument(skip_all)]
    pub async fn vpn_connect(
        &self,
        entry_node: EntryNode,
        exit_node: ExitNode,
        two_hop_mod: bool,
        dns: Option<Dns>,
    ) -> Result<bool, VpndError> {
        debug!("vpn_connect");
        let mut vpnd = self.vpnd().await.map_err(|_| VpndError::NotConnected)?;

        let request = Request::new(ConnectRequest {
            entry: Some(entry_node),
            exit: Some(exit_node),
            disable_routing: false,
            enable_two_hop: two_hop_mod,
            enable_poisson_rate: false,
            disable_background_cover_traffic: false,
            enable_credentials_mode: false,
            dns,
        });
        let response = vpnd.vpn_connect(request).await.map_err(|e| {
            error!("grpc vpn_connect: {}", e);
            VpndError::RpcError(e)
        })?;
        debug!("grpc response: {:?}", response);

        Ok(response.into_inner().success)
    }

    /// Disconnect from the VPN
    #[instrument(skip_all)]
    pub async fn vpn_disconnect(&self) -> Result<bool, VpndError> {
        debug!("vpn_disconnect");
        let mut vpnd = self.vpnd().await.map_err(|_| VpndError::NotConnected)?;

        let request = Request::new(DisconnectRequest {});
        let response = vpnd.vpn_disconnect(request).await.map_err(|e| {
            error!("grpc vpn_disconnect: {}", e);
            VpndError::RpcError(e)
        })?;
        debug!("grpc response: {:?}", response);

        Ok(response.into_inner().success)
    }

    /// Import user credential from base58 encoded string
    #[instrument(skip_all)]
    pub async fn import_credential(&self, credential: Vec<u8>) -> Result<bool, VpndError> {
        debug!("import_credential");
        let mut vpnd = self.vpnd().await.map_err(|_| VpndError::NotConnected)?;

        let request = Request::new(ImportUserCredentialRequest { credential });
        let response = vpnd.import_user_credential(request).await.map_err(|e| {
            error!("grpc import_user_credential: {}", e);
            VpndError::RpcError(e)
        })?;
        debug!("grpc response: {:?}", response);

        Ok(response.into_inner().success)
    }

    /// Watch the connection with the grpc server
    #[instrument(skip_all)]
    pub async fn watch(&self, app: &AppHandle) -> Result<()> {
        let mut health = self.health().await?;
        let app_state = app.state::<SharedAppState>();

        let request = Request::new(HealthCheckRequest {
            service: VPND_SERVICE.into(),
        });
        let mut stream = health
            .watch(request)
            .await
            .inspect_err(|e| {
                error!("health check failed: {}", e);
            })?
            .into_inner();

        let (tx, mut rx) = mpsc::channel(32);
        tokio::spawn(async move {
            loop {
                match stream.message().await {
                    Ok(Some(res)) => {
                        tx.send(res.status()).await.unwrap();
                    }
                    Ok(None) => {
                        warn!("watch health stream closed by the server");
                        tx.send(ServingStatus::NotServing).await.unwrap();
                        return;
                    }
                    Err(e) => {
                        warn!("watch health stream get a grpc error: {}", e);
                    }
                }
            }
        });

        while let Some(status) = rx.recv().await {
            debug!("health check status: {:?}", status);
            app.emit_vpnd_status(status.into());
            let mut state = app_state.lock().await;
            state.vpnd_status = status.into();
        }

        Ok(())
    }
}

impl From<ServingStatus> for VpndStatus {
    fn from(status: ServingStatus) -> Self {
        match status {
            ServingStatus::Serving => VpndStatus::Ok,
            _ => VpndStatus::NotOk,
        }
    }
}

async fn get_channel(socket_path: PathBuf) -> anyhow::Result<Channel> {
    // NOTE the uri here is ignored
    Ok(TonicEndpoint::from_static(DEFAULT_HTTP_ENDPOINT)
        .connect_with_connector(tower::service_fn(move |_| {
            IpcEndpoint::connect(socket_path.clone())
        }))
        .await?)
}

impl Default for Transport {
    fn default() -> Self {
        Transport::Ipc(DEFAULT_SOCKET_PATH.into())
    }
}

impl From<(&AppConfig, &Cli)> for Transport {
    fn from((config, cli): (&AppConfig, &Cli)) -> Self {
        let http_mode = config.grpc_http_mode.unwrap_or(cli.grpc_http_mode);
        match http_mode {
            true => Transport::Http(
                cli.grpc_http_endpoint.clone().unwrap_or(
                    config
                        .grpc_http_endpoint
                        .clone()
                        .unwrap_or(DEFAULT_HTTP_ENDPOINT.into()),
                ),
            ),
            false => Transport::Ipc(
                cli.grpc_socket_endpoint.clone().unwrap_or(
                    config
                        .grpc_socket_endpoint
                        .clone()
                        .unwrap_or(DEFAULT_SOCKET_PATH.into()),
                ),
            ),
        }
    }
}
