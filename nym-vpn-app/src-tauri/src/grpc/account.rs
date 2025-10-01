use crate::error::BackendError;

use serde::Serialize;
use tracing::{debug, instrument};
use tracing::{info, warn};
use ts_rs::TS;

use nym_vpn_proto::proto::account_controller_state::{
    ErrorStateReason, State as ProtoState, State,
};

#[derive(strum::AsRefStr, Default, Serialize, Clone, Debug, TS)]
#[ts(export, export_to = "tauri.ts", rename = "TAccountState")]
#[serde(rename_all = "kebab-case")]
pub enum AccountState {
    #[default]
    Ready,
    LoggedOut,
    Syncing,
    Offline,
    BandwidthExceeded,
    StatusNotActive,
    NoSubscription,
    MaxDeviceReached,
    Error(BackendError),
}

impl AccountState {
    pub fn from_proto(state: ProtoState) -> AccountState {
        match state {
            State::LoggedOut(_) => AccountState::LoggedOut,
            State::Syncing(_) => AccountState::Syncing,
            State::ReadyToConnect(_) => AccountState::Ready,
            State::Offline(_) => AccountState::Offline,
            State::Error(error) => match error.reason() {
                ErrorStateReason::BandwidthExceeded => AccountState::BandwidthExceeded,
                ErrorStateReason::AccountStatusNotActive => AccountState::StatusNotActive,
                ErrorStateReason::InactiveSubscription => AccountState::NoSubscription,
                ErrorStateReason::MaxDeviceReached => AccountState::MaxDeviceReached,
                _ => AccountState::Error(error.into()),
            },
        }
    }
}

#[instrument]
pub fn log_account_state(state: &ProtoState) {
    match state {
        ProtoState::Error(e) => {
            let message = format!(
                "account state error: [{:?}] - details: {} - context: {}",
                e.reason(),
                e.details(),
                e.context()
            );
            match e.reason() {
                ErrorStateReason::Storage
                | ErrorStateReason::ApiFailure
                | ErrorStateReason::Internal
                | ErrorStateReason::DeviceTimeDesynced => {
                    warn!(message);
                }
                _ => info!(message),
            }
        }
        _ => debug!("account state: [{:?}]", state),
    }
}
