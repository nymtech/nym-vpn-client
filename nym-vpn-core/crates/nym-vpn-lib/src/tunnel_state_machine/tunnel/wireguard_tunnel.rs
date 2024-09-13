use std::net::IpAddr;

use nym_authenticator_client::AuthClient;
use nym_wg_gateway_client::{GatewayData, WgGatewayClient};
use tokio_util::sync::CancellationToken;

use nym_gateway_directory::{AuthAddresses, Gateway, GatewayClient, Recipient};
use nym_task::TaskManager;

use super::{gateway_selector::SelectedGateways, Error, Result};
use crate::{mixnet::SharedMixnetClient, wg_config::WgNodeConfig, GenericNymVpnConfig};

pub struct WireGuardTunnel {
    task_manager: TaskManager,
    mixnet_client: SharedMixnetClient,
    shutdown_token: CancellationToken,
}

impl WireGuardTunnel {
    pub async fn run(
        nym_config: GenericNymVpnConfig,
        task_manager: TaskManager,
        mixnet_client: SharedMixnetClient,
        gateway_directory_client: GatewayClient,
        selected_gateways: SelectedGateways,
        shutdown_token: CancellationToken,
    ) -> Result<()> {
        let auth_addresses =
            Self::setup_auth_addresses(&selected_gateways.entry, &selected_gateways.exit)?;
        let (Some(entry_auth_recipient), Some(exit_auth_recipient)) =
            (auth_addresses.entry().0, auth_addresses.exit().0)
        else {
            return Err(Error::AuthenticationNotPossible(auth_addresses.to_string()));
        };
        let auth_client = AuthClient::new_from_inner(mixnet_client.inner()).await;

        let wg_entry_config = Self::start_wg_entry_client(
            &task_manager,
            &gateway_directory_client,
            auth_client.clone(),
            &nym_config,
            entry_auth_recipient,
        )
        .await?;
        let wg_exit_config = Self::start_wg_exit_client(
            &task_manager,
            &gateway_directory_client,
            auth_client.clone(),
            &nym_config,
            exit_auth_recipient,
        )
        .await?;

        let tunnel = Self {
            task_manager,
            mixnet_client,
            shutdown_token,
        };
        tunnel.wait().await;

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

    async fn start_wg_entry_client(
        task_manager: &TaskManager,
        gateway_directory_client: &GatewayClient,
        auth_client: AuthClient,
        nym_config: &GenericNymVpnConfig,
        recipient: Recipient,
    ) -> Result<WgNodeConfig> {
        let mut wg_entry_gateway_client =
            WgGatewayClient::new_entry(&nym_config.data_path, auth_client.clone(), recipient);

        let (gateway_data, _gateway_host) =
            Self::register_wg_key(&gateway_directory_client, &mut wg_entry_gateway_client).await?;
        let key_pair = wg_entry_gateway_client.keypair();
        let node_config = WgNodeConfig::with_gateway_data(gateway_data, key_pair.private_key());

        if wg_entry_gateway_client.suspended().await? {
            return Err(Error::NotEnoughBandwidth);
        }

        tokio::spawn(
            wg_entry_gateway_client.run(task_manager.subscribe_named("bandwidth_entry_client")),
        );

        Ok(node_config)
    }

    async fn start_wg_exit_client(
        task_manager: &TaskManager,
        gateway_directory_client: &GatewayClient,
        auth_client: AuthClient,
        nym_config: &GenericNymVpnConfig,
        recipient: Recipient,
    ) -> Result<WgNodeConfig> {
        let mut wg_exit_gateway_client =
            WgGatewayClient::new_exit(&nym_config.data_path, auth_client.clone(), recipient);

        let (gateway_data, _gateway_host) =
            Self::register_wg_key(&gateway_directory_client, &mut wg_exit_gateway_client).await?;
        let key_pair = wg_exit_gateway_client.keypair();
        let node_config = WgNodeConfig::with_gateway_data(gateway_data, key_pair.private_key());

        if wg_exit_gateway_client.suspended().await? {
            return Err(Error::NotEnoughBandwidth);
        }

        tokio::spawn(
            wg_exit_gateway_client.run(task_manager.subscribe_named("bandwidth_exit_client")),
        );

        Ok(node_config)
    }

    async fn register_wg_key(
        gateway_directory_client: &GatewayClient,
        wg_gateway_client: &mut WgGatewayClient,
    ) -> Result<(GatewayData, IpAddr)> {
        // First we need to register with the gateway to setup keys and IP assignment
        tracing::info!("Registering with wireguard gateway");
        let gateway_id = wg_gateway_client
            .auth_recipient()
            .gateway()
            .to_base58_string();
        let gateway_host = gateway_directory_client
            .lookup_gateway_ip(&gateway_id)
            .await
            .map_err(|source| Error::FailedToLookupGatewayIp { gateway_id, source })?;
        let wg_gateway_data = wg_gateway_client.register_wireguard(gateway_host).await?;
        tracing::debug!("Received wireguard gateway data: {wg_gateway_data:?}");
        Ok((wg_gateway_data, gateway_host))
    }

    async fn wait(mut self) {
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
    }
}
