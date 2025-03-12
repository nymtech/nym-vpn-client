// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;

use nym_credential_storage::models::BasicTicketbookInformation;
use nym_credentials_interface::TicketType;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use time::Date;

use crate::error::Error;

// If we go below this threshold, we should request more tickets
const TICKET_NUMBER_THRESHOLD: u64 = 30;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AvailableTicketbook {
    pub id: i64,
    pub typ: TicketType,
    pub expiration: Date,
    pub issued_tickets: u32,
    pub claimed_tickets: u32,
    pub ticket_size: u64,
}

impl AvailableTicketbook {
    pub fn issued_tickets(&self) -> u32 {
        self.issued_tickets
    }

    pub fn issued_tickets_si(&self) -> String {
        si_scale::helpers::bibytes2((self.issued_tickets as u64 * self.ticket_size) as f64)
    }

    pub fn claimed_tickets(&self) -> u32 {
        self.claimed_tickets
    }

    pub fn claimed_tickets_si(&self) -> String {
        si_scale::helpers::bibytes2((self.claimed_tickets as u64 * self.ticket_size) as f64)
    }

    pub fn remaing_tickets(&self) -> u32 {
        self.issued_tickets.saturating_sub(self.claimed_tickets)
    }

    pub fn remaining_tickets_si(&self) -> String {
        si_scale::helpers::bibytes2((self.remaing_tickets() as u64 * self.ticket_size) as f64)
    }

    pub fn ticket_size(&self) -> u64 {
        self.ticket_size
    }

    pub fn ticket_size_si(&self) -> String {
        si_scale::helpers::bibytes2(self.ticket_size as f64)
    }

    pub fn has_expired(&self) -> bool {
        self.expiration <= nym_ecash_time::ecash_today().date()
    }
}

impl fmt::Display for AvailableTicketbook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ecash_today = nym_ecash_time::ecash_today().date();

        let expiration = if self.expiration <= ecash_today {
            format!("EXPIRED ON: {}", self.expiration)
        } else {
            format!("expires: {}", self.expiration)
        };

        write!(
            f,
            "{{ id: {}, type: {}, tickets: {}/{}, size: {}, remaining: {}/{}, {} }}",
            self.id,
            self.typ,
            self.remaing_tickets(),
            self.issued_tickets,
            self.ticket_size_si(),
            self.remaining_tickets_si(),
            self.issued_tickets_si(),
            expiration
        )
    }
}

impl TryFrom<BasicTicketbookInformation> for AvailableTicketbook {
    type Error = Error;

    fn try_from(value: BasicTicketbookInformation) -> Result<Self, Self::Error> {
        let typ = value
            .ticketbook_type
            .parse()
            .map_err(|_| Error::ParseTicketType(value.ticketbook_type))?;
        Ok(AvailableTicketbook {
            id: value.id,
            typ,
            expiration: value.expiration_date,
            issued_tickets: value.total_tickets,
            claimed_tickets: value.used_tickets,
            ticket_size: typ.to_repr().bandwidth_value(),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AvailableTicketbooks {
    pub ticketbooks: Vec<AvailableTicketbook>,
}

impl AvailableTicketbooks {
    fn tickets_by_type(&self, typ: TicketType) -> impl Iterator<Item = &AvailableTicketbook> {
        self.ticketbooks
            .iter()
            .filter(move |ticketbook| ticketbook.typ == typ)
    }

    pub fn remaining_tickets(&self, typ: TicketType) -> u64 {
        self.tickets_by_type(typ)
            .filter(|ticketbook| !ticketbook.has_expired())
            .map(|ticketbook| ticketbook.remaing_tickets())
            .fold(0, |acc, remaining| acc.saturating_add(remaining.into()))
    }

    pub fn remaining_data(&self, typ: TicketType) -> u64 {
        self.remaining_tickets(typ) * typ.to_repr().bandwidth_value()
    }

    pub fn remaining_data_si(&self, typ: TicketType) -> String {
        si_scale::helpers::bibytes2(
            self.remaining_tickets(typ) as f64 * typ.to_repr().bandwidth_value() as f64,
        )
    }

    pub fn ticket_types_above_threshold(&self, threshold: u64) -> Vec<TicketType> {
        Self::ticketbook_types()
            .filter(|ticket_type| self.remaining_tickets(*ticket_type) > threshold)
            .collect()
    }

    pub fn ticket_types_below_or_at_threshold(&self, threshold: u64) -> Vec<TicketType> {
        Self::ticketbook_types()
            .filter(|ticket_type| self.remaining_tickets(*ticket_type) <= threshold)
            .collect()
    }

    pub fn ticket_types_above_soft_threshold(&self) -> Vec<TicketType> {
        self.ticket_types_above_threshold(TICKET_NUMBER_THRESHOLD)
    }

    pub fn ticket_types_below_or_at_soft_threshold(&self) -> Vec<TicketType> {
        self.ticket_types_below_or_at_threshold(TICKET_NUMBER_THRESHOLD)
    }

    pub fn is_all_ticket_types_above_threshold(&self, threshold: u64) -> bool {
        self.ticket_types_above_threshold(threshold).len() == Self::ticketbook_types().count()
    }

    pub fn is_all_ticket_types_above_soft_threshold(&self) -> bool {
        self.is_all_ticket_types_above_threshold(TICKET_NUMBER_THRESHOLD)
    }

    pub fn ticket_types_running_low(&self) -> Vec<TicketType> {
        let remaining_tickets = self
            .ticket_types_above_threshold(0)
            .into_iter()
            .map(|ticket_type| format!("{ticket_type}: {}", self.remaining_tickets(ticket_type)))
            .collect::<Vec<String>>()
            .join(", ");
        tracing::debug!("Remaining unexpired tickets: {remaining_tickets}");

        self.ticket_types_below_or_at_soft_threshold()
    }

    pub fn len(&self) -> usize {
        self.ticketbooks.len()
    }

    pub fn len_not_expired(&self) -> usize {
        self.ticketbooks
            .iter()
            .filter(|ticketbook| !ticketbook.has_expired())
            .count()
    }

    pub fn is_empty(&self) -> bool {
        self.ticketbooks.is_empty()
    }

    fn ticketbook_types() -> impl Iterator<Item = TicketType> {
        // We don't include the mixnet exit ticket type as it's not used by the client
        TicketType::iter().filter(|&t| t != TicketType::V1MixnetExit)
    }
}

impl Iterator for AvailableTicketbooks {
    type Item = AvailableTicketbook;

    fn next(&mut self) -> Option<Self::Item> {
        self.ticketbooks.pop()
    }
}

impl From<Vec<AvailableTicketbook>> for AvailableTicketbooks {
    fn from(ticketbooks: Vec<AvailableTicketbook>) -> Self {
        Self { ticketbooks }
    }
}

impl TryFrom<Vec<BasicTicketbookInformation>> for AvailableTicketbooks {
    type Error = Error;

    fn try_from(value: Vec<BasicTicketbookInformation>) -> Result<Self, Self::Error> {
        let ticketbooks: Vec<_> = value
            .into_iter()
            .filter_map(|ticketbook| {
                AvailableTicketbook::try_from(ticketbook)
                    .inspect_err(|err| {
                        tracing::error!("Failed to parse ticketbook: {}", err);
                    })
                    .ok()
            })
            .collect();
        Ok(AvailableTicketbooks::from(ticketbooks))
    }
}
