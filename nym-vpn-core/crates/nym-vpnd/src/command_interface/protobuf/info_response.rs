// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_credentials_interface::TicketType;

pub fn into_proto_available_tickets(
    ticketbooks: nym_vpn_account_controller::AvailableTicketbooks,
) -> nym_vpn_proto::get_available_tickets_response::AvailableTickets {
    nym_vpn_proto::get_available_tickets_response::AvailableTickets {
        mixnet_entry_tickets: ticketbooks.remaining_tickets(TicketType::V1MixnetEntry),
        mixnet_entry_data: ticketbooks.remaining_data(TicketType::V1MixnetEntry),
        mixnet_entry_data_si: ticketbooks.remaining_data_si(TicketType::V1MixnetEntry),
        mixnet_exit_tickets: ticketbooks.remaining_tickets(TicketType::V1MixnetExit),
        mixnet_exit_data: ticketbooks.remaining_data(TicketType::V1MixnetExit),
        mixnet_exit_data_si: ticketbooks.remaining_data_si(TicketType::V1MixnetExit),
        vpn_entry_tickets: ticketbooks.remaining_tickets(TicketType::V1WireguardEntry),
        vpn_entry_data: ticketbooks.remaining_data(TicketType::V1WireguardEntry),
        vpn_entry_data_si: ticketbooks.remaining_data_si(TicketType::V1WireguardEntry),
        vpn_exit_tickets: ticketbooks.remaining_tickets(TicketType::V1WireguardExit),
        vpn_exit_data: ticketbooks.remaining_data(TicketType::V1WireguardExit),
        vpn_exit_data_si: ticketbooks.remaining_data_si(TicketType::V1WireguardExit),
    }
}
