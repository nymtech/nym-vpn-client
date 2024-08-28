use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use ipnetwork::{IpNetwork, Ipv4Network};
use tokio_util::sync::CancellationToken;

use nym_authenticator_client::AuthClient;
use nym_gateway_directory::{
    AuthAddresses, EntryPoint, ExitPoint, Gateway, GatewayClient, Recipient,
};
use nym_sdk::UserAgent;
use nym_task::TaskManager;
use nym_wg_gateway_client::{GatewayData, WgGatewayClient};
use nym_wg_go::{PrivateKey, PublicKey};

use crate::mixnet::SharedMixnetClient;
use crate::platform::VPNConfig;
use crate::{bandwidth_controller::BandwidthController, GenericNymVpnConfig};
use crate::{GatewayDirectoryError, MixnetClientConfig};

use super::ios::tun_provider::OSTunProvider;
use super::{
    two_hop_tunnel::TwoHopTunnel,
    wg_config::{WgInterface, WgNodeConfig, WgPeer},
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("mixnet timed out during startup")]
    StartMixnetTimeout,

    #[error("failed to start mixnet client: {0}")]
    StartMixnetClient(#[source] Box<dyn std::error::Error + Send>),

    #[error("gateway directory error: {0}")]
    GatewayDirectory(#[from] GatewayDirectoryError),

    #[error("authenticator address not found")]
    AuthenticatorAddressNotFound,

    #[error("not enough bandwidth")]
    NotEnoughBandwidth,

    #[error("wirewireguardurad authentication is not possible due to one of the gateways not running the authenticator process: {0}")]
    AuthenticationNotPossible(String),

    #[error("wireguard gateway failure: {0}")]
    WgGatewayClientFailure(#[from] nym_wg_gateway_client::Error),

    #[error("failed to run two hop tunnel: {0}")]
    Tunnel(#[source] super::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

const MIXNET_CLIENT_STARTUP_TIMEOUT_SECS: Duration = Duration::from_secs(30);
const TASK_MANAGER_SHUTDOWN_TIMER_SECS: u64 = 10;

pub struct TunnelRunner {
    gateway_directory_client: GatewayClient,
    task_manager: TaskManager,
    enable_wireguard: bool,
    generic_config: GenericNymVpnConfig,
    #[cfg(target_os = "ios")]
    tun_provider: Arc<dyn OSTunProvider>,
    shutdown_token: CancellationToken,
}

impl TunnelRunner {
    pub fn new(config: VPNConfig, shutdown_token: CancellationToken) -> Result<Self> {
        let user_agent = UserAgent::from(nym_bin_common::bin_info_local_vergen!());
        let generic_config = GenericNymVpnConfig {
            mixnet_client_config: MixnetClientConfig {
                enable_poisson_rate: false,
                disable_background_cover_traffic: false,
                enable_credentials_mode: false,
                min_mixnode_performance: None,
                min_gateway_performance: None,
            },
            data_path: None,
            gateway_config: nym_gateway_directory::Config::new_from_env(None),
            entry_point: EntryPoint::from(config.entry_gateway),
            exit_point: ExitPoint::from(config.exit_router),
            nym_ips: None,
            nym_mtu: None,
            dns: None,
            disable_routing: false,
            user_agent: Some(user_agent.clone()),
        };

        let task_manager = TaskManager::new(TASK_MANAGER_SHUTDOWN_TIMER_SECS).named("nym_vpn_lib");
        let gateway_directory_client =
            GatewayClient::new(generic_config.gateway_config.clone(), user_agent.clone()).map_err(
                |err| GatewayDirectoryError::FailedtoSetupGatewayDirectoryClient {
                    config: Box::new(generic_config.gateway_config.clone()),
                    source: err,
                },
            )?;

        Ok(Self {
            gateway_directory_client,
            task_manager,
            enable_wireguard: config.enable_two_hop,
            generic_config,
            #[cfg(target_os = "ios")]
            tun_provider: config.tun_provider,
            shutdown_token,
        })
    }

    pub async fn start(self) -> Result<()> {
        let SelectedGateways { entry, exit } = self.select_gateways().await?;
        let mixnet_client = self.start_mixnet_client(&entry).await?;

        if self.enable_wireguard {
            let auth_addresses = match self.setup_auth_addresses(&entry, &exit) {
                Ok(auth_addr) => auth_addr,
                Err(err) => {
                    // Put in some manual error handling, the correct long-term solution is that handling
                    // errors and diconnecting the mixnet client needs to be unified down this code path
                    // and merged with the mix tunnel one.
                    mixnet_client.disconnect().await;
                    return Err(err);
                }
            };
            self.start_wireguard(mixnet_client, auth_addresses).await
        } else {
            todo!("implement mixnet?")
        }
    }

    async fn start_mixnet_client(&self, entry: &Gateway) -> Result<SharedMixnetClient> {
        tracing::info!("Setting up mixnet client");
        tracing::info!("Connecting to mixnet gateway: {}", entry.identity());

        let mixnet_client = tokio::time::timeout(
            MIXNET_CLIENT_STARTUP_TIMEOUT_SECS,
            crate::mixnet::setup_mixnet_client(
                entry.identity(),
                &self.generic_config.data_path,
                self.task_manager.subscribe_named("mixnet_client_main"),
                true,
                self.generic_config.mixnet_client_config.clone(),
            ),
        )
        .await
        .map_err(|_| Error::StartMixnetTimeout)?
        .map_err(|e| Error::StartMixnetClient(Box::new(e)))?;

        Ok(mixnet_client)
    }

    async fn select_gateways(&self) -> Result<SelectedGateways> {
        // The set of exit gateways is smaller than the set of entry gateways, so we start by selecting
        // the exit gateway and then filter out the exit gateway from the set of entry gateways.

        let (mut entry_gateways, exit_gateways) = if self.enable_wireguard {
            let all_gateways = self
                .gateway_directory_client
                .lookup_all_gateways()
                .await
                .map_err(|source| GatewayDirectoryError::FailedToLookupGateways { source })?;
            (all_gateways.clone(), all_gateways)
        } else {
            // Setup the gateway that we will use as the exit point
            let exit_gateways = self
                .gateway_directory_client
                .lookup_exit_gateways()
                .await
                .map_err(|source| GatewayDirectoryError::FailedToLookupGateways { source })?;
            // Setup the gateway that we will use as the entry point
            let entry_gateways = self
                .gateway_directory_client
                .lookup_entry_gateways()
                .await
                .map_err(|source| GatewayDirectoryError::FailedToLookupGateways { source })?;
            (entry_gateways, exit_gateways)
        };

        let exit_gateway = self
            .generic_config
            .exit_point
            .lookup_gateway(&exit_gateways)
            .map_err(|source| GatewayDirectoryError::FailedToSelectExitGateway { source })?;

        // Exclude the exit gateway from the list of entry gateways for privacy reasons
        entry_gateways.remove_gateway(&exit_gateway);

        let entry_gateway = self
            .generic_config
            .entry_point
            .lookup_gateway(&entry_gateways)
            .await
            .map_err(|source| match source {
                nym_gateway_directory::Error::NoMatchingEntryGatewayForLocation {
                    requested_location,
                    available_countries: _,
                } if Some(requested_location.as_str())
                    == exit_gateway.two_letter_iso_country_code() =>
                {
                    GatewayDirectoryError::SameEntryAndExitGatewayFromCountry {
                        requested_location: requested_location.to_string(),
                    }
                }
                _ => GatewayDirectoryError::FailedToSelectEntryGateway { source },
            })?;

        tracing::info!("Found {} entry gateways", entry_gateways.len());
        tracing::info!("Found {} exit gateways", exit_gateways.len());
        tracing::info!(
            "Using entry gateway: {}, location: {}, performance: {}",
            *entry_gateway.identity(),
            entry_gateway
                .two_letter_iso_country_code()
                .map_or_else(|| "unknown".to_string(), |code| code.to_string()),
            entry_gateway
                .performance
                .map_or_else(|| "unknown".to_string(), |perf| perf.to_string()),
        );
        tracing::info!(
            "Using exit gateway: {}, location: {}, performance: {}",
            *exit_gateway.identity(),
            exit_gateway
                .two_letter_iso_country_code()
                .map_or_else(|| "unknown".to_string(), |code| code.to_string()),
            entry_gateway
                .performance
                .map_or_else(|| "unknown".to_string(), |perf| perf.to_string()),
        );
        tracing::info!(
            "Using exit router address {}",
            exit_gateway
                .ipr_address
                .map_or_else(|| "none".to_string(), |ipr| ipr.to_string())
        );

        Ok(SelectedGateways {
            entry: entry_gateway,
            exit: exit_gateway,
        })
    }

    fn setup_auth_addresses(
        &self,
        entry: &nym_gateway_directory::Gateway,
        exit: &nym_gateway_directory::Gateway,
    ) -> Result<AuthAddresses> {
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

    async fn start_wireguard(
        self,
        mixnet_client: SharedMixnetClient,
        auth_addresses: AuthAddresses,
    ) -> Result<()> {
        let bandwidth_controller =
            BandwidthController::new(mixnet_client.clone(), self.task_manager.subscribe());
        tokio::spawn(bandwidth_controller.run());

        let (Some(entry_auth_recipient), Some(exit_auth_recipient)) =
            (auth_addresses.entry().0, auth_addresses.exit().0)
        else {
            return Err(Error::AuthenticationNotPossible(auth_addresses.to_string()));
        };
        let auth_client = AuthClient::new_from_inner(mixnet_client.inner()).await;

        let wg_entry_config = self
            .start_wg_entry_client(auth_client.clone(), entry_auth_recipient)
            .await?;
        let wg_exit_config = self
            .start_wg_exit_client(auth_client.clone(), exit_auth_recipient)
            .await?;

        tracing::info!("Created wg gateway clients");

        TwoHopTunnel::start(
            wg_entry_config,
            wg_exit_config,
            self.tun_provider,
            self.shutdown_token,
        )
        .await
        .map_err(Error::Tunnel)
    }

    async fn start_wg_entry_client(
        &self,
        auth_client: AuthClient,
        recipient: Recipient,
    ) -> Result<WgNodeConfig> {
        let mut wg_entry_gateway_client = WgGatewayClient::new_entry(
            &self.generic_config.data_path,
            auth_client.clone(),
            recipient,
        );

        let (gateway_data, gateway_host) =
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

    async fn start_wg_exit_client(
        &self,
        auth_client: AuthClient,
        recipient: Recipient,
    ) -> Result<WgNodeConfig> {
        let mut wg_exit_gateway_client = WgGatewayClient::new_exit(
            &self.generic_config.data_path,
            auth_client.clone(),
            recipient,
        );

        let (gateway_data, gateway_host) =
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
            .map_err(|source| GatewayDirectoryError::FailedToLookupGatewayIp {
                gateway_id,
                source,
            })?;
        let wg_gateway_data = wg_gateway_client.register_wireguard(gateway_host).await?;
        tracing::debug!("Received wireguard gateway data: {wg_gateway_data:?}");
        Ok((wg_gateway_data, gateway_host))
    }
}

struct SelectedGateways {
    entry: nym_gateway_directory::Gateway,
    exit: nym_gateway_directory::Gateway,
}

impl WgNodeConfig {
    fn with_gateway_data(
        gateway_data: GatewayData,
        private_key: &nym_crypto::asymmetric::encryption::PrivateKey,
    ) -> Self {
        Self {
            interface: WgInterface {
                listen_port: None,
                private_key: PrivateKey::from(private_key.to_bytes()),
                addresses: vec![IpNetwork::V4(
                    Ipv4Network::new(gateway_data.private_ipv4, 32)
                        .expect("private_ipv4/32 to ipnetwork"),
                )],
                dns: crate::DEFAULT_DNS_SERVERS.to_vec(),
                mtu: 0,
            },
            peer: WgPeer {
                public_key: PublicKey::from(*gateway_data.public_key.as_bytes()),
                endpoint: gateway_data.endpoint,
            },
        }
    }
}
