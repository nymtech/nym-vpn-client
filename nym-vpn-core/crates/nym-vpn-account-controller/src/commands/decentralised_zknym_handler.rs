// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::shared_state::SharedAccountState;
use nym_bandwidth_controller::AvailableTicketbooks;

use nym_offline_monitor::ConnectivityMonitor;
use nym_validator_client::nyxd::Coin;
use nym_vpn_lib_types::AccountCommandError;
use tracing::info;

pub(crate) async fn handle_obtain_ticketbooks<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
) -> Result<(), AccountCommandError> {
    info!("attempting to obtain ticketbooks of each type");

    shared_state.use_decentralised_fetcher().await?;

    // We need to unconditionnaly remove the fetcher once it's done
    let result = shared_state
        .bandwidth_control_command_tx
        .wait_for_ticketbooks(AvailableTicketbooks::ticketbook_types())
        .await
        .map_err(|e| AccountCommandError::ZkNymAcquisitionFailure(format!("{e:?}")));

    shared_state.clear_credential_fetcher().await?;

    result
}

pub(crate) async fn handle_account_balance<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
) -> Result<Vec<Coin>, AccountCommandError> {
    info!("retrieving account balance");
    let Some(account) = shared_state.vpn_api_account.as_ref() else {
        return Err(AccountCommandError::NoAccountStored);
    };
    shared_state
        .nyxd_client
        .account_balance(&account.get_mnemonic())
        .await
}
