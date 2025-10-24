// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(Clone, Debug)]
pub enum Platform {
    Apple,
    Android { purchase_token: String },
}
