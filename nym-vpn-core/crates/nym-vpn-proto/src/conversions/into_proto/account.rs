// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

impl From<Option<String>> for crate::AccountIdentity {
    fn from(identity: Option<String>) -> Self {
        Self {
            account_identity: identity,
        }
    }
}

impl From<nym_vpn_account_controller::ReadyToConnect> for crate::IsReadyToConnectResponse {
    fn from(ready: nym_vpn_account_controller::ReadyToConnect) -> Self {
        use crate::is_ready_to_connect_response::IsReadyToConnectResponseType;
        use nym_vpn_account_controller::ReadyToConnect;
        let kind = match ready {
            ReadyToConnect::Ready => IsReadyToConnectResponseType::Ready,
            ReadyToConnect::NoMnemonicStored => IsReadyToConnectResponseType::NoAccountStored,
            ReadyToConnect::AccountNotSynced => IsReadyToConnectResponseType::AccountNotSynced,
            ReadyToConnect::AccountNotRegistered => {
                IsReadyToConnectResponseType::AccountNotRegistered
            }
            ReadyToConnect::AccountNotActive => IsReadyToConnectResponseType::AccountNotActive,
            ReadyToConnect::NoActiveSubscription => {
                IsReadyToConnectResponseType::NoActiveSubscription
            }
            ReadyToConnect::DeviceNotRegistered => {
                IsReadyToConnectResponseType::DeviceNotRegistered
            }
            ReadyToConnect::DeviceNotActive => IsReadyToConnectResponseType::DeviceNotActive,
        } as i32;
        Self { kind }
    }
}

impl From<nym_vpn_account_controller::shared_state::MnemonicState> for crate::MnemonicState {
    fn from(mnemonic: nym_vpn_account_controller::shared_state::MnemonicState) -> Self {
        match mnemonic {
            nym_vpn_account_controller::shared_state::MnemonicState::Stored { .. } => {
                crate::MnemonicState::Stored
            }
            nym_vpn_account_controller::shared_state::MnemonicState::NotStored => {
                crate::MnemonicState::NotStored
            }
        }
    }
}

impl From<nym_vpn_account_controller::shared_state::AccountRegistered>
    for crate::AccountRegistered
{
    fn from(
        account_registered: nym_vpn_account_controller::shared_state::AccountRegistered,
    ) -> Self {
        match account_registered {
            nym_vpn_account_controller::shared_state::AccountRegistered::Registered => {
                crate::AccountRegistered::AccountRegistered
            }
            nym_vpn_account_controller::shared_state::AccountRegistered::NotRegistered => {
                crate::AccountRegistered::AccountNotRegistered
            }
        }
    }
}

impl From<nym_vpn_account_controller::shared_state::AccountState> for crate::AccountState {
    fn from(account: nym_vpn_account_controller::shared_state::AccountState) -> Self {
        match account {
            nym_vpn_account_controller::shared_state::AccountState::Inactive => {
                crate::AccountState::Inactive
            }
            nym_vpn_account_controller::shared_state::AccountState::Active => {
                crate::AccountState::Active
            }
            nym_vpn_account_controller::shared_state::AccountState::DeleteMe => {
                crate::AccountState::DeleteMe
            }
        }
    }
}

impl From<nym_vpn_account_controller::shared_state::SubscriptionState>
    for crate::SubscriptionState
{
    fn from(subscription: nym_vpn_account_controller::shared_state::SubscriptionState) -> Self {
        match subscription {
            nym_vpn_account_controller::shared_state::SubscriptionState::NotActive => {
                crate::SubscriptionState::NotRegistered
            }
            nym_vpn_account_controller::shared_state::SubscriptionState::Pending => {
                crate::SubscriptionState::Pending
            }
            nym_vpn_account_controller::shared_state::SubscriptionState::Active => {
                crate::SubscriptionState::Active
            }
            nym_vpn_account_controller::shared_state::SubscriptionState::Complete => {
                crate::SubscriptionState::Complete
            }
        }
    }
}

impl From<nym_vpn_account_controller::shared_state::DeviceSummary> for crate::DeviceSummary {
    fn from(device_summary: nym_vpn_account_controller::shared_state::DeviceSummary) -> Self {
        Self {
            active: device_summary.active,
            max: device_summary.max,
            remaining: device_summary.remaining,
        }
    }
}

impl From<nym_vpn_account_controller::shared_state::AccountSummary> for crate::AccountSummary {
    fn from(account_summary: nym_vpn_account_controller::shared_state::AccountSummary) -> Self {
        Self {
            account: crate::AccountState::from(account_summary.account) as i32,
            subscription: crate::SubscriptionState::from(account_summary.subscription) as i32,
            device_summary: Some(crate::DeviceSummary::from(account_summary.device_summary)),
        }
    }
}

impl From<nym_vpn_account_controller::shared_state::DeviceState> for crate::DeviceState {
    fn from(device: nym_vpn_account_controller::shared_state::DeviceState) -> Self {
        match device {
            nym_vpn_account_controller::shared_state::DeviceState::NotRegistered => {
                crate::DeviceState::NotRegistered
            }
            nym_vpn_account_controller::shared_state::DeviceState::Inactive => {
                crate::DeviceState::Inactive
            }
            nym_vpn_account_controller::shared_state::DeviceState::Active => {
                crate::DeviceState::Active
            }
            nym_vpn_account_controller::shared_state::DeviceState::DeleteMe => {
                crate::DeviceState::DeleteMe
            }
        }
    }
}

impl From<nym_vpn_account_controller::shared_state::RegisterDeviceResult>
    for crate::RegisterDeviceResult
{
    fn from(
        device_registration: nym_vpn_account_controller::shared_state::RegisterDeviceResult,
    ) -> Self {
        match device_registration {
            nym_vpn_account_controller::shared_state::RegisterDeviceResult::InProgress => Self {
                kind: crate::register_device_result::RegisterDeviceResultType::InProgress as i32,
                ..Default::default()
            },
            nym_vpn_account_controller::shared_state::RegisterDeviceResult::Success => Self {
                kind: crate::register_device_result::RegisterDeviceResultType::Success as i32,
                ..Default::default()
            },
            nym_vpn_account_controller::shared_state::RegisterDeviceResult::Failed(err) => match err
            {
                nym_vpn_account_controller::RegisterDeviceError::RegisterDeviceEndpointFailure(
                    err,
                ) => Self {
                    kind: crate::register_device_result::RegisterDeviceResultType::Failed as i32,
                    message: Some(err.message),
                    message_id: err.message_id,
                },
                nym_vpn_account_controller::RegisterDeviceError::General(err) => Self {
                    kind: crate::register_device_result::RegisterDeviceResultType::Failed as i32,
                    message: Some(err),
                    message_id: None,
                },
            },
        }
    }
}

impl From<nym_vpn_account_controller::RequestZkNymSuccess> for crate::RequestZkNymSuccess {
    fn from(request_success: nym_vpn_account_controller::RequestZkNymSuccess) -> Self {
        Self {
            id: request_success.id.to_string(),
        }
    }
}

impl From<nym_vpn_account_controller::RequestZkNymError> for crate::RequestZkNymError {
    fn from(error: nym_vpn_account_controller::RequestZkNymError) -> Self {
        match error {
            nym_vpn_account_controller::RequestZkNymError::GetZkNymsAvailableForDownloadEndpointFailure {
                source,
            } => Self {
                kind: crate::request_zk_nym_error::RequestZkNymErrorType::GetZkNymsAvailableForDownloadEndpointFailure as i32,
                id: None,
                ticketbook_type: None,
                message: Some(source.message.clone()),
                message_id: source.message_id.clone(),
            },
            nym_vpn_account_controller::RequestZkNymError::CreateEcashKeyPair(err) => Self {
                kind: crate::request_zk_nym_error::RequestZkNymErrorType::CreateEcashKeyPair as i32,
                id: None,
                ticketbook_type: None,
                message: Some(err.to_string()),
                message_id: None,
            },
            nym_vpn_account_controller::RequestZkNymError::ConstructWithdrawalRequest(err) => Self {
                kind: crate::request_zk_nym_error::RequestZkNymErrorType::ConstructWithdrawalRequest
                    as i32,
                id: None,
                ticketbook_type: None,
                message: Some(err.to_string()),
                message_id: None,
            },
            nym_vpn_account_controller::RequestZkNymError::RequestZkNymEndpointFailure {
                source,
                ticket_type,
            } => Self {
                kind:
                    crate::request_zk_nym_error::RequestZkNymErrorType::RequestZkNymEndpointFailure
                        as i32,
                id: None,
                ticketbook_type: Some(ticket_type),
                message: Some(source.message.clone()),
                message_id: source.message_id.clone(),
            },
            nym_vpn_account_controller::RequestZkNymError::InvalidTicketTypeInResponse(err) => Self {
                kind: crate::request_zk_nym_error::RequestZkNymErrorType::InvalidTicketTypeInResponse
                    as i32,
                id: None,
                ticketbook_type: None,
                message: Some(err.to_string()),
                message_id: None,
            },
            nym_vpn_account_controller::RequestZkNymError::TicketTypeMismatch => {
                Self {
                    kind: crate::request_zk_nym_error::RequestZkNymErrorType::TicketTypeMismatch
                        as i32,
                    id: None,
                    ticketbook_type: None,
                    message: None,
                    message_id: None,
                }
            }
            nym_vpn_account_controller::RequestZkNymError::PollZkNymEndpointFailure {
                source,
            } => Self {
                kind: crate::request_zk_nym_error::RequestZkNymErrorType::PollZkNymEndpointFailure
                    as i32,
                id: None,
                ticketbook_type: None,
                message: Some(source.message.clone()),
                message_id: source.message_id.clone(),
            },
            nym_vpn_account_controller::RequestZkNymError::PollingTaskError => Self {
                kind: crate::request_zk_nym_error::RequestZkNymErrorType::PollingTaskError as i32,
                id: None,
                ticketbook_type: None,
                message: None,
                message_id: None,
            },
            nym_vpn_account_controller::RequestZkNymError::PollingTimeout { id } => {
                Self {
                    kind: crate::request_zk_nym_error::RequestZkNymErrorType::PollingTimeout as i32,
                    id: Some(id.clone()),
                    ticketbook_type: None,
                    message: None,
                    message_id: None,
                }
            }
            nym_vpn_account_controller::RequestZkNymError::FinishedWithError {
                id,
                ticket_type,
                status,
            } => Self {
                kind: crate::request_zk_nym_error::RequestZkNymErrorType::FinishedWithError as i32,
                id: Some(id.clone()),
                ticketbook_type: Some(ticket_type),
                message: Some(status.to_string()),
                message_id: None,
            },
            nym_vpn_account_controller::RequestZkNymError::MissingBlindedShares => Self {
                kind: crate::request_zk_nym_error::RequestZkNymErrorType::MissingBlindedShares as i32,
                id: None,
                ticketbook_type: None,
                message: None,
                message_id: None,
            },
            nym_vpn_account_controller::RequestZkNymError::ResponseHasInvalidMasterVerificationKey(err) => {
                Self {
                    kind: crate::request_zk_nym_error::RequestZkNymErrorType::ResponseHasInvalidMasterVerificationKey as i32,
                    id: None,
                    ticketbook_type: None,
                    message: Some(err.to_string()),
                    message_id: None,
                }
            },
            nym_vpn_account_controller::RequestZkNymError::EpochIdMismatch => Self {
                kind: crate::request_zk_nym_error::RequestZkNymErrorType::EpochIdMismatch as i32,
                id: None,
                ticketbook_type: None,
                message: None,
                message_id: None,
            },
            nym_vpn_account_controller::RequestZkNymError::ExpirationDateMismatch => Self {
                kind: crate::request_zk_nym_error::RequestZkNymErrorType::ExpirationDateMismatch as i32,
                id: None,
                ticketbook_type: None,
                message: None,
                message_id: None,
            },
            nym_vpn_account_controller::RequestZkNymError::GetPartialVerificationKeysEndpointFailure { source, .. } => {
                Self {
                    kind: crate::request_zk_nym_error::RequestZkNymErrorType::GetPartialVerificationKeysEndpointFailure as i32,
                    id: None,
                    ticketbook_type: None,
                    message: Some(source.message.clone()),
                    message_id: source.message_id.clone(),
                }
            },
            nym_vpn_account_controller::RequestZkNymError::NoMasterVerificationKeyInStorage => Self {
                kind: crate::request_zk_nym_error::RequestZkNymErrorType::NoMasterVerificationKeyInStorage as i32,
                id: None,
                ticketbook_type: None,
                message: None,
                message_id: None,
            },
            nym_vpn_account_controller::RequestZkNymError::NoCoinIndexSignaturesInStorage => Self {
                kind: crate::request_zk_nym_error::RequestZkNymErrorType::NoCoinIndexSignaturesInStorage as i32,
                id: None,
                ticketbook_type: None,
                message: None,
                message_id: None,
            },
            nym_vpn_account_controller::RequestZkNymError::NoExpirationDateSignaturesInStorage => Self {
                kind: crate::request_zk_nym_error::RequestZkNymErrorType::NoExpirationDateSignaturesInStorage as i32,
                id: None,
                ticketbook_type: None,
                message: None,
                message_id: None,
            },
            nym_vpn_account_controller::RequestZkNymError::InvalidVerificationKey(err) => Self {
                kind: crate::request_zk_nym_error::RequestZkNymErrorType::InvalidVerificationKey as i32,
                id: None,
                ticketbook_type: None,
                message: Some(err.to_string()),
                message_id: None,
            },
            nym_vpn_account_controller::RequestZkNymError::DeserializeBlindedSignature(err) => Self {
                kind: crate::request_zk_nym_error::RequestZkNymErrorType::DeserializeBlindedSignature as i32,
                id: None,
                ticketbook_type: None,
                message: Some(err.to_string()),
                message_id: None,
            },
            nym_vpn_account_controller::RequestZkNymError::DecodedKeysMissingIndex => Self {
                kind: crate::request_zk_nym_error::RequestZkNymErrorType::DecodedKeysMissingIndex as i32,
                id: None,
                ticketbook_type: None,
                message: None,
                message_id: None,
            },
            nym_vpn_account_controller::RequestZkNymError::ImportZkNym { ticket_type, error } => {
                Self {
                    kind: crate::request_zk_nym_error::RequestZkNymErrorType::Import as i32,
                    id: None,
                    ticketbook_type: Some(ticket_type),
                    message: Some(error.to_string()),
                    message_id: None,
                }
            }
            nym_vpn_account_controller::RequestZkNymError::AggregateWallets(err) => Self {
                kind: crate::request_zk_nym_error::RequestZkNymErrorType::AggregateWallets as i32,
                id: None,
                ticketbook_type: None,
                message: Some(err.to_string()),
                message_id: None,
            },
            nym_vpn_account_controller::RequestZkNymError::ConfirmZkNymDownloadEndpointFailure { source, id } => {
                Self {
                    kind: crate::request_zk_nym_error::RequestZkNymErrorType::ConfirmZkNymDownloadEndpointFailure as i32,
                    id: Some(id.clone()),
                    ticketbook_type: None,
                    message: Some(source.message.clone()),
                    message_id: source.message_id.clone(),
                }
            },
            nym_vpn_account_controller::RequestZkNymError::MissingPendingRequest(id) => Self {
                kind: crate::request_zk_nym_error::RequestZkNymErrorType::MissingPendingRequest as i32,
                id: Some(id.clone()),
                ticketbook_type: None,
                message: None,
                message_id: None,
            },
            nym_vpn_account_controller::RequestZkNymError::RemovePendingRequest { id, error } => Self {
                kind: crate::request_zk_nym_error::RequestZkNymErrorType::RemovePendingRequest as i32,
                id: Some(id.clone()),
                ticketbook_type: None,
                message: Some(error.to_string()),
                message_id: None,
            },
            nym_vpn_account_controller::RequestZkNymError::CredentialStorage(credential_error) => Self {
                kind: crate::request_zk_nym_error::RequestZkNymErrorType::CredentialStorage as i32,
                id: None,
                ticketbook_type: None,
                message: Some(credential_error.to_string()),
                message_id: None,
            },
            nym_vpn_account_controller::RequestZkNymError::Internal(err) => Self {
                kind: crate::request_zk_nym_error::RequestZkNymErrorType::Internal as i32,
                id: None,
                ticketbook_type: None,
                message: Some(err.to_string()),
                message_id: None,
            },
            nym_vpn_account_controller::RequestZkNymError::UnexpectedErrorResponse(err) => Self {
                kind: crate::request_zk_nym_error::RequestZkNymErrorType::UnexpectedErrorResponse as i32,
                id: None,
                ticketbook_type: None,
                message: Some(err.to_string()),
                message_id: None,
            },
        }
    }
}

impl From<nym_vpn_account_controller::shared_state::RequestZkNymResult>
    for crate::RequestZkNymResult
{
    fn from(zk_nym_request: nym_vpn_account_controller::shared_state::RequestZkNymResult) -> Self {
        match zk_nym_request {
            nym_vpn_account_controller::shared_state::RequestZkNymResult::InProgress => {
                crate::RequestZkNymResult {
                    kind: crate::request_zk_nym_result::RequestZkNymResultType::InProgress as i32,
                    successes: Default::default(),
                    failures: Default::default(),
                }
            }
            nym_vpn_account_controller::shared_state::RequestZkNymResult::Done {
                successes,
                failures,
            } => crate::RequestZkNymResult {
                kind: crate::request_zk_nym_result::RequestZkNymResultType::Done as i32,
                successes: successes
                    .into_iter()
                    .map(crate::RequestZkNymSuccess::from)
                    .collect(),
                failures: failures
                    .into_iter()
                    .map(crate::RequestZkNymError::from)
                    .collect(),
            },
            nym_vpn_account_controller::shared_state::RequestZkNymResult::Error(e) => {
                crate::RequestZkNymResult {
                    kind: crate::request_zk_nym_result::RequestZkNymResultType::Error as i32,
                    successes: Default::default(),
                    failures: vec![crate::RequestZkNymError::from(e)],
                }
            }
        }
    }
}

impl From<nym_vpn_account_controller::AccountStateSummary> for crate::AccountStateSummary {
    fn from(state: nym_vpn_account_controller::AccountStateSummary) -> Self {
        Self {
            mnemonic: state
                .mnemonic
                .map(crate::MnemonicState::from)
                .map(|m| m as i32),
            account_registered: state
                .account_registered
                .map(crate::AccountRegistered::from)
                .map(|m| m as i32),
            account_summary: state.account_summary.map(crate::AccountSummary::from),
            device: state.device.map(crate::DeviceState::from).map(|m| m as i32),
            register_device_result: state
                .register_device_result
                .map(crate::RegisterDeviceResult::from),
            request_zk_nym_result: state
                .request_zk_nym_result
                .map(crate::RequestZkNymResult::from),
        }
    }
}

impl From<nym_vpn_api_client::response::NymVpnUsage> for crate::AccountUsage {
    fn from(usage: nym_vpn_api_client::response::NymVpnUsage) -> Self {
        Self {
            created_on_utc: usage.created_on_utc,
            last_updated_utc: usage.last_updated_utc,
            id: usage.id,
            subscription_id: usage.subscription_id,
            valid_until_utc: usage.valid_until_utc,
            valid_from_utc: usage.valid_from_utc,
            bandwidth_allowance_gb: usage.bandwidth_allowance_gb,
            bandwidth_used_gb: usage.bandwidth_used_gb,
        }
    }
}

impl From<Vec<nym_vpn_api_client::response::NymVpnUsage>> for crate::AccountUsages {
    fn from(usage: Vec<nym_vpn_api_client::response::NymVpnUsage>) -> Self {
        Self {
            account_usages: usage.into_iter().map(crate::AccountUsage::from).collect(),
        }
    }
}

impl From<nym_vpn_api_client::response::NymVpnDeviceStatus> for crate::DeviceStatus {
    fn from(value: nym_vpn_api_client::response::NymVpnDeviceStatus) -> Self {
        match value {
            nym_vpn_api_client::response::NymVpnDeviceStatus::Active => Self::Active,
            nym_vpn_api_client::response::NymVpnDeviceStatus::Inactive => Self::Inactive,
            nym_vpn_api_client::response::NymVpnDeviceStatus::DeleteMe => Self::DeleteMe,
        }
    }
}

impl From<nym_vpn_api_client::response::NymVpnDevice> for crate::Device {
    fn from(device: nym_vpn_api_client::response::NymVpnDevice) -> Self {
        Self {
            created_on_utc: device.created_on_utc,
            last_updated_utc: device.last_updated_utc,
            device_identity_key: device.device_identity_key,
            status: crate::DeviceStatus::from(device.status) as i32,
        }
    }
}

impl From<Vec<nym_vpn_api_client::response::NymVpnDevice>> for crate::Devices {
    fn from(devices: Vec<nym_vpn_api_client::response::NymVpnDevice>) -> Self {
        Self {
            devices: devices.into_iter().map(crate::Device::from).collect(),
        }
    }
}
