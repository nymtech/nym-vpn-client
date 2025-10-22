// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub fn display_on_off(value: bool) -> &'static str {
    match value {
        true => "on",
        false => "off",
    }
}
