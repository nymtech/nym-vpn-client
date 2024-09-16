use std::net::IpAddr;

use nym_authenticator_client::AuthClient;
use nym_wg_gateway_client::{GatewayData, WgGatewayClient};
use tokio_util::sync::CancellationToken;

use nym_gateway_directory::{AuthAddresses, Gateway, GatewayClient, Recipient};
use nym_task::TaskManager;

use super::{gateway_selector::SelectedGateways, Error, Result};
use crate::{mixnet::SharedMixnetClient, wg_config::WgNodeConfig, GenericNymVpnConfig};

pub struct WireGuardTunnel {
    nym_config: GenericNymVpnConfig,
    task_manager: TaskManager,
    mixnet_client: SharedMixnetClient,
    auth_client: AuthClient,
    entry_auth_recipient: Recipient,
    exit_auth_recipient: Recipient,
    gateway_directory_client: GatewayClient,
    selected_gateways: SelectedGateways,
    shutdown_token: CancellationToken,
}

impl WireGuardTunnel {
    pub async fn new(
        nym_config: GenericNymVpnConfig,
        task_manager: TaskManager,
        mixnet_client: SharedMixnetClient,
        gateway_directory_client: GatewayClient,
        selected_gateways: SelectedGateways,
        shutdown_token: CancellationToken,
    ) -> Result<Self> {
        let auth_addresses =
            Self::setup_auth_addresses(&selected_gateways.entry, &selected_gateways.exit)?;
        let (Some(entry_auth_recipient), Some(exit_auth_recipient)) =
            (auth_addresses.entry().0, auth_addresses.exit().0)
        else {
            return Err(Error::AuthenticationNotPossible(auth_addresses.to_string()));
        };
        let auth_client = AuthClient::new_from_inner(mixnet_client.inner()).await;

        Ok(Self {
            nym_config,
            task_manager,
            mixnet_client,
            auth_client,
            entry_auth_recipient,
            exit_auth_recipient,
            gateway_directory_client,
            selected_gateways,
            shutdown_token,
        })
    }

    pub async fn run(mut self) -> Result<()> {
        let wg_entry_config = self.start_wg_entry_client().await?;
        let wg_exit_config = self.start_wg_exit_client().await?;

        let mut shutdown_task_client = self.task_manager.subscribe();

        tokio::select! {
            _ = shutdown_task_client.recv() => {
                tracing::debug!("Task manager received shutdown.");
            }
            _ = self.shutdown_token.cancelled() => {
                tracing::debug!("Received cancellation. Shutting down task manager.");
                _ = self.task_manager.signal_shutdown();
            }
        }

        self.task_manager.wait_for_shutdown().await;
        self.mixnet_client.disconnect().await;

        Ok(())
    }

    fn setup_auth_addresses(entry: &Gateway, exit: &Gateway) -> Result<AuthAddresses> {
        let entry_authenticator_address = entry
            .authenticator_address
            .ok_or(Error::AuthenticatorAddressNotFound)?;
        let exit_authenticator_address = exit
            .authenticator_address
            .ok_or(Error::AuthenticatorAddressNotFound)?;
        Ok(AuthAddresses::new(
            entry_authenticator_address,
            exit_authenticator_address,
        ))
    }

    async fn start_wg_entry_client(&self) -> Result<WgNodeConfig> {
        let mut wg_entry_gateway_client = WgGatewayClient::new_entry(
            &self.nym_config.data_path,
            self.auth_client.clone(),
            self.entry_auth_recipient,
        );

        let (gateway_data, _gateway_host) =
            self.register_wg_key(&mut wg_entry_gateway_client).await?;
        let key_pair = wg_entry_gateway_client.keypair();
        let node_config = WgNodeConfig::with_gateway_data(gateway_data, key_pair.private_key());

        if wg_entry_gateway_client.suspended().await? {
            return Err(Error::NotEnoughBandwidth);
        }

        tokio::spawn(
            wg_entry_gateway_client
                .run(self.task_manager.subscribe_named("bandwidth_entry_client")),
        );

        Ok(node_config)
    }

    async fn start_wg_exit_client(&self) -> Result<WgNodeConfig> {
        let mut wg_exit_gateway_client = WgGatewayClient::new_exit(
            &self.nym_config.data_path,
            self.auth_client.clone(),
            self.exit_auth_recipient,
        );

        let (gateway_data, _gateway_host) =
            self.register_wg_key(&mut wg_exit_gateway_client).await?;
        let key_pair = wg_exit_gateway_client.keypair();
        let node_config = WgNodeConfig::with_gateway_data(gateway_data, key_pair.private_key());

        if wg_exit_gateway_client.suspended().await? {
            return Err(Error::NotEnoughBandwidth);
        }

        tokio::spawn(
            wg_exit_gateway_client.run(self.task_manager.subscribe_named("bandwidth_exit_client")),
        );

        Ok(node_config)
    }

    async fn register_wg_key(
        &self,
        wg_gateway_client: &mut WgGatewayClient,
    ) -> Result<(GatewayData, IpAddr)> {
        // First we need to register with the gateway to setup keys and IP assignment
        tracing::info!("Registering with wireguard gateway");
        let gateway_id = wg_gateway_client
            .auth_recipient()
            .gateway()
            .to_base58_string();
        let gateway_host = self
            .gateway_directory_client
            .lookup_gateway_ip(&gateway_id)
            .await
            .map_err(|source| Error::FailedToLookupGatewayIp { gateway_id, source })?;
        let wg_gateway_data = wg_gateway_client.register_wireguard(gateway_host).await?;
        tracing::debug!("Received wireguard gateway data: {wg_gateway_data:?}");
        Ok((wg_gateway_data, gateway_host))
    }
}
