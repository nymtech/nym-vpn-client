// Copyright 2024 - Nym Technologies SA<contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use nym_vpn_api_client::{response::NymVpnAccountSummaryResponse, types::VpnApiAccount};
use tracing::Level;

use crate::{
    commands::VpnApiEndpointFailure,
    shared_state::{AccountRegistered, AccountSummary, SharedAccountState},
};

use super::{AccountCommandError, AccountCommandResult};

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
    pub(crate) async fn run_inner(
        self,
    ) -> Result<NymVpnAccountSummaryResponse, AccountCommandError> {
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

async fn update_state(
    account: &VpnApiAccount,
    account_state: &SharedAccountState,
    vpn_api_client: &nym_vpn_api_client::VpnApiClient,
    previous_account_summary_response: &PreviousAccountSummaryResponse,
) -> Result<NymVpnAccountSummaryResponse, AccountCommandError> {
    tracing::debug!("Updating account state");
    let response = vpn_api_client.get_account_summary(account).await;

    let account_summary = match response {
        Ok(account_summary) => account_summary,
        Err(err) => {
            if let Some(e) = nym_vpn_api_client::response::extract_error_response(&err) {
                tracing::warn!(message = %e.message, message_id=?e.message_id, code_reference_id=?e.code_reference_id, "nym-vpn-api reports");
                // TODO: check the message_id to confirm it's an error saying we are not registered
                account_state
                    .set_account_registered(AccountRegistered::NotRegistered)
                    .await;
                return Err(AccountCommandError::SyncAccountEndpointFailure(
                    VpnApiEndpointFailure {
                        message: e.message.clone(),
                        message_id: e.message_id.clone(),
                        code_reference_id: e.code_reference_id.clone(),
                    },
                ));
            }
            return Err(AccountCommandError::General(err.to_string()));
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
