// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Error, Result};
use nym_bandwidth_controller::{AvailableTicketbooks, requests::BandwidthControllerRequestSender};
use nym_bandwidth_fetcher::NyxdCredentialFetcher;
use nym_offline_monitor::ConnectivityMonitor;
use nym_vpn_api_client::{
    VpnApiClient,
    types::{Device, VpnAccount},
};
use nym_vpn_lib_types::{StorableAccount, VpnAccountSummary};
use std::sync::Arc;

use nym_vpn_credential_fetcher::VpnApiCredentialFetcher;
use nym_vpn_store::{device_state::StoredDeviceState, keys::wireguard::WireguardKeysDb};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::{
    AccountControllerConfig, AccountControllerEventSender,
    commands::ReturnSender,
    deeplink::Deeplinks,
    nyxd_client::NyxdClient,
    qr_pairing::QrPairingHandler,
    storage::{AccountStorageOp, request_stored_account, request_vpn_api_account},
};

/// This shared state is the sole propriety of the AccountController and contains the element that must be passed around the different states
/// Ideally, we would have tunnel state here. But it makes circular dependency where tunnel needs AC state and AC needs tunnel state
pub(crate) struct SharedAccountState<C: ConnectivityMonitor> {
    /// Offline monitoring
    pub connectivity_handle: C,

    /// Channel to the bandwidth controller, to set the proper fetcher
    pub(crate) bandwidth_control_command_tx: BandwidthControllerRequestSender,

    /// Config for the account controller
    pub config: AccountControllerConfig,

    /// Wireguard keys database storage
    pub(crate) wireguard_keys_storage: WireguardKeysDb,

    /// VPN API client
    pub(crate) vpn_api_client: VpnApiClient,

    /// Nyxd RPC client
    pub(crate) nyxd_client: NyxdClient,

    /// Current device state
    /// Includes metadata such as account summary when available.
    pub(crate) device_state: Option<StoredDeviceState>,

    /// Registered device
    pub(crate) device: Option<Device>,

    /// Deeplinks for signing-in via services like Privy
    pub(crate) deeplinks: Deeplinks,

    /// Firewall status
    pub(crate) firewall_active: bool,

    /// Which credential fetcher is currently installed on the bandwidth controller
    pub(crate) current_credential_fetcher: CredentialFetcherInUse,

    /// Channel to send storage operation to the AccountController
    pub(crate) storage_op_sender: mpsc::UnboundedSender<AccountStorageOp>,

    /// Channel for broadcasting global `AccountController` events
    pub(crate) event_sender: AccountControllerEventSender,

    /// Handler for the QR pairing process
    pub(crate) qr_pairing_handler: QrPairingHandler<VpnApiClient>,

    /// AccountController cancellation token; child tokens are handed to constructed fetchers so
    /// controller shutdown interrupts their in-flight work.
    pub(crate) cancel_token: CancellationToken,
}

impl<C: ConnectivityMonitor> SharedAccountState<C> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        connectivity_handle: C,
        bandwidth_control_command_tx: BandwidthControllerRequestSender,
        config: AccountControllerConfig,
        wireguard_keys_storage: WireguardKeysDb,
        vpn_api_client: VpnApiClient,
        nyxd_client: NyxdClient,
        device_state: Option<StoredDeviceState>,
        device: Option<Device>,
        storage_op_sender: mpsc::UnboundedSender<AccountStorageOp>,
        event_sender: AccountControllerEventSender,
        cancel_token: CancellationToken,
    ) -> Self {
        let qr_pairing_handler = QrPairingHandler::new(vpn_api_client.clone());

        Self {
            connectivity_handle,
            bandwidth_control_command_tx,
            config,
            wireguard_keys_storage,
            vpn_api_client,
            nyxd_client,
            device_state,
            device,
            deeplinks: Deeplinks::default(),
            firewall_active: false,
            current_credential_fetcher: CredentialFetcherInUse::None,
            storage_op_sender,
            event_sender,
            qr_pairing_handler,
            cancel_token,
        }
    }

    /// Retrieves the stored account, which requires step-up authentication.
    pub(crate) async fn load_stored_account(&self) -> Result<Option<StorableAccount>> {
        request_stored_account(&self.storage_op_sender).await
    }

    /// Materializes the stored account, which requires step-up authentication.
    pub(crate) async fn load_vpn_api_account(&self) -> Result<VpnAccount> {
        request_vpn_api_account(&self.storage_op_sender).await
    }

    // Set the firewall_active flag to `active`. Returns wethere the status had to be updated or not
    pub(crate) fn set_firewall_state(&mut self, active: bool) -> bool {
        if self.firewall_active != active {
            if active {
                self.current_credential_fetcher.pause();
            } else {
                self.current_credential_fetcher.resume();
            }
            self.firewall_active = active;
            true
        } else {
            false
        }
    }

    /// Ensure the VPN-API credential fetcher is installed on the bandwidth controller. No-op if it is
    /// already the active fetcher.
    pub(crate) async fn use_vpn_api_fetcher(&mut self) -> Result<()> {
        if matches!(
            self.current_credential_fetcher,
            CredentialFetcherInUse::VpnApi(_)
        ) {
            return Ok(());
        }

        let Some(StoredDeviceState {
            account_identity,
            ecash_keypair,
            ..
        }) = &self.device_state
        else {
            return Err(Error::NoAccountStored);
        };

        let Some(device) = self.device.clone() else {
            return Err(Error::NoDeviceStored);
        };

        let fetcher = match VpnApiCredentialFetcher::new(
            self.vpn_api_client.clone(),
            account_identity.id.clone(),
            ecash_keypair.clone(),
            device,
            self.config.data_dir.clone(),
            self.cancel_token.child_token(),
        )
        .await
        {
            Ok(fetcher) => Arc::new(fetcher),
            Err(err) => {
                return Err(Error::internal(format!(
                    "failed to construct VPN-API credential fetcher: {err}"
                )));
            }
        };

        // If we install the fetcher with firewall active, tag it as paused
        if self.firewall_active {
            fetcher.pause();
        }

        self.bandwidth_control_command_tx
            .set_credential_fetcher(fetcher.clone())
            .await
            .map_err(|err| {
                Error::internal(format!(
                    "failed to install credential fetcher on the bandwidth controller: {err}"
                ))
            })?;

        self.current_credential_fetcher = CredentialFetcherInUse::VpnApi(fetcher);
        tracing::info!("installed the VPN-API credential fetcher on the bandwidth controller");
        Ok(())
    }

    /// Ensure the decentralised setup is installed on the bandwidth controller. No-op if it is
    /// already active.
    pub(crate) async fn use_decentralised_fetcher(&mut self) -> Result<()> {
        if matches!(
            self.current_credential_fetcher,
            CredentialFetcherInUse::Decentralised
        ) {
            return Ok(());
        }

        // No check to ensure decentralised account, that is up to the caller
        let account = self.load_vpn_api_account().await.inspect_err(|err| {
            tracing::warn!("cannot install Nyxd credential fetcher: {err}");
        })?;

        let ecash_keypair = self
            .device_state
            .as_ref()
            .map(|state| state.ecash_keypair.clone())
            .ok_or(Error::NoAccountStored)?;

        let client = self.nyxd_client.inner_client(&account.get_mnemonic())?;

        let fetcher = Arc::new(
            NyxdCredentialFetcher::new_with_keypair(
                client,
                &self.config.storage_paths.credential_requests_database_path,
                ecash_keypair,
            )
            .await
            .map_err(|err| {
                Error::internal(format!(
                    "failed to construct Nyxd credential fetcher: {err}"
                ))
            })?,
        );

        if !fetcher
            .check_balance(AvailableTicketbooks::ticketbook_types().len() as u128)
            .await
            .map_err(|err| Error::internal(format!("failed check account balance: {err}")))?
        {
            return Err(Error::InsufficientFunds);
        }

        self.bandwidth_control_command_tx
            .set_credential_fetcher(fetcher.clone())
            .await
            .map_err(|err| {
                Error::internal(format!(
                    "failed to install credential fetcher on the bandwidth controller: {err}"
                ))
            })?;

        self.current_credential_fetcher = CredentialFetcherInUse::Decentralised;
        tracing::info!("installed the Nyxd credential fetcher on the bandwidth controller");
        Ok(())
    }

    /// Remove any credential fetcher from the bandwidth controller. No-op if none is installed.
    pub(crate) async fn clear_credential_fetcher(&mut self) -> Result<()> {
        if matches!(
            self.current_credential_fetcher,
            CredentialFetcherInUse::None
        ) {
            return Ok(());
        }

        self.bandwidth_control_command_tx
            .unset_credential_fetcher()
            .await
            .map_err(|err| {
                Error::internal(format!(
                    "failed to remove credential fetcher from the bandwidth controller: {err}"
                ))
            })?;

        self.current_credential_fetcher = CredentialFetcherInUse::None;
        tracing::info!("removed the credential fetcher from the bandwidth controller");
        Ok(())
    }

    /// Marks the cached summary as stale and persists it. No-op when no summary is cached.
    pub(crate) async fn mark_summary_as_stale(&mut self) -> Result<()> {
        let Some(StoredDeviceState {
            summary: Some(account_summary),
            ..
        }) = &mut self.device_state
        else {
            return Ok(());
        };
        account_summary.stale = true;
        self.persist_device_state().await
    }

    /// Stores the account summary in memory and persists it.
    pub(crate) async fn store_summary(
        &mut self,
        new_account_summary: VpnAccountSummary,
    ) -> Result<()> {
        let Some(StoredDeviceState {
            summary: account_summary,
            ..
        }) = &mut self.device_state
        else {
            return Err(Error::NoAccountStored);
        };
        account_summary.replace(new_account_summary);
        self.persist_device_state().await
    }

    async fn persist_device_state(&self) -> Result<()> {
        let device_state = self.device_state.clone().ok_or(Error::NoAccountStored)?;
        let (tx, rx) = ReturnSender::new();
        self.storage_op_sender
            .send(AccountStorageOp::StoreDeviceState(tx, device_state))
            .map_err(Error::internal)?;
        rx.await.map_err(Error::internal)?
    }
}

pub(crate) enum CredentialFetcherInUse {
    /// Centralised fetcher that acquires zk-nyms from the VPN API.
    VpnApi(Arc<VpnApiCredentialFetcher>),
    /// Decentralised mode, operating independently of the VPN API.
    Decentralised,

    None,
}

impl CredentialFetcherInUse {
    pub fn pause(&self) {
        match self {
            CredentialFetcherInUse::VpnApi(vpn_api_credential_fetcher) => {
                vpn_api_credential_fetcher.pause()
            }
            CredentialFetcherInUse::Decentralised => {
                // explicitly not implemented. No behavior changes as it is only on demand
                // has to be implemented if we want to make it automatic
            }
            CredentialFetcherInUse::None => {}
        }
    }
    pub fn resume(&self) {
        match self {
            CredentialFetcherInUse::VpnApi(vpn_api_credential_fetcher) => {
                vpn_api_credential_fetcher.resume()
            }
            CredentialFetcherInUse::Decentralised => {
                // explicitly not implemented. No behavior changes as it is only on demand
                // has to be implemented if we want to make it automatic
            }
            CredentialFetcherInUse::None => {}
        }
    }
}
