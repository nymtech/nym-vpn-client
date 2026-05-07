// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::{
    AccountCommandError, AutologinResponse, AvailableTickets, DeeplinkClient, DeeplinkKind,
    GetDeeplinkParams, NymVpnSubscription, NymVpnSubscriptionKind, NymVpnSubscriptionStatus,
    StoredAccountMode, Subscription, VpnAccountAuthMethod, VpnAccountStatus, VpnAccountSummary,
    VpnApiError, VpnApiErrorResponse,
};

use crate::{
    conversions::{
        ConversionError,
        prost::{offset_datetime_into_proto_timestamp, prost_timestamp_into_offset_datetime},
    },
    proto,
};

impl TryFrom<proto::AccountCommandError> for AccountCommandError {
    type Error = ConversionError;

    fn try_from(value: proto::AccountCommandError) -> Result<Self, Self::Error> {
        let error_detail = value.error_detail.ok_or(ConversionError::NoValueSet(
            "StoreAccountError.error_detail",
        ))?;
        Ok(match error_detail {
            proto::account_command_error::ErrorDetail::StorageError(err) => Self::Storage(err),
            proto::account_command_error::ErrorDetail::Internal(err) => Self::Internal(err),
            proto::account_command_error::ErrorDetail::VpnApi(vpn_api) => {
                Self::VpnApi(vpn_api.try_into()?)
            }
            proto::account_command_error::ErrorDetail::UnexpectedResponse(err) => {
                Self::UnexpectedVpnApiResponse(err)
            }
            proto::account_command_error::ErrorDetail::NoAccountStored(_) => Self::NoAccountStored,
            proto::account_command_error::ErrorDetail::NoDeviceStored(_) => Self::NoDeviceStored,
            proto::account_command_error::ErrorDetail::ExistingAccount(_) => Self::ExistingAccount,
            proto::account_command_error::ErrorDetail::Offline(_) => Self::Offline,
            proto::account_command_error::ErrorDetail::InsufficientFunds(_) => {
                Self::InsufficientFunds
            }
            proto::account_command_error::ErrorDetail::InvalidMnemonic(message) => {
                Self::InvalidMnemonic(message)
            }
            proto::account_command_error::ErrorDetail::InvalidSecret(message) => {
                Self::InvalidSecret(message)
            }
            proto::account_command_error::ErrorDetail::NyxdConnectionFailure(err) => {
                Self::NyxdConnectionFailure(err)
            }
            proto::account_command_error::ErrorDetail::NyxdQueryFailure(err) => {
                Self::NyxdQueryFailure(err)
            }
            proto::account_command_error::ErrorDetail::AccountDoesntExistOnChain(_) => {
                Self::AccountDoesntExistOnChain
            }
            proto::account_command_error::ErrorDetail::AccountDecentralised(_) => {
                Self::AccountDecentralised
            }
            proto::account_command_error::ErrorDetail::AccountNotDecentralised(_) => {
                Self::AccountNotDecentralised
            }
            proto::account_command_error::ErrorDetail::ZkNymAcquisitionFailure(err) => {
                Self::ZkNymAcquisitionFailure(err)
            }
            proto::account_command_error::ErrorDetail::DeeplinkError(message) => {
                Self::DeeplinkError(message)
            }
        })
    }
}

impl TryFrom<proto::VpnApiError> for VpnApiError {
    type Error = ConversionError;

    fn try_from(value: proto::VpnApiError) -> Result<Self, Self::Error> {
        let error_detail = value
            .error_detail
            .ok_or(ConversionError::NoValueSet("VpnApiError.error_detail"))?;
        Ok(match error_detail {
            proto::vpn_api_error::ErrorDetail::Timeout(msg) => Self::Timeout(msg),
            proto::vpn_api_error::ErrorDetail::StatusCode(e) => Self::StatusCode {
                code: e.code.try_into().map_err(|e| {
                    ConversionError::Generic(format!("failed to convert status code: {e}"))
                })?,
                msg: e.message,
            },
            proto::vpn_api_error::ErrorDetail::Response(vpn_api_error_response) => {
                Self::Response(vpn_api_error_response.into())
            }
        })
    }
}

impl From<proto::VpnApiErrorResponse> for VpnApiErrorResponse {
    fn from(value: proto::VpnApiErrorResponse) -> Self {
        Self {
            message: value.message,
            message_id: value.message_id,
            code_reference_id: value.code_reference_id,
        }
    }
}

impl From<AccountCommandError> for proto::AccountCommandError {
    fn from(value: AccountCommandError) -> Self {
        match value {
            AccountCommandError::Internal(err) => proto::AccountCommandError {
                error_detail: Some(proto::account_command_error::ErrorDetail::Internal(err)),
            },
            AccountCommandError::Storage(err) => proto::AccountCommandError {
                error_detail: Some(proto::account_command_error::ErrorDetail::StorageError(err)),
            },
            AccountCommandError::VpnApi(vpn_api_error) => proto::AccountCommandError {
                error_detail: Some(proto::account_command_error::ErrorDetail::VpnApi(
                    vpn_api_error.into(),
                )),
            },
            AccountCommandError::UnexpectedVpnApiResponse(err) => proto::AccountCommandError {
                error_detail: Some(
                    proto::account_command_error::ErrorDetail::UnexpectedResponse(err),
                ),
            },
            AccountCommandError::NoAccountStored => proto::AccountCommandError {
                error_detail: Some(proto::account_command_error::ErrorDetail::NoAccountStored(
                    true,
                )),
            },
            AccountCommandError::NoDeviceStored => proto::AccountCommandError {
                error_detail: Some(proto::account_command_error::ErrorDetail::NoDeviceStored(
                    true,
                )),
            },
            AccountCommandError::ExistingAccount => proto::AccountCommandError {
                error_detail: Some(proto::account_command_error::ErrorDetail::ExistingAccount(
                    true,
                )),
            },
            AccountCommandError::Offline => proto::AccountCommandError {
                error_detail: Some(proto::account_command_error::ErrorDetail::Offline(true)),
            },
            AccountCommandError::InsufficientFunds => proto::AccountCommandError {
                error_detail: Some(
                    proto::account_command_error::ErrorDetail::InsufficientFunds(true),
                ),
            },
            AccountCommandError::InvalidMnemonic(err) => proto::AccountCommandError {
                error_detail: Some(proto::account_command_error::ErrorDetail::InvalidMnemonic(
                    err,
                )),
            },
            AccountCommandError::InvalidSecret(err) => proto::AccountCommandError {
                error_detail: Some(proto::account_command_error::ErrorDetail::InvalidSecret(
                    err,
                )),
            },
            AccountCommandError::NyxdConnectionFailure(err) => proto::AccountCommandError {
                error_detail: Some(
                    proto::account_command_error::ErrorDetail::NyxdConnectionFailure(err),
                ),
            },
            AccountCommandError::NyxdQueryFailure(err) => proto::AccountCommandError {
                error_detail: Some(proto::account_command_error::ErrorDetail::NyxdQueryFailure(
                    err,
                )),
            },
            AccountCommandError::AccountDoesntExistOnChain => proto::AccountCommandError {
                error_detail: Some(
                    proto::account_command_error::ErrorDetail::AccountDoesntExistOnChain(true),
                ),
            },
            AccountCommandError::AccountNotDecentralised => proto::AccountCommandError {
                error_detail: Some(
                    proto::account_command_error::ErrorDetail::AccountNotDecentralised(true),
                ),
            },
            AccountCommandError::AccountDecentralised => proto::AccountCommandError {
                error_detail: Some(
                    proto::account_command_error::ErrorDetail::AccountDecentralised(true),
                ),
            },
            AccountCommandError::ZkNymAcquisitionFailure(err) => proto::AccountCommandError {
                error_detail: Some(
                    proto::account_command_error::ErrorDetail::ZkNymAcquisitionFailure(err),
                ),
            },
            AccountCommandError::DeeplinkError(message) => proto::AccountCommandError {
                error_detail: Some(proto::account_command_error::ErrorDetail::DeeplinkError(
                    message,
                )),
            },
        }
    }
}

impl From<AvailableTickets> for proto::AvailableTickets {
    fn from(ticketbooks: AvailableTickets) -> Self {
        Self {
            mixnet_entry_tickets: ticketbooks.mixnet_entry_tickets,
            mixnet_entry_data: ticketbooks.mixnet_entry_data,
            mixnet_entry_data_si: ticketbooks.mixnet_entry_data_si,
            mixnet_exit_tickets: ticketbooks.mixnet_exit_tickets,
            mixnet_exit_data: ticketbooks.mixnet_exit_data,
            mixnet_exit_data_si: ticketbooks.mixnet_exit_data_si,
            vpn_entry_tickets: ticketbooks.vpn_entry_tickets,
            vpn_entry_data: ticketbooks.vpn_entry_data,
            vpn_entry_data_si: ticketbooks.vpn_entry_data_si,
            vpn_exit_tickets: ticketbooks.vpn_exit_tickets,
            vpn_exit_data: ticketbooks.vpn_exit_data,
            vpn_exit_data_si: ticketbooks.vpn_exit_data_si,
        }
    }
}

impl From<proto::AvailableTickets> for AvailableTickets {
    fn from(ticketbooks: proto::AvailableTickets) -> Self {
        Self {
            mixnet_entry_tickets: ticketbooks.mixnet_entry_tickets,
            mixnet_entry_data: ticketbooks.mixnet_entry_data,
            mixnet_entry_data_si: ticketbooks.mixnet_entry_data_si,
            mixnet_exit_tickets: ticketbooks.mixnet_exit_tickets,
            mixnet_exit_data: ticketbooks.mixnet_exit_data,
            mixnet_exit_data_si: ticketbooks.mixnet_exit_data_si,
            vpn_entry_tickets: ticketbooks.vpn_entry_tickets,
            vpn_entry_data: ticketbooks.vpn_entry_data,
            vpn_entry_data_si: ticketbooks.vpn_entry_data_si,
            vpn_exit_tickets: ticketbooks.vpn_exit_tickets,
            vpn_exit_data: ticketbooks.vpn_exit_data,
            vpn_exit_data_si: ticketbooks.vpn_exit_data_si,
        }
    }
}

impl From<VpnApiError> for proto::VpnApiError {
    fn from(value: VpnApiError) -> Self {
        let error_detail = match value {
            VpnApiError::Timeout(msg) => proto::vpn_api_error::ErrorDetail::Timeout(msg),
            VpnApiError::StatusCode { code, msg: message } => {
                proto::vpn_api_error::ErrorDetail::StatusCode(proto::vpn_api_error::StatusError {
                    code: u32::from(code),
                    message,
                })
            }
            VpnApiError::Response(vpn_api_error_response) => {
                proto::vpn_api_error::ErrorDetail::Response(vpn_api_error_response.into())
            }
        };
        Self {
            error_detail: Some(error_detail),
        }
    }
}

impl From<VpnApiErrorResponse> for proto::VpnApiErrorResponse {
    fn from(value: VpnApiErrorResponse) -> Self {
        Self {
            message: value.message,
            message_id: value.message_id,
            code_reference_id: value.code_reference_id,
        }
    }
}

impl TryFrom<proto::VpnAccountSummary> for VpnAccountSummary {
    type Error = ConversionError;

    fn try_from(value: proto::VpnAccountSummary) -> Result<Self, Self::Error> {
        let traffic_reset_time = value
            .traffic_reset_time
            .map(|ts| {
                prost_timestamp_into_offset_datetime(ts).map_err(|e| {
                    ConversionError::ConvertTime("VpnAccountSummary.traffic_reset_time", e)
                })
            })
            .transpose()?;

        let account_mode = value
            .account_mode
            .map(|mode| {
                proto::StoredAccountMode::try_from(mode)
                    .map(StoredAccountMode::from)
                    .map_err(|_| ConversionError::NoValueSet("VpnAccountSummary.account_mode"))
            })
            .transpose()?;

        let auth_methods = value
            .auth_methods
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, ConversionError>>()?;

        let subscription = value.subscription.map(Subscription::try_from).transpose()?;

        Ok(Self {
            traffic_used_gb: value.traffic_used_gb,
            traffic_limit_gb: value.traffic_limit_gb,
            traffic_reset_time,
            fair_usage_data_unavailable: value.fair_usage_data_unavailable,
            account_addr: value.account_addr,
            canonical_account_addr: value.canonical_account_addr,
            auth_methods,
            account_mode,
            subscription,
            is_subscription_stacked: value.is_subscription_stacked,
        })
    }
}

impl From<VpnAccountSummary> for proto::VpnAccountSummary {
    fn from(value: VpnAccountSummary) -> Self {
        let traffic_reset_time = value
            .traffic_reset_time
            .map(offset_datetime_into_proto_timestamp);

        let auth_methods = value
            .auth_methods
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, ConversionError>>()
            .unwrap_or_default();

        let account_mode = value
            .account_mode
            .map(|mode| proto::StoredAccountMode::from(mode) as i32);

        let subscription = value.subscription.map(proto::Subscription::from);

        Self {
            traffic_used_gb: value.traffic_used_gb,
            traffic_limit_gb: value.traffic_limit_gb,
            traffic_reset_time,
            fair_usage_data_unavailable: value.fair_usage_data_unavailable,
            account_addr: value.account_addr,
            canonical_account_addr: value.canonical_account_addr,
            auth_methods,
            account_mode,
            subscription,
            is_subscription_stacked: value.is_subscription_stacked,
        }
    }
}

impl From<NymVpnSubscription> for proto::NymVpnSubscription {
    fn from(value: NymVpnSubscription) -> Self {
        Self {
            created_on_utc: value.created_on_utc,
            last_updated_utc: value.last_updated_utc,
            id: value.id,
            valid_until_utc: value.valid_until_utc,
            valid_from_utc: value.valid_from_utc,
            status: value.status,
            kind: Some(proto::NymVpnSubscriptionKind::from(value.kind)),
            is_recurring: value.is_recurring,
        }
    }
}

impl TryFrom<proto::NymVpnSubscription> for NymVpnSubscription {
    type Error = ConversionError;

    fn try_from(value: proto::NymVpnSubscription) -> Result<Self, Self::Error> {
        let kind = value
            .kind
            .map(NymVpnSubscriptionKind::try_from)
            .transpose()?
            .ok_or(ConversionError::NoValueSet("NymVpnSubscription.kind"))?;

        Ok(Self {
            created_on_utc: value.created_on_utc,
            last_updated_utc: value.last_updated_utc,
            id: value.id,
            valid_until_utc: value.valid_until_utc,
            valid_from_utc: value.valid_from_utc,
            status: value.status,
            kind,
            is_recurring: value.is_recurring,
        })
    }
}

impl From<Subscription> for proto::Subscription {
    fn from(value: Subscription) -> Self {
        Self {
            status: proto::NymVpnSubscriptionStatus::from(value.status) as i32,
            subscription: Some(proto::NymVpnSubscription::from(value.subscription)),
        }
    }
}

impl TryFrom<proto::Subscription> for Subscription {
    type Error = ConversionError;

    fn try_from(value: proto::Subscription) -> Result<Self, Self::Error> {
        let status = proto::NymVpnSubscriptionStatus::try_from(value.status)
            .map(NymVpnSubscriptionStatus::from)
            .map_err(|_| ConversionError::NoValueSet("Subscription.status"))?;

        let subscription = value
            .subscription
            .ok_or(ConversionError::NoValueSet("Subscription.subscription"))?
            .try_into()?;

        Ok(Self {
            status,
            subscription,
        })
    }
}

impl TryFrom<proto::VpnAccountAuthMethod> for VpnAccountAuthMethod {
    type Error = ConversionError;

    fn try_from(value: proto::VpnAccountAuthMethod) -> Result<Self, Self::Error> {
        let status: VpnAccountStatus = proto::VpnAccountStatus::try_from(value.status)
            .map_err(|_| ConversionError::NoValueSet("VpnAccountAuthMethod.status"))?
            .into();

        let Some(created) = value.created else {
            return Err(ConversionError::NoValueSet("VpnAccountAuthMethod.created"));
        };
        let created = prost_timestamp_into_offset_datetime(created)
            .map_err(|e| ConversionError::ConvertTime("VpnAccountAuthMethod.created", e))?;

        Ok(Self {
            id: value.id,
            pubkey: value.pubkey,
            kind: value.kind,
            label: value.label,
            status,
            created,
        })
    }
}

impl TryFrom<VpnAccountAuthMethod> for proto::VpnAccountAuthMethod {
    type Error = ConversionError;

    fn try_from(value: VpnAccountAuthMethod) -> Result<Self, Self::Error> {
        let status: proto::VpnAccountStatus = value.status.into();
        Ok(Self {
            id: value.id,
            pubkey: value.pubkey,
            kind: value.kind,
            label: value.label,
            status: status as i32,
            created: Some(offset_datetime_into_proto_timestamp(value.created)), // Should not be optional!
        })
    }
}

impl From<proto::VpnAccountStatus> for VpnAccountStatus {
    fn from(value: proto::VpnAccountStatus) -> Self {
        match value {
            proto::VpnAccountStatus::Active => VpnAccountStatus::Active,
            proto::VpnAccountStatus::Inactive => VpnAccountStatus::Inactive,
            proto::VpnAccountStatus::DeleteMe => VpnAccountStatus::DeleteMe,
        }
    }
}

impl From<VpnAccountStatus> for proto::VpnAccountStatus {
    fn from(value: VpnAccountStatus) -> Self {
        match value {
            VpnAccountStatus::Active => proto::VpnAccountStatus::Active,
            VpnAccountStatus::Inactive => proto::VpnAccountStatus::Inactive,
            VpnAccountStatus::DeleteMe => proto::VpnAccountStatus::DeleteMe,
        }
    }
}

impl From<proto::NymVpnSubscriptionStatus> for NymVpnSubscriptionStatus {
    fn from(value: proto::NymVpnSubscriptionStatus) -> Self {
        match value {
            proto::NymVpnSubscriptionStatus::SubscriptionPending => {
                NymVpnSubscriptionStatus::Pending
            }
            proto::NymVpnSubscriptionStatus::SubscriptionActive => NymVpnSubscriptionStatus::Active,
        }
    }
}

impl From<NymVpnSubscriptionStatus> for proto::NymVpnSubscriptionStatus {
    fn from(value: NymVpnSubscriptionStatus) -> Self {
        match value {
            NymVpnSubscriptionStatus::Pending => {
                proto::NymVpnSubscriptionStatus::SubscriptionPending
            }
            NymVpnSubscriptionStatus::Active => proto::NymVpnSubscriptionStatus::SubscriptionActive,
        }
    }
}

impl From<GetDeeplinkParams> for proto::GetDeeplinkParams {
    fn from(value: GetDeeplinkParams) -> Self {
        let client: proto::DeeplinkClient = value.client.into();
        let kind: proto::DeeplinkKind = value.kind.into();
        Self {
            client: client as i32,
            locale: value.locale,
            kind: kind as i32,
            name: value.name,
        }
    }
}

impl TryFrom<proto::GetDeeplinkParams> for GetDeeplinkParams {
    type Error = ConversionError;

    fn try_from(value: proto::GetDeeplinkParams) -> Result<Self, Self::Error> {
        let client = proto::DeeplinkClient::try_from(value.client)
            .map_err(|_| ConversionError::NoValueSet("GetDeeplinkParams.client"))?;

        let kind = proto::DeeplinkKind::try_from(value.kind)
            .map_err(|_| ConversionError::NoValueSet("GetDeeplinkParams.kind"))?;

        Ok(Self {
            client: client.into(),
            locale: value.locale,
            kind: kind.into(),
            name: value.name,
        })
    }
}

impl From<AutologinResponse> for proto::AutologinResponse {
    fn from(value: AutologinResponse) -> Self {
        Self {
            url: value.url,
            pin_code: value.pin_code,
        }
    }
}

impl TryFrom<proto::AutologinResponse> for AutologinResponse {
    type Error = ConversionError;

    fn try_from(value: proto::AutologinResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            url: value.url,
            pin_code: value.pin_code,
        })
    }
}

impl From<DeeplinkClient> for proto::DeeplinkClient {
    fn from(value: DeeplinkClient) -> Self {
        match value {
            DeeplinkClient::Mobile => proto::DeeplinkClient::ClientMobile,
            DeeplinkClient::Desktop => proto::DeeplinkClient::ClientDesktop,
            DeeplinkClient::Web => proto::DeeplinkClient::ClientWeb,
        }
    }
}

impl From<proto::DeeplinkClient> for DeeplinkClient {
    fn from(value: proto::DeeplinkClient) -> Self {
        match value {
            proto::DeeplinkClient::ClientMobile => DeeplinkClient::Mobile,
            proto::DeeplinkClient::ClientDesktop => DeeplinkClient::Desktop,
            proto::DeeplinkClient::ClientWeb => DeeplinkClient::Web,
        }
    }
}

impl From<DeeplinkKind> for proto::DeeplinkKind {
    fn from(value: DeeplinkKind) -> Self {
        match value {
            DeeplinkKind::Privy => proto::DeeplinkKind::KindPrivy,
            DeeplinkKind::PrivyLink => proto::DeeplinkKind::KindPrivyLink,
            DeeplinkKind::AutologinRenew => proto::DeeplinkKind::AutologinRenew,
            DeeplinkKind::AutologinView => proto::DeeplinkKind::AutologinView,
            DeeplinkKind::CreateAccount => proto::DeeplinkKind::CreateAccount,
        }
    }
}

impl From<proto::DeeplinkKind> for DeeplinkKind {
    fn from(value: proto::DeeplinkKind) -> Self {
        match value {
            proto::DeeplinkKind::KindPrivy => DeeplinkKind::Privy,
            proto::DeeplinkKind::KindPrivyLink => DeeplinkKind::PrivyLink,
            proto::DeeplinkKind::AutologinRenew => DeeplinkKind::AutologinRenew,
            proto::DeeplinkKind::AutologinView => DeeplinkKind::AutologinView,
            proto::DeeplinkKind::CreateAccount => DeeplinkKind::CreateAccount,
        }
    }
}

impl From<StoredAccountMode> for proto::StoredAccountMode {
    fn from(value: StoredAccountMode) -> Self {
        match value {
            StoredAccountMode::Api => proto::StoredAccountMode::ModeApi,
            StoredAccountMode::Decentralised => proto::StoredAccountMode::ModeDecentralised,
            StoredAccountMode::Privy => proto::StoredAccountMode::ModePrivy,
        }
    }
}

impl From<proto::StoredAccountMode> for StoredAccountMode {
    fn from(value: proto::StoredAccountMode) -> Self {
        match value {
            proto::StoredAccountMode::ModeApi => StoredAccountMode::Api,
            proto::StoredAccountMode::ModeDecentralised => StoredAccountMode::Decentralised,
            proto::StoredAccountMode::ModePrivy => StoredAccountMode::Privy,
        }
    }
}

impl From<Option<StoredAccountMode>> for proto::GetAccountModeResponse {
    fn from(value: Option<StoredAccountMode>) -> Self {
        let mode = value.map(|m| proto::StoredAccountMode::from(m) as i32);
        Self { mode }
    }
}

impl TryFrom<proto::GetAccountModeResponse> for Option<StoredAccountMode> {
    type Error = ConversionError;

    fn try_from(value: proto::GetAccountModeResponse) -> Result<Self, Self::Error> {
        match value.mode {
            Some(mode) => {
                let mode = proto::StoredAccountMode::try_from(mode)
                    .map_err(|_| ConversionError::NoValueSet("GetAccountModeResponse.mode"))?;
                Ok(Some(mode.into()))
            }
            None => Ok(None),
        }
    }
}

impl TryFrom<proto::NymVpnSubscriptionKind> for NymVpnSubscriptionKind {
    type Error = ConversionError;

    fn try_from(value: proto::NymVpnSubscriptionKind) -> Result<Self, Self::Error> {
        let state = value
            .kind
            .ok_or(ConversionError::NoValueSet("NymVpnSubscriptionKind.kind"))?;
        Ok(match state {
            proto::nym_vpn_subscription_kind::Kind::OneMonth(
                proto::nym_vpn_subscription_kind::OneMonth {},
            ) => NymVpnSubscriptionKind::OneMonth,
            proto::nym_vpn_subscription_kind::Kind::OneYear(
                proto::nym_vpn_subscription_kind::OneYear {},
            ) => NymVpnSubscriptionKind::OneYear,
            proto::nym_vpn_subscription_kind::Kind::TwoYears(
                proto::nym_vpn_subscription_kind::TwoYears {},
            ) => NymVpnSubscriptionKind::TwoYears,
            proto::nym_vpn_subscription_kind::Kind::FreePass(
                proto::nym_vpn_subscription_kind::Freepass {},
            ) => NymVpnSubscriptionKind::Freepass,
            proto::nym_vpn_subscription_kind::Kind::Other(
                proto::nym_vpn_subscription_kind::Other { other },
            ) => NymVpnSubscriptionKind::Other(other),
        })
    }
}

impl From<NymVpnSubscriptionKind> for proto::NymVpnSubscriptionKind {
    fn from(value: NymVpnSubscriptionKind) -> Self {
        let kind: proto::nym_vpn_subscription_kind::Kind = match value {
            NymVpnSubscriptionKind::OneMonth => proto::nym_vpn_subscription_kind::Kind::OneMonth(
                proto::nym_vpn_subscription_kind::OneMonth {},
            ),
            NymVpnSubscriptionKind::OneYear => proto::nym_vpn_subscription_kind::Kind::OneYear(
                proto::nym_vpn_subscription_kind::OneYear {},
            ),
            NymVpnSubscriptionKind::TwoYears => proto::nym_vpn_subscription_kind::Kind::TwoYears(
                proto::nym_vpn_subscription_kind::TwoYears {},
            ),
            NymVpnSubscriptionKind::Freepass => proto::nym_vpn_subscription_kind::Kind::FreePass(
                proto::nym_vpn_subscription_kind::Freepass {},
            ),
            NymVpnSubscriptionKind::Other(other) => proto::nym_vpn_subscription_kind::Kind::Other(
                proto::nym_vpn_subscription_kind::Other { other },
            ),
        };

        proto::NymVpnSubscriptionKind { kind: Some(kind) }
    }
}
