// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::ops::Deref;

use clap::builder::{PossibleValuesParser, TypedValueParser, ValueParser};

use nym_vpn_lib_types::GatewaySelectionAlgorithm;

#[derive(Debug, Clone, Copy)]
pub struct GatewaySelectionAlgorithmParser {
    state: GatewaySelectionAlgorithm,
    explicit_label: &'static str,
    auto_entry_label: &'static str,
    auto_label: &'static str,
}

impl Deref for GatewaySelectionAlgorithmParser {
    type Target = GatewaySelectionAlgorithm;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl clap::builder::ValueParserFactory for GatewaySelectionAlgorithmParser {
    type Parser = ValueParser;

    /// A value parser that parses numeric value into a `GatewaySelectionAlgorithm`
    fn value_parser() -> Self::Parser {
        Self::custom_parser("explicit", "auto-entry", "auto")
    }
}

impl GatewaySelectionAlgorithmParser {
    /// A value parser that parses `explicit_label`, `auto_entry_label` and `auto_label`
    /// into a `GatewaySelectionAlgorithmParser`
    fn custom_parser(
        explicit_label: &'static str,
        auto_entry_label: &'static str,
        auto_label: &'static str,
    ) -> ValueParser {
        assert!(explicit_label != auto_entry_label);
        assert!(explicit_label != auto_label);
        assert!(auto_entry_label != auto_label);

        ValueParser::new(
            PossibleValuesParser::new([explicit_label, auto_entry_label, auto_label]).map(
                move |value| Self::with_labels(value, explicit_label, auto_entry_label, auto_label),
            ),
        )
    }

    fn with_labels(
        value: String,
        explicit_label: &'static str,
        auto_entry_label: &'static str,
        auto_label: &'static str,
    ) -> Self {
        let state = match value.as_str() {
            "explicit" => GatewaySelectionAlgorithm::Explicit,
            "auto-entry" => GatewaySelectionAlgorithm::AutoEntryExplicitExit,
            "auto" => GatewaySelectionAlgorithm::Auto,
            _ => panic!("Unrecognized label"),
        };
        Self {
            state,
            explicit_label,
            auto_entry_label,
            auto_label,
        }
    }
}

impl std::fmt::Display for GatewaySelectionAlgorithmParser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.state {
            GatewaySelectionAlgorithm::Explicit => self.explicit_label.fmt(f),
            GatewaySelectionAlgorithm::AutoEntryExplicitExit => self.auto_entry_label.fmt(f),
            GatewaySelectionAlgorithm::Auto => self.auto_label.fmt(f),
        }
    }
}
