// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::shared_state::SharedAccountState;
use nym_credential_utils::utils::issue_credential;
use nym_credentials_interface::TicketType;
use nym_offline_monitor::ConnectivityMonitor;
use nym_validator_client::nyxd::{Coin, CosmWasmClient, contract_traits::EcashQueryClient};
use nym_vpn_lib_types::AccountCommandError;
use tracing::{debug, info, warn};

const TICKET_TYPES: &[TicketType] = &[
    TicketType::V1MixnetEntry,
    TicketType::V1WireguardEntry,
    TicketType::V1WireguardExit,
];
// be quite conservative about it
const DEPOSIT_TX_FEE_AMOUNT: u128 = 50_000;

pub(crate) async fn handle_obtain_ticketbooks<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
    amount: u64,
) -> Result<(), AccountCommandError> {
    info!("attempting to obtain {amount} ticketbooks of each type");
    let Some(account) = shared_state.vpn_api_account.as_ref() else {
        return Err(AccountCommandError::NoAccountStored);
    };

    let ecash_seed = account.ecash_keypair_seed().map_err(|err| {
        AccountCommandError::Internal(format!("ecash seed derivation failure: {err}"))
    })?;
    let client = shared_state
        .nyxd_client
        .inner_client(&account.get_mnemonic())?;

    // determine required deposits amount and funds and ensure we have enough
    // (plus a bit more for tx fees)
    let deposits_amount = amount * TICKET_TYPES.len() as u64;
    let raw_deposit_cost = client
        .get_reduced_deposit_amount(client.address().to_string())
        .await?
        .unwrap_or(client.get_default_deposit_amount().await?);
    let per_deposit_cost_amount = raw_deposit_cost.amount.u128() + DEPOSIT_TX_FEE_AMOUNT;
    let total_deposits_cost = Coin::new(
        per_deposit_cost_amount * deposits_amount as u128,
        raw_deposit_cost.denom,
    );

    info!(
        "this will require {deposits_amount} deposits that will cost approximately {total_deposits_cost}"
    );

    let available_balance = client
        .get_balance(&client.address(), total_deposits_cost.denom.clone())
        .await?
        .unwrap_or_else(|| Coin::new(0, &total_deposits_cost.denom));
    debug!("available_balance: {available_balance}");

    if available_balance.amount < total_deposits_cost.amount {
        warn!(
            "insufficient funds for obtaining desired amount of ticketbooks. available: {available_balance} required (approximately): {total_deposits_cost}"
        );
        return Err(AccountCommandError::InsufficientFunds);
    }

    let credential_storage = shared_state.credential_storage.credential_storage();

    // TODO: future optimisation:
    // technically we could make all deposits in a single transaction thus making the whole thing an order
    // of magnitude faster, but:
    // - this handler is called very infrequently
    // - it's a temporary solution anyway
    // thus rather than re-inventing the wheel to make this happen,
    // we use the helper that obtains one ticketbook at a time
    // we can optimise it later.
    for i in 0..amount {
        for ticket_type in TICKET_TYPES {
            info!("obtaining ticketbook set {}/{amount}: {ticket_type}", i + 1);
            issue_credential(client, credential_storage, &ecash_seed, *ticket_type)
                .await
                .map_err(|err| AccountCommandError::ZkNymAcquisitionFailure(err.to_string()))?;
        }
    }

    Ok(())
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
