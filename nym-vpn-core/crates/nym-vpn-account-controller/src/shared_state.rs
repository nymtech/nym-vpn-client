// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_offline_monitor::ConnectivityMonitor;
use nym_vpn_api_client::{
    VpnApiClient,
    types::{Device, VpnAccount},
};

use tokio::sync::mpsc;

use crate::nyxd_client::NyxdClient;
use crate::{
    AccountControllerConfig,
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

    /// VPN API client
    pub(crate) vpn_api_client: VpnApiClient,

    /// Nyxd RPC client
    pub(crate) nyxd_client: NyxdClient,

    /// Stored account
    pub(crate) vpn_api_account: Option<VpnAccount>,

    /// Registered device
    pub(crate) device: Option<Device>,

    /// Firewall status
    pub(crate) firewall_active: bool,

    /// Channel to send storage operation to the AccountController
    pub(crate) storage_op_sender: mpsc::UnboundedSender<AccountStorageOp>,
}

impl<C: ConnectivityMonitor> SharedAccountState<C> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        connectivity_handle: C,
        config: AccountControllerConfig,
        credential_storage: VpnCredentialStorage,
        vpn_api_client: VpnApiClient,
        nyxd_client: NyxdClient,
        vpn_api_account: Option<VpnAccount>,
        device: Option<Device>,
        storage_op_sender: mpsc::UnboundedSender<AccountStorageOp>,
    ) -> Self {
        SharedAccountState {
            connectivity_handle,
            config,
            credential_storage,
            vpn_api_client,
            nyxd_client,
            vpn_api_account,
            device,
            firewall_active: false,
            storage_op_sender,
        }
    }
}
