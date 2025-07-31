// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_offline_monitor::ConnectivityHandle;
use nym_vpn_api_client::types::{Device, VpnApiAccount};

use tokio::sync::mpsc;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::{
    AccountControllerConfig,
    storage::{AccountStorageOp, VpnCredentialStorage},
    vpn_api_client::AccountControllerVpnApiClient,
};

#[derive(Zeroize, ZeroizeOnDrop)]
pub(crate) struct SharedAccountState {
    // SW add tunnel state? Yes, to remove some conditions on forget account and reset device id
    #[zeroize(skip)]
    pub connectivity_handle: ConnectivityHandle,

    #[zeroize(skip)]
    pub config: AccountControllerConfig,

    #[zeroize(skip)]
    pub(crate) credential_storage: VpnCredentialStorage,

    #[zeroize(skip)]
    pub(crate) vpn_api_client: AccountControllerVpnApiClient,

    pub(crate) vpn_api_account: Option<VpnApiAccount>,

    #[zeroize(skip)]
    pub(crate) device: Option<Device>,

    #[zeroize(skip)]
    pub(crate) storage_op_sender: mpsc::UnboundedSender<AccountStorageOp>,
}

impl SharedAccountState {
    pub(crate) fn new(
        connectivity_handle: ConnectivityHandle,
        config: AccountControllerConfig,
        credential_storage: VpnCredentialStorage,
        vpn_api_client: AccountControllerVpnApiClient,
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
