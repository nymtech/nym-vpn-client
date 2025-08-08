use crate::error::BackendError;

use serde::Serialize;
use tracing::warn;
use tracing::{debug, instrument};
use ts_rs::TS;

use nym_vpn_proto::proto::account_controller_state::{State as ProtoState, State};

#[derive(strum::AsRefStr, Default, Serialize, Clone, Debug, TS)]
#[ts(export)]
#[serde(rename_all = "kebab-case")]
pub enum AccountState {
    #[default]
    Ready,
    LoggedOut,
    Syncing,
    Offline,
    Error(BackendError),
}

impl AccountState {
    pub fn from_proto(state: ProtoState) -> AccountState {
        match state {
            State::LoggedOut(_) => AccountState::LoggedOut,
            State::Syncing(_) => AccountState::Syncing,
            State::ReadyToConnect(_) => AccountState::Ready,
            State::Offline(_) => AccountState::Offline,
            State::Error(error) => AccountState::Error(error.into()),
        }
    }
}

#[instrument]
pub fn log_account_state(state: &ProtoState) {
    match state {
        ProtoState::Error(e) => {
            warn!(
                "account state error: [{:?}] - details: {} - context: {}",
                e.reason(),
                e.details(),
                e.context()
            );
        }
        _ => debug!("account state: [{:?}]", state),
    }
}
