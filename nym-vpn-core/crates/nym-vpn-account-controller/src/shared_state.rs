// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_offline_monitor::ConnectivityHandle;
use nym_vpn_api_client::VpnApiClient;
use nym_vpn_api_client::types::{Device, VpnApiAccount};

use tokio::sync::mpsc;

use crate::{
    AccountControllerConfig,
    storage::{AccountStorageOp, VpnCredentialStorage},
};

pub(crate) struct SharedAccountState {
    // Ideally, we would have tunnel state here. But it makes circular dependency where tunnel needs AC state and AC needs tunnel state
    pub connectivity_handle: ConnectivityHandle,

    pub config: AccountControllerConfig,

    // This is bound to live in the bandwidth controller in a near future
    pub(crate) credential_storage: VpnCredentialStorage,

    pub(crate) vpn_api_client: VpnApiClient,

    pub(crate) vpn_api_account: Option<VpnApiAccount>,

    pub(crate) device: Option<Device>,

    pub(crate) storage_op_sender: mpsc::UnboundedSender<AccountStorageOp>,
}

impl SharedAccountState {
    pub(crate) fn new(
        connectivity_handle: ConnectivityHandle,
        config: AccountControllerConfig,
        credential_storage: VpnCredentialStorage,
        vpn_api_client: VpnApiClient,
        vpn_api_account: Option<VpnApiAccount>,
        device: Option<Device>,
        storage_op_sender: mpsc::UnboundedSender<AccountStorageOp>,
    ) -> Self {
        SharedAccountState {
            connectivity_handle,
            config,
            credential_storage,
            vpn_api_client,
            vpn_api_account,
            device,
            storage_op_sender,
        }
    }
}
