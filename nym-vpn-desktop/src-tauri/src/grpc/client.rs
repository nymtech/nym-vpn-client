use anyhow::{anyhow, Result};
use nym_vpn_proto::nym_vpnd_client::NymVpndClient;
use tonic::transport::Channel;
use tracing::error;

#[derive(Debug, Default)]
pub struct GrpcClient {
    pub client: Option<NymVpndClient<Channel>>,
    pub endpoint: String,
}

impl GrpcClient {
    pub fn new(address: &str) -> Self {
        Self {
            client: None,
            endpoint: address.to_string(),
        }
    }

    pub async fn try_connect(&mut self) -> Result<()> {
        self.client = Some(
            NymVpndClient::connect(self.endpoint.clone())
                .await
                .inspect_err(|e| {
                    error!("failed to connect to the daemon: {:?}", e);
                })?,
        );
        Ok(())
    }

    pub fn client(&self) -> Result<NymVpndClient<Channel>> {
        let client = self
            .client
            .clone()
            .ok_or_else(|| anyhow!("gRPC client not connected"))?;
        Ok(client)
    }
}
