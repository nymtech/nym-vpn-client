use crate::error::BackendError;

use nym_vpn_lib_types as lib;
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
    Decentralised,
    BandwidthExceeded,
    StatusNotActive,
    NoSubscription,
    MaxDeviceReached,
    RequestingZkNyms,
    Error(BackendError),
}

impl AccountState {
    pub fn from_lib(state: lib::AccountControllerState) -> AccountState {
        match state {
            lib::AccountControllerState::LoggedOut => AccountState::LoggedOut,
            lib::AccountControllerState::Syncing => AccountState::Syncing,
            lib::AccountControllerState::ReadyToConnect => AccountState::Ready,
            lib::AccountControllerState::Decentralised => AccountState::Decentralised,
            lib::AccountControllerState::Offline => AccountState::Offline,
            lib::AccountControllerState::RequestingZkNyms(_) => AccountState::RequestingZkNyms,
            lib::AccountControllerState::Error(error) => match error {
                lib::AccountControllerErrorStateReason::BandwidthExceeded {
                    context: _, /* TODO */
                } => AccountState::BandwidthExceeded,
                lib::AccountControllerErrorStateReason::AccountStatusNotActive {
                    status: _, /* TODO */
                } => AccountState::StatusNotActive,
                lib::AccountControllerErrorStateReason::InactiveSubscription => {
                    AccountState::NoSubscription
                }
                lib::AccountControllerErrorStateReason::MaxDeviceReached => {
                    AccountState::MaxDeviceReached
                }
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
