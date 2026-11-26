// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;

use time::{OffsetDateTime, UtcOffset, format_description::well_known::Rfc3339};

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

/// Formats a date as `YYYY-MM-DD HH:MM UTC`.
pub fn display_date_time(date_time: OffsetDateTime) -> String {
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

/// Formats a Unix timestamp in seconds, falling back to the raw number when it's out of range.
pub fn display_unix_timestamp(timestamp: i64) -> String {
    OffsetDateTime::from_unix_timestamp(timestamp)
        .map_or_else(|_| timestamp.to_string(), display_date_time)
}

/// Formats an RFC 3339 date, falling back to the raw string when it can't be parsed.
pub fn display_rfc3339(date_time: &str) -> String {
    OffsetDateTime::parse(date_time, &Rfc3339)
        .map_or_else(|_| date_time.to_owned(), display_date_time)
}

/// Lines of `label: value` rows with values aligned in one column.
#[derive(Debug, Default)]
pub struct Rows {
    lines: Vec<Line>,
}

#[derive(Debug)]
enum Line {
    Row { label: String, value: String },
    Text(String),
    Blank,
}

impl Rows {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a `label: value` row. Indent the label to nest it under a section.
    pub fn row(&mut self, label: impl Into<String>, value: impl fmt::Display) -> &mut Self {
        self.lines.push(Line::Row {
            label: label.into(),
            value: value.to_string(),
        });
        self
    }

    /// Adds a section header, e.g. `Traffic:`.
    pub fn section(&mut self, title: &str) -> &mut Self {
        self.lines.push(Line::Text(format!("{title}:")));
        self
    }

    /// Adds a list under `label`, or a `label: none` row when the list is empty.
    pub fn list<T: fmt::Display>(
        &mut self,
        label: &str,
        items: impl IntoIterator<Item = T>,
    ) -> &mut Self {
        let indent = label.len() - label.trim_start().len();
        let mut items = items.into_iter().peekable();
        if items.peek().is_none() {
            return self.row(label, "none");
        }
        self.section(label);
        for item in items {
            self.lines
                .push(Line::Text(format!("{:indent$}  - {item}", "")));
        }
        self
    }

    pub fn blank(&mut self) -> &mut Self {
        self.lines.push(Line::Blank);
        self
    }

    pub fn print(&self) {
        print!("{self}");
    }
}

impl fmt::Display for Rows {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let width = self
            .lines
            .iter()
            .filter_map(|line| match line {
                Line::Row { label, .. } => Some(label.chars().count() + 2),
                Line::Text(_) | Line::Blank => None,
            })
            .max()
            .unwrap_or_default();

        for line in &self.lines {
            match line {
                Line::Row { label, value } => {
                    writeln!(f, "{:<width$}{value}", format!("{label}:"))?
                }
                Line::Text(text) => writeln!(f, "{text}")?,
                Line::Blank => writeln!(f)?,
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_align_values() {
        let mut rows = Rows::new();
        rows.row("IPv6", "on")
            .blank()
            .section("Traffic")
            .row("  Used", "1 GB")
            .list("  Servers", ["a", "b"])
            .list("Empty", Vec::<String>::new());

        assert_eq!(
            rows.to_string(),
            "IPv6:   on\n\nTraffic:\n  Used: 1 GB\n  Servers:\n    - a\n    - b\nEmpty:  none\n"
        );
    }
}
