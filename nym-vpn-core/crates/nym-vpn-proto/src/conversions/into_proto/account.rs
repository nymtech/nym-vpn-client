// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::{
    AvailableTickets, RequestZkNymError, RequestZkNymErrorReason, RequestZkNymSuccess,
    StoreAccountError,
};

use crate::proto;

impl From<StoreAccountError> for proto::StoreAccountError {
    fn from(value: StoreAccountError) -> Self {
        match value {
            StoreAccountError::InvalidMnemonic(err) => proto::StoreAccountError {
                error_detail: Some(proto::store_account_error::ErrorDetail::InvalidMnemonic(
                    err,
                )),
            },
            StoreAccountError::Storage(err) => proto::StoreAccountError {
                error_detail: Some(proto::store_account_error::ErrorDetail::StorageError(err)),
            },
            StoreAccountError::GetAccountEndpointFailure(vpn_api) => proto::StoreAccountError {
                error_detail: Some(proto::store_account_error::ErrorDetail::VpnApi(
                    vpn_api.into(),
                )),
            },
            StoreAccountError::UnexpectedResponse(err) => proto::StoreAccountError {
                error_detail: Some(proto::store_account_error::ErrorDetail::UnexpectedResponse(
                    err,
                )),
            },
            StoreAccountError::Internal(err) => proto::StoreAccountError {
                error_detail: Some(proto::store_account_error::ErrorDetail::Internal(err)),
            },
        }
    }
}

impl From<RequestZkNymSuccess> for proto::RequestZkNymSuccess {
    fn from(value: RequestZkNymSuccess) -> Self {
        Self { id: value.id }
    }
}

impl From<RequestZkNymErrorReason> for proto::RequestZkNymError {
    fn from(error: RequestZkNymErrorReason) -> Self {
        let outcome = match error {
            RequestZkNymErrorReason::NoAccountStored => {
                proto::request_zk_nym_error::Outcome::NoAccountStored(true)
            }
            RequestZkNymErrorReason::NoDeviceStored => {
                proto::request_zk_nym_error::Outcome::NoDeviceStored(true)
            }
            RequestZkNymErrorReason::VpnApi(vpn_api_endpoint_failure) => {
                proto::request_zk_nym_error::Outcome::VpnApi(vpn_api_endpoint_failure.into())
            }
            RequestZkNymErrorReason::UnexpectedVpnApiResponse(err) => {
                proto::request_zk_nym_error::Outcome::UnexpectedVpnApiResponse(err)
            }
            RequestZkNymErrorReason::Storage(err) => {
                proto::request_zk_nym_error::Outcome::Storage(err)
            }
            RequestZkNymErrorReason::Offline => proto::request_zk_nym_error::Outcome::Offline(true),
            RequestZkNymErrorReason::Internal(err) => {
                proto::request_zk_nym_error::Outcome::Internal(err)
            }
        };
        Self {
            outcome: Some(outcome),
        }
    }
}

impl From<RequestZkNymError> for proto::RequestZkNymError {
    fn from(error: RequestZkNymError) -> Self {
        RequestZkNymErrorReason::from(error).into()
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
