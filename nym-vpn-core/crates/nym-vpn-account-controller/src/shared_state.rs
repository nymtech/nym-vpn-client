// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_offline_monitor::ConnectivityMonitor;
use nym_vpn_api_client::{
    VpnApiClient,
    types::{Device, VpnAccount},
};
use nym_vpn_lib_types::VpnAccountSummary;
use std::sync::Arc;

use nym_vpn_store::keys::wireguard::WireguardKeysDb;
use tokio::sync::mpsc;

use crate::{
    AccountControllerConfig, AccountControllerEventSender,
    deeplink::Deeplinks,
    nyxd_client::NyxdClient,
    storage::{AccountStorageOp, VpnCredentialStorage},
};

/// This shared state is the sole propriety of the AccountController and contains the element that must be passed around the different states
/// Ideally, we would have tunnel state here. But it makes circular dependency where tunnel needs AC state and AC needs tunnel state
pub(crate) struct SharedAccountState<C: ConnectivityMonitor> {
    /// Offline monitoring
    pub connectivity_handle: C,

    /// Config for the account controller
    pub config: AccountControllerConfig,

    // This is bound to live in the bandwidth controller in a near future
    /// Zk-nym storage
    pub(crate) credential_storage: VpnCredentialStorage,

    /// Wireguard keys database storage
    pub(crate) wireguard_keys_storage: WireguardKeysDb,

    /// VPN API client
    pub(crate) vpn_api_client: VpnApiClient,

    /// Nyxd RPC client
    pub(crate) nyxd_client: NyxdClient,

    /// Stored account
    pub(crate) vpn_api_account: Option<Arc<VpnAccount>>,

    /// Account summary. The persistent copy lives in the platform storage (see
    /// [`AccountSummaryStorage`](nym_vpn_store::account_summary::AccountSummaryStorage)); this is the
    /// in-memory working copy, kept in sync via the storage-op channel.
    pub(crate) vpn_account_summary: Option<VpnAccountSummary>,

    /// Registered device
    pub(crate) device: Option<Device>,

    /// Deeplinks for signing-in via services like Privy
    pub(crate) deeplinks: Deeplinks,

    /// Firewall status
    pub(crate) firewall_active: bool,

    /// Channel to send storage operation to the AccountController
    pub(crate) storage_op_sender: mpsc::UnboundedSender<AccountStorageOp>,

    /// Channel for broadcasting global `AccountController` events
    pub(crate) event_sender: AccountControllerEventSender,
}

impl<C: ConnectivityMonitor> SharedAccountState<C> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        connectivity_handle: C,
        config: AccountControllerConfig,
        credential_storage: VpnCredentialStorage,
        wireguard_keys_storage: WireguardKeysDb,
        vpn_api_client: VpnApiClient,
        nyxd_client: NyxdClient,
        vpn_api_account: Option<VpnAccount>,
        vpn_account_summary: Option<VpnAccountSummary>,
        device: Option<Device>,
        storage_op_sender: mpsc::UnboundedSender<AccountStorageOp>,
        event_sender: AccountControllerEventSender,
    ) -> Self {
        let deeplinks = Deeplinks::default();

        Self {
            connectivity_handle,
            config,
            credential_storage,
            wireguard_keys_storage,
            vpn_api_client,
            nyxd_client,
            vpn_api_account: vpn_api_account.map(Arc::new),
            vpn_account_summary,
            device,
            deeplinks,
            firewall_active: false,
            storage_op_sender,
            event_sender,
        }
    }

    // Mark the summary as stale and best effort to propagate to disk
    pub(crate) fn mark_summary_as_stale(&mut self) {
        if let Some(summary) = self.vpn_account_summary.as_mut() {
            summary.stale = true;
            let _ = self
                .storage_op_sender
                .send(AccountStorageOp::StoreAccountSummary(Box::new(
                    summary.clone(),
                )));
        }
    }

    // Store the account summary in memory and best effort to propagate to disk
    pub(crate) fn store_summary(&mut self, summary: VpnAccountSummary) {
        let _ = self
            .storage_op_sender
            .send(AccountStorageOp::StoreAccountSummary(Box::new(
                summary.clone(),
            )));
        self.vpn_account_summary = Some(summary);
    }
}
