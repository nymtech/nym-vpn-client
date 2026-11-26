// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use clap::ValueEnum;
use tabled::{Table, settings::Style};

/// Environment variable used to select the table style.
const TABLE_STYLE_ENV: &str = "TABLE_STYLE";

/// Table style output.
#[derive(Debug, Default, Copy, Clone, ValueEnum)]
pub enum TableStyle {
    #[default]
    Psql,
    Ascii,
    AsciiRounded,
    Modern,
    Sharp,
    Rounded,
    ModernRounded,
    Extended,
    Markdown,
    ReStructuredText,
    Dots,
    Blank,
}

impl TableStyle {
    /// Help text describing the `TABLE_STYLE` environment variable, laid out like clap's own
    /// option help.
    pub fn env_help() -> String {
        const INDENT: &str = "          ";
        const MAX_WIDTH: usize = 80;

        let name = |style: &TableStyle| {
            style
                .to_possible_value()
                .map(|value| value.get_name().to_owned())
        };
        let default = name(&Self::default()).unwrap_or_default();

        let mut help = format!(
            "Environment variables:\n  {TABLE_STYLE_ENV}\n{INDENT}Table style used to print lists, \
             e.g. gateways or devices\n\n{INDENT}[default: {default}]\n"
        );
        let mut line = format!("{INDENT}[possible values:");
        for (i, style) in Self::value_variants().iter().filter_map(name).enumerate() {
            let separator = if i == 0 { " " } else { ", " };
            if line.len() + separator.len() + style.len() + 1 > MAX_WIDTH {
                help.push_str(&line);
                help.push_str(",\n");
                line = format!("{INDENT}{style}");
            } else {
                line.push_str(separator);
                line.push_str(&style);
            }
        }
        help.push_str(&line);
        help.push(']');
        help
    }

    /// Returns the table style set by the `TABLE_STYLE` environment variable, or the default
    /// style when it's unset or invalid.
    pub fn from_env() -> Self {
        std::env::var(TABLE_STYLE_ENV)
            .ok()
            .and_then(|value| Self::from_str(&value, true).ok())
            .unwrap_or_default()
    }

    pub fn apply_style(&self, table: &mut Table) {
        match self {
            TableStyle::Psql => table.with(Style::psql()),
            TableStyle::Ascii => table.with(Style::ascii()),
            TableStyle::AsciiRounded => table.with(Style::ascii_rounded()),
            TableStyle::Modern => table.with(Style::modern()),
            TableStyle::Sharp => table.with(Style::sharp()),
            TableStyle::Rounded => table.with(Style::rounded()),
            TableStyle::ModernRounded => table.with(Style::modern_rounded()),
            TableStyle::Extended => table.with(Style::extended()),
            TableStyle::Markdown => table.with(Style::markdown()),
            TableStyle::ReStructuredText => table.with(Style::re_structured_text()),
            TableStyle::Dots => table.with(Style::dots()),
            TableStyle::Blank => table.with(Style::blank()),
        };
    }
}
