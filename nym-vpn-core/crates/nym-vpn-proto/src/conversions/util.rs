// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub fn or_not_set<T: ToString + Clone>(value: &Option<T>) -> String {
    value
        .clone()
        .map(|v| v.to_string())
        .unwrap_or("not set".to_string())
}
