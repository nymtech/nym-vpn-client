// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::{AccountControllerErrorStateReason, AccountControllerState};

use crate::{conversions::ConversionError, proto};

impl From<proto::account_controller_state::Error> for AccountControllerErrorStateReason {
    fn from(value: proto::account_controller_state::Error) -> Self {
        match value.reason() {
            proto::account_controller_state::ErrorStateReason::Storage => {
                AccountControllerErrorStateReason::Storage {
                    context: value.context.unwrap_or_default(),
                    details: value.details.unwrap_or_default(),
                }
            }
            proto::account_controller_state::ErrorStateReason::ApiFailure => {
                AccountControllerErrorStateReason::ApiFailure {
                    context: value.context.unwrap_or_default(),
                    details: value.details.unwrap_or_default(),
                }
            }
            proto::account_controller_state::ErrorStateReason::Internal => {
                AccountControllerErrorStateReason::Internal {
                    context: value.context.unwrap_or_default(),
                    details: value.details.unwrap_or_default(),
                }
            }
            proto::account_controller_state::ErrorStateReason::BandwidthExceeded => {
                AccountControllerErrorStateReason::BandwidthExceeded {
                    context: value.context.unwrap_or_default(),
                }
            }
            proto::account_controller_state::ErrorStateReason::AccountStatusNotActive => {
                AccountControllerErrorStateReason::AccountStatusNotActive {
                    status: value.details.unwrap_or_default(),
                }
            }
            proto::account_controller_state::ErrorStateReason::InactiveSubscription => {
                AccountControllerErrorStateReason::InactiveSubscription
            }
            proto::account_controller_state::ErrorStateReason::MaxDeviceReached => {
                AccountControllerErrorStateReason::MaxDeviceReached
            }
            proto::account_controller_state::ErrorStateReason::DeviceTimeDesynced => {
                AccountControllerErrorStateReason::DeviceTimeDesynced
            }
        }
    }
}

impl From<AccountControllerErrorStateReason> for proto::account_controller_state::Error {
    fn from(value: AccountControllerErrorStateReason) -> Self {
        match value {
            AccountControllerErrorStateReason::Storage { context, details } => {
                proto::account_controller_state::Error {
                    reason: proto::account_controller_state::ErrorStateReason::Storage.into(),
                    context: Some(context),
                    details: Some(details),
                }
            }
            AccountControllerErrorStateReason::ApiFailure { context, details } => {
                proto::account_controller_state::Error {
                    reason: proto::account_controller_state::ErrorStateReason::ApiFailure.into(),
                    context: Some(context),
                    details: Some(details),
                }
            }
            AccountControllerErrorStateReason::Internal { context, details } => {
                proto::account_controller_state::Error {
                    reason: proto::account_controller_state::ErrorStateReason::Internal.into(),
                    context: Some(context),
                    details: Some(details),
                }
            }
            AccountControllerErrorStateReason::BandwidthExceeded { context } => {
                proto::account_controller_state::Error {
                    reason: proto::account_controller_state::ErrorStateReason::BandwidthExceeded
                        .into(),
                    context: Some(context),
                    details: None,
                }
            }
            AccountControllerErrorStateReason::AccountStatusNotActive { status } => {
                proto::account_controller_state::Error {
                    reason:
                        proto::account_controller_state::ErrorStateReason::AccountStatusNotActive
                            .into(),
                    context: None,
                    details: Some(status),
                }
            }
            AccountControllerErrorStateReason::InactiveSubscription => {
                proto::account_controller_state::Error {
                    reason: proto::account_controller_state::ErrorStateReason::InactiveSubscription
                        .into(),
                    context: None,
                    details: None,
                }
            }
            AccountControllerErrorStateReason::MaxDeviceReached => {
                proto::account_controller_state::Error {
                    reason: proto::account_controller_state::ErrorStateReason::MaxDeviceReached
                        .into(),
                    context: None,
                    details: None,
                }
            }
            AccountControllerErrorStateReason::DeviceTimeDesynced => {
                proto::account_controller_state::Error {
                    reason: proto::account_controller_state::ErrorStateReason::DeviceTimeDesynced
                        .into(),
                    context: None,
                    details: None,
                }
            }
        }
    }
}

impl From<AccountControllerState> for proto::AccountControllerState {
    fn from(state: AccountControllerState) -> Self {
        let proto_state: proto::account_controller_state::State = match state {
            AccountControllerState::LoggedOut => proto::account_controller_state::State::LoggedOut(
                proto::account_controller_state::LoggedOut {},
            ),
            AccountControllerState::ReadyToConnect => {
                proto::account_controller_state::State::ReadyToConnect(
                    proto::account_controller_state::ReadyToConnect {},
                )
            }
            AccountControllerState::Syncing => proto::account_controller_state::State::Syncing(
                proto::account_controller_state::Syncing {},
            ),
            AccountControllerState::Offline => proto::account_controller_state::State::Offline(
                proto::account_controller_state::Offline {},
            ),
            AccountControllerState::Decentralised => {
                proto::account_controller_state::State::Decentralised(
                    proto::account_controller_state::Decentralised {},
                )
            }
            AccountControllerState::PendingSubscription => {
                proto::account_controller_state::State::PendingSubscription(
                    proto::account_controller_state::PendingSubscription {},
                )
            }
            AccountControllerState::Error(reason) => proto::account_controller_state::State::Error(
                proto::account_controller_state::Error::from(reason),
            ),
        };

        proto::AccountControllerState {
            state: Some(proto_state),
        }
    }
}

impl TryFrom<proto::AccountControllerState> for AccountControllerState {
    type Error = ConversionError;

    fn try_from(value: proto::AccountControllerState) -> Result<Self, ConversionError> {
        let state = value
            .state
            .ok_or(ConversionError::NoValueSet("AccountControllerState.state"))?;

        Ok(match state {
            proto::account_controller_state::State::LoggedOut(
                proto::account_controller_state::LoggedOut {},
            ) => Self::LoggedOut,
            proto::account_controller_state::State::ReadyToConnect(
                proto::account_controller_state::ReadyToConnect {},
            ) => Self::ReadyToConnect,
            proto::account_controller_state::State::Syncing(
                proto::account_controller_state::Syncing {},
            ) => Self::Syncing,
            proto::account_controller_state::State::Error(error_state_reason) => {
                Self::Error(error_state_reason.into())
            }
            proto::account_controller_state::State::Offline(
                proto::account_controller_state::Offline {},
            ) => Self::Offline,
            proto::account_controller_state::State::Decentralised(
                proto::account_controller_state::Decentralised {},
            ) => Self::Decentralised,
            proto::account_controller_state::State::PendingSubscription(
                proto::account_controller_state::PendingSubscription {},
            ) => Self::PendingSubscription,
        })
    }
}
