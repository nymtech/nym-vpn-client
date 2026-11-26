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
