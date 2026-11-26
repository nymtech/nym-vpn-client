// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::{
    AccountControllerState, AvailableTickets, NymVpnSubscriptionKind, NymVpnSubscriptionStatus,
    NymVpnUsage, StoredAccountMode, Subscription, VpnAccountAuthMethod, VpnAccountStatus,
    VpnAccountSummary,
};

use crate::display_helpers::{
    Rows, display_date_time, display_rfc3339, display_unix_timestamp, display_yes_no,
};

pub fn print_summary(summary: Option<&VpnAccountSummary>) {
    let Some(summary) = summary else {
        println!("No account summary available");
        return;
    };

    let last_synced = if summary.stale {
        format!("{} (stale)", display_date_time(summary.last_synced_utc))
    } else {
        display_date_time(summary.last_synced_utc)
    };

    let mut rows = Rows::new();
    rows.row("Account", &summary.account_addr);
    if let Some(canonical_account_addr) = summary
        .canonical_account_addr
        .as_ref()
        .filter(|addr| **addr != summary.account_addr)
    {
        rows.row("Canonical account", canonical_account_addr);
    }
    rows.row("Mode", display_account_mode(summary.account_mode))
        .row("Status", display_account_status(&summary.account_status))
        .row(
            "This device",
            display_device_active(summary.is_device_active),
        )
        .row("Remaining devices", summary.remaining_devices)
        .row("Last synced", last_synced)
        .blank();

    add_subscription(
        &mut rows,
        summary.subscription.as_ref(),
        summary.is_subscription_stacked,
    );
    rows.blank();
    add_traffic(&mut rows, summary);
    rows.blank();
    add_auth_methods(&mut rows, &summary.auth_methods);
    rows.print();
}

pub fn print_available_tickets(tickets: &AvailableTickets) {
    Rows::new()
        .row(
            "Mixnet entry",
            display_tickets(tickets.mixnet_entry_tickets, &tickets.mixnet_entry_data_si),
        )
        .row(
            "Mixnet exit",
            display_tickets(tickets.mixnet_exit_tickets, &tickets.mixnet_exit_data_si),
        )
        .row(
            "WireGuard entry",
            display_tickets(tickets.vpn_entry_tickets, &tickets.vpn_entry_data_si),
        )
        .row(
            "WireGuard exit",
            display_tickets(tickets.vpn_exit_tickets, &tickets.vpn_exit_data_si),
        )
        .print();
}

pub fn print_usage(usage: &[NymVpnUsage]) {
    if usage.is_empty() {
        println!("No usage data available");
        return;
    }

    let mut rows = Rows::new();
    for (i, usage) in usage.iter().enumerate() {
        if i > 0 {
            rows.blank();
        }
        rows.row("From", display_rfc3339(&usage.valid_from_utc))
            .row("Until", display_rfc3339(&usage.valid_until_utc))
            .row(
                "Used",
                format!(
                    "{} GB of {} GB",
                    usage.bandwidth_used_gb, usage.bandwidth_allowance_gb
                ),
            );
    }
    rows.print();
}

pub fn display_account_state(state: &AccountControllerState) -> String {
    match state {
        AccountControllerState::Offline => "offline".to_owned(),
        AccountControllerState::Syncing => "syncing".to_owned(),
        AccountControllerState::LoggedOut => "logged out".to_owned(),
        AccountControllerState::ReadyToConnect => "ready to connect".to_owned(),
        AccountControllerState::Decentralised => "decentralised".to_owned(),
        AccountControllerState::PendingSubscription => "pending subscription".to_owned(),
        AccountControllerState::Error(reason) => format!("error: {reason}"),
    }
}

pub fn display_account_mode(account_mode: Option<StoredAccountMode>) -> &'static str {
    match account_mode {
        None => "unset",
        Some(StoredAccountMode::Api) => "api",
        Some(StoredAccountMode::Decentralised) => "decentralised",
        Some(StoredAccountMode::Privy) => "privy",
    }
}

fn add_subscription(rows: &mut Rows, subscription: Option<&Subscription>, is_stacked: bool) {
    let Some(subscription) = subscription else {
        rows.row("Subscription", "none");
        return;
    };
    let details = &subscription.subscription;

    rows.section("Subscription")
        .row("  Kind", display_subscription_kind(&details.kind))
        .row(
            "  Status",
            display_subscription_status(&subscription.status),
        )
        .row(
            "  Valid from",
            display_unix_timestamp(details.valid_from_utc),
        )
        .row(
            "  Valid until",
            display_unix_timestamp(details.valid_until_utc),
        )
        .row("  Recurring", display_yes_no(details.is_recurring))
        .row("  Stacked", display_yes_no(is_stacked));
}

fn add_traffic(rows: &mut Rows, summary: &VpnAccountSummary) {
    let used = if summary.fair_usage_data_unavailable {
        "unavailable".to_owned()
    } else {
        format!(
            "{} GB of {} GB",
            summary.traffic_used_gb, summary.traffic_limit_gb
        )
    };

    rows.section("Traffic").row("  Used", used);
    if let Some(traffic_reset_time) = summary.traffic_reset_time {
        rows.row("  Resets", display_date_time(traffic_reset_time));
    }
}

fn add_auth_methods(rows: &mut Rows, auth_methods: &[VpnAccountAuthMethod]) {
    if auth_methods.is_empty() {
        rows.row("Sign-in methods", "none");
        return;
    }

    rows.section("Sign-in methods");
    for auth_method in auth_methods {
        rows.row(
            format!("  {}", auth_method.label),
            format!(
                "{}, {}, created {}",
                auth_method.kind,
                display_account_status(&auth_method.status),
                display_date_time(auth_method.created)
            ),
        );
    }
}

fn display_tickets(tickets: u64, data: &str) -> String {
    format!("{tickets} tickets ({data})")
}

fn display_account_status(status: &VpnAccountStatus) -> &'static str {
    match status {
        VpnAccountStatus::Active => "active",
        VpnAccountStatus::Inactive => "inactive",
        VpnAccountStatus::DeleteMe => "pending deletion",
    }
}

fn display_device_active(is_device_active: bool) -> &'static str {
    match is_device_active {
        true => "active",
        false => "inactive",
    }
}

fn display_subscription_status(status: &NymVpnSubscriptionStatus) -> &'static str {
    match status {
        NymVpnSubscriptionStatus::Pending => "pending",
        NymVpnSubscriptionStatus::Active => "active",
    }
}

fn display_subscription_kind(kind: &NymVpnSubscriptionKind) -> &str {
    match kind {
        NymVpnSubscriptionKind::OneMonth => "1 month",
        NymVpnSubscriptionKind::SixMonths => "6 months",
        NymVpnSubscriptionKind::OneYear => "1 year",
        NymVpnSubscriptionKind::TwoYears => "2 years",
        NymVpnSubscriptionKind::Freepass => "free pass",
        NymVpnSubscriptionKind::Other(kind) => kind,
    }
}
