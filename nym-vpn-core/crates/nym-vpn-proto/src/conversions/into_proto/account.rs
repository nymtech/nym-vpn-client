// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::{
    AvailableTickets, RequestZkNymError, RequestZkNymErrorReason, RequestZkNymSuccess,
};

use crate::{
    AvailableTickets as ProtoAvailableTickets, RequestZkNymError as ProtoRequestZkNymError,
    RequestZkNymSuccess as ProtoRequestZkNymSuccess,
};

impl From<RequestZkNymSuccess> for ProtoRequestZkNymSuccess {
    fn from(value: RequestZkNymSuccess) -> Self {
        Self { id: value.id }
    }
}

impl From<RequestZkNymErrorReason> for ProtoRequestZkNymError {
    fn from(error: RequestZkNymErrorReason) -> Self {
        let outcome = match error {
            RequestZkNymErrorReason::NoAccountStored => {
                Some(crate::request_zk_nym_error::Outcome::NoAccountStored(true))
            }
            RequestZkNymErrorReason::NoDeviceStored => {
                Some(crate::request_zk_nym_error::Outcome::NoDeviceStored(true))
            }
            RequestZkNymErrorReason::VpnApi(vpn_api_endpoint_failure) => Some(
                crate::request_zk_nym_error::Outcome::VpnApi(vpn_api_endpoint_failure.into()),
            ),
            RequestZkNymErrorReason::UnexpectedVpnApiResponse(err) => {
                Some(crate::request_zk_nym_error::Outcome::UnexpectedVpnApiResponse(err))
            }
            RequestZkNymErrorReason::Storage(err) => {
                Some(crate::request_zk_nym_error::Outcome::Storage(err))
            }
            RequestZkNymErrorReason::Offline => {
                Some(crate::request_zk_nym_error::Outcome::Offline(true))
            }
            RequestZkNymErrorReason::Internal(err) => {
                Some(crate::request_zk_nym_error::Outcome::Internal(err))
            }
        };
        Self { outcome }
    }
}

impl From<RequestZkNymError> for ProtoRequestZkNymError {
    fn from(error: RequestZkNymError) -> Self {
        RequestZkNymErrorReason::from(error).into()
    }
}

impl From<AvailableTickets> for ProtoAvailableTickets {
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
