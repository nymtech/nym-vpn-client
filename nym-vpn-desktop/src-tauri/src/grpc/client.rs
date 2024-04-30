use anyhow::{anyhow, Result};
use nym_vpn_proto::{
    health_check_response::ServingStatus, health_client::HealthClient,
    nym_vpnd_client::NymVpndClient, HealthCheckRequest,
};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tokio::sync::mpsc;
use tonic::{transport::Channel, Request};
use tracing::{debug, error, instrument, warn};
use ts_rs::TS;

use crate::events::AppHandleEventEmitter;

const VPND_SERVICE: &str = "nym.vpn.NymVpnd";

#[derive(Serialize, Deserialize, Default, Clone, Debug, TS)]
pub enum VpndStatus {
    Ok,
    #[default]
    NotOk,
}

#[derive(Debug, Default, Clone)]
pub struct GrpcClient {
    pub vpnd: Option<NymVpndClient<Channel>>,
    pub health: Option<HealthClient<Channel>>,
    pub endpoint: String,
    status: ServingStatus,
}

impl GrpcClient {
    pub fn new(address: &str) -> Self {
        Self {
            endpoint: address.to_string(),
            status: ServingStatus::Unknown,
            ..Default::default()
        }
    }

    /// Try to connect to the grpc servers and set Vpnd and Health service clients
    #[instrument(skip_all)]
    pub async fn try_connect(&mut self) -> Result<()> {
        self.health = Some(
            HealthClient::connect(self.endpoint.clone())
                .await
                .inspect_err(|e| {
                    error!("failed to connect to the daemon: {:?}", e);
                })?,
        );
        self.vpnd = Some(
            NymVpndClient::connect(self.endpoint.clone())
                .await
                .inspect_err(|e| {
                    error!("failed to connect to the daemon: {:?}", e);
                })?,
        );
        Ok(())
    }

    /// Get the Vpnd service client
    #[instrument(skip_all)]
    pub fn vpnd(&self) -> Result<NymVpndClient<Channel>> {
        let client = self
            .vpnd
            .clone()
            .ok_or_else(|| anyhow!("gRPC client not connected"))?;
        Ok(client)
    }

    /// Get the health service client
    #[instrument(skip_all)]
    fn health(&self) -> Result<HealthClient<Channel>> {
        let client = self
            .health
            .clone()
            .ok_or_else(|| anyhow!("gRPC client not connected"))?;
        Ok(client)
    }

    /// Get latest reported connection status with the grpc server
    #[instrument(skip_all)]
    pub fn status(&self) -> VpndStatus {
        self.status.into()
    }

    /// Check the connection with the grpc server
    #[instrument(skip_all)]
    pub async fn check(&mut self) -> Result<VpndStatus> {
        let mut health = self.health().inspect_err(|_| {
            warn!("not connected to the daemon");
        })?;

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
        self.status = status;

        Ok(status.into())
    }

    /// Watch the connection with the grpc server
    #[instrument(skip_all)]
    pub async fn watch(&mut self, app: &AppHandle) -> Result<()> {
        let mut health = self.health().inspect_err(|_| {
            warn!("not connected to the daemon");
        })?;

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
            while let Some(res) = stream
                .message()
                .await
                .inspect_err(|e| error!("health check response: {}", e))
                .unwrap()
            {
                tx.send(res.status()).await.unwrap();
            }
        });

        while let Some(status) = rx.recv().await {
            debug!("health check status: {:?}", status);
            self.status = status;
            app.emit_vpnd_status(status.into());
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
