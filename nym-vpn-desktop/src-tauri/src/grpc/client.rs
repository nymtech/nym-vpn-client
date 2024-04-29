use anyhow::{anyhow, Result};
use nym_vpn_proto::{health_client::HealthClient, nym_vpnd_client::NymVpndClient};
use tonic::transport::Channel;
use tracing::error;

#[derive(Debug, Default, Clone)]
pub struct GrpcClient {
    pub vpnd: Option<NymVpndClient<Channel>>,
    pub health: Option<HealthClient<Channel>>,
    pub endpoint: String,
}

impl GrpcClient {
    pub fn new(address: &str) -> Self {
        Self {
            vpnd: None,
            health: None,
            endpoint: address.to_string(),
        }
    }

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

    pub fn vpnd(&self) -> Result<NymVpndClient<Channel>> {
        let client = self
            .vpnd
            .clone()
            .ok_or_else(|| anyhow!("gRPC client not connected"))?;
        Ok(client)
    }

    pub fn health(&self) -> Result<HealthClient<Channel>> {
        let client = self
            .health
            .clone()
            .ok_or_else(|| anyhow!("gRPC client not connected"))?;
        Ok(client)
    }
}
