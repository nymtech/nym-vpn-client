// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(Clone, Copy, Debug)]
pub enum Platform {
    Apple,
    Unspecified,
}

impl AsRef<str> for Platform {
    fn as_ref(&self) -> &str {
        match self {
            Platform::Apple => crate::routes::APPLE,
            Platform::Unspecified => "",
        }
    }
}
