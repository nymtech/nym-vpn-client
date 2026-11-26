// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub fn display_on_off(value: bool) -> &'static str {
    match value {
        true => "on",
        false => "off",
    }
}

pub fn display_yes_no(value: bool) -> &'static str {
    match value {
        true => "yes",
        false => "no",
    }
}

/// Prints `label: value` with the value aligned at `width` columns.
pub fn print_row(width: usize, label: &str, value: impl std::fmt::Display) {
    println!("{:<width$}{value}", format!("{label}:"));
}
