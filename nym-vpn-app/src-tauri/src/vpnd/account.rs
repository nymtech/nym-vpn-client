use crate::error::BackendError;

use nym_vpn_lib_types as lib;
use serde::Serialize;
use tracing::{debug, instrument};
use tracing::{info, warn};
use ts_rs::TS;

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
    UpgradeMode,
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
            lib::AccountControllerState::UpgradeMode => AccountState::UpgradeMode,
            lib::AccountControllerState::Offline => AccountState::Offline,
            lib::AccountControllerState::RequestingZkNyms => AccountState::RequestingZkNyms,
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
pub fn log_account_state(state: &lib::AccountControllerState) {
    match state {
        lib::AccountControllerState::Error(e) => match e {
            lib::AccountControllerErrorStateReason::Storage { context, details } => {
                warn!("account state error: {e:?}, context: {context}, details: {details}")
            }
            lib::AccountControllerErrorStateReason::ApiFailure { context, details }
            | lib::AccountControllerErrorStateReason::Internal { context, details } => {
                warn!("account state error: {e:?}, context: {context}, details: {details}",)
            }
            lib::AccountControllerErrorStateReason::DeviceTimeDesynced => {
                warn!("account state error: {e:?}")
            }
            _ => info!("account state error: {e:?}"),
        },
        _ => debug!("account state: [{state:?}]"),
    }
}

// #[derive(strum::AsRefStr, Default, Serialize, Clone, Debug, TS)]
#[derive(Serialize, Clone, Debug, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts", rename = "TAccountMode")]
#[serde(rename_all = "kebab-case")]
pub enum StoredAccountMode {
    // #[default]
    Privy,
    Decentralised,
    Api,
}

// impl AccountMode {
//     pub fn from_lib(mode: lib::AccountMode) -> AccountMode {
//         match mode {
//             lib::AccountMode::Privy => AccountMode::Privy,
//             lib::AccountMode::Decentralised => AccountMode::Decentralised,
//             lib::AccountMode::Api => AccountMode::Api,
//         }
//     }
// }
impl From<lib::StoredAccountMode> for StoredAccountMode {
    fn from(mode: lib::StoredAccountMode) -> Self {
        match mode {
            lib::StoredAccountMode::Privy => StoredAccountMode::Privy,
            lib::StoredAccountMode::Decentralised => StoredAccountMode::Decentralised,
            lib::StoredAccountMode::Api => StoredAccountMode::Api,
        }
    }
}