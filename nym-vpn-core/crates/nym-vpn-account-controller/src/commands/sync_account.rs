// Copyright 2024 - Nym Technologies SA<contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use nym_vpn_api_client::{
    response::NymVpnAccountSummaryResponse,
    types::{VpnApiAccount, VpnApiTime},
};
use nym_vpn_lib_types::{SyncAccountError, VpnApiErrorResponse};
use tracing::Level;

use crate::shared_state::{AccountRegistered, AccountSummary, SharedAccountState};

use super::AccountCommandResult;

type PreviousAccountSummaryResponse = Arc<tokio::sync::Mutex<Option<NymVpnAccountSummaryResponse>>>;

pub(crate) struct WaitingSyncAccountCommandHandler {
    account_state: SharedAccountState,
    vpn_api_client: nym_vpn_api_client::VpnApiClient,

    previous_account_summary_response: PreviousAccountSummaryResponse,
}

impl WaitingSyncAccountCommandHandler {
    pub(crate) fn new(
        account_state: SharedAccountState,
        vpn_api_client: nym_vpn_api_client::VpnApiClient,
    ) -> Self {
        WaitingSyncAccountCommandHandler {
            account_state,
            vpn_api_client,
            previous_account_summary_response: Default::default(),
        }
    }

    pub(crate) fn build(&self, account: VpnApiAccount) -> SyncStateCommandHandler {
        let id = uuid::Uuid::new_v4();
        tracing::debug!("Created new sync state command handler: {}", id);
        SyncStateCommandHandler {
            id,
            account,
            account_state: self.account_state.clone(),
            vpn_api_client: self.vpn_api_client.clone(),
            previous_account_summary_response: self.previous_account_summary_response.clone(),
        }
    }

    pub(crate) fn update_vpn_api_client(
        &mut self,
        vpn_api_client: nym_vpn_api_client::VpnApiClient,
    ) {
        self.vpn_api_client = vpn_api_client;
    }
}

pub(crate) struct SyncStateCommandHandler {
    id: uuid::Uuid,
    account: VpnApiAccount,
    account_state: SharedAccountState,
    vpn_api_client: nym_vpn_api_client::VpnApiClient,

    previous_account_summary_response: PreviousAccountSummaryResponse,
}

impl SyncStateCommandHandler {
    pub(crate) async fn run(self) -> AccountCommandResult {
        AccountCommandResult::SyncAccountState(self.run_inner().await)
    }

    fn id_str(&self) -> String {
        format!("{:.8}", self.id.to_string())
    }

    #[tracing::instrument(
        skip(self),
        name = "sync_account",
        fields(id = %self.id_str()),
        ret,
        err,
        level = Level::DEBUG,
    )]
    pub(crate) async fn run_inner(self) -> Result<NymVpnAccountSummaryResponse, SyncAccountError> {
        tracing::debug!("Running sync account state command handler: {}", self.id);
        let update_result = update_state(
            &self.account,
            &self.account_state,
            &self.vpn_api_client,
            &self.previous_account_summary_response,
        )
        .await;

        tracing::debug!("Current state: {:?}", self.account_state.lock().await);
        update_result
    }
}

fn handle_remote_time(remote_time: VpnApiTime) {
    if remote_time.is_synced() {
        tracing::info!("{remote_time}");
    } else {
        tracing::warn!(
            "The time skew between the local and remote time is too large ({remote_time})."
        );
    }
}

async fn update_state(
    account: &VpnApiAccount,
    account_state: &SharedAccountState,
    vpn_api_client: &nym_vpn_api_client::VpnApiClient,
    previous_account_summary_response: &PreviousAccountSummaryResponse,
) -> Result<NymVpnAccountSummaryResponse, SyncAccountError> {
    tracing::debug!("Updating account state");
    let (remote_time, response) = tokio::join!(
        vpn_api_client.get_remote_time(),
        vpn_api_client.get_account_summary(account)
    );

    match remote_time {
        Ok(remote_time) => handle_remote_time(remote_time),
        Err(err) => tracing::error!("Failed to get remote time: {err}"),
    }

    let account_summary = match response {
        Ok(account_summary) => account_summary,
        Err(err) => {
            account_state
                .set_account_registered(AccountRegistered::NotRegistered)
                .await;
            return Err(VpnApiErrorResponse::try_from(err)
                .map(SyncAccountError::SyncAccountEndpointFailure)
                .unwrap_or_else(SyncAccountError::unexpected_response));
        }
    };

    if previous_account_summary_response
        .lock()
        .await
        .replace(account_summary.clone())
        .as_ref()
        != Some(&account_summary)
    {
        tracing::debug!("Synced account summary: {:#?}", account_summary);
    }

    account_state
        .set_account_registered(AccountRegistered::Registered)
        .await;

    account_state
        .set_account_summary(AccountSummary::from(account_summary.clone()))
        .await;

    Ok(account_summary)
}
