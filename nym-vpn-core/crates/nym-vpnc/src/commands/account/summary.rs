// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::{
    NymVpnSubscriptionKind, NymVpnSubscriptionStatus, StoredAccountMode, Subscription,
    VpnAccountAuthMethod, VpnAccountStatus, VpnAccountSummary,
};
use time::{OffsetDateTime, UtcOffset};

use crate::display_helpers::{display_yes_no, print_row};

const WIDTH: usize = 22;

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

    print_row(WIDTH, "Account", &summary.account_addr);
    if let Some(canonical_account_addr) = summary
        .canonical_account_addr
        .as_ref()
        .filter(|addr| **addr != summary.account_addr)
    {
        print_row(WIDTH, "Canonical account", canonical_account_addr);
    }
    print_row(WIDTH, "Mode", display_account_mode(summary.account_mode));
    print_row(
        WIDTH,
        "Status",
        display_account_status(&summary.account_status),
    );
    print_row(
        WIDTH,
        "This device",
        display_device_active(summary.is_device_active),
    );
    print_row(WIDTH, "Remaining devices", summary.remaining_devices);
    print_row(WIDTH, "Last synced", last_synced);

    println!();
    print_subscription(
        summary.subscription.as_ref(),
        summary.is_subscription_stacked,
    );

    println!();
    print_traffic(summary);

    println!();
    print_auth_methods(&summary.auth_methods);
}

fn print_subscription(subscription: Option<&Subscription>, is_stacked: bool) {
    let Some(subscription) = subscription else {
        print_row(WIDTH, "Subscription", "none");
        return;
    };
    let details = &subscription.subscription;

    println!("Subscription:");
    print_row(WIDTH, "  Kind", display_subscription_kind(&details.kind));
    print_row(
        WIDTH,
        "  Status",
        display_subscription_status(&subscription.status),
    );
    print_row(
        WIDTH,
        "  Valid from",
        display_unix_timestamp(details.valid_from_utc),
    );
    print_row(
        WIDTH,
        "  Valid until",
        display_unix_timestamp(details.valid_until_utc),
    );
    print_row(WIDTH, "  Recurring", display_yes_no(details.is_recurring));
    print_row(WIDTH, "  Stacked", display_yes_no(is_stacked));
}

fn print_traffic(summary: &VpnAccountSummary) {
    println!("Traffic:");
    if summary.fair_usage_data_unavailable {
        print_row(WIDTH, "  Used", "unavailable");
    } else {
        print_row(
            WIDTH,
            "  Used",
            format!(
                "{} GB of {} GB",
                summary.traffic_used_gb, summary.traffic_limit_gb
            ),
        );
    }
    if let Some(traffic_reset_time) = summary.traffic_reset_time {
        print_row(WIDTH, "  Resets", display_date_time(traffic_reset_time));
    }
}

fn print_auth_methods(auth_methods: &[VpnAccountAuthMethod]) {
    if auth_methods.is_empty() {
        print_row(WIDTH, "Sign-in methods", "none");
        return;
    }

    println!("Sign-in methods:");
    for auth_method in auth_methods {
        print_row(
            WIDTH,
            &format!("  {}", auth_method.label),
            format!(
                "{}, {}, created {}",
                auth_method.kind,
                display_account_status(&auth_method.status),
                display_date_time(auth_method.created)
            ),
        );
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

fn display_unix_timestamp(timestamp: i64) -> String {
    OffsetDateTime::from_unix_timestamp(timestamp)
        .map_or_else(|_| timestamp.to_string(), display_date_time)
}

fn display_date_time(date_time: OffsetDateTime) -> String {
    let date_time = date_time.to_offset(UtcOffset::UTC);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02} UTC",
        date_time.year(),
        u8::from(date_time.month()),
        date_time.day(),
        date_time.hour(),
        date_time.minute()
    )
}
