// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(Clone, Debug)]
pub enum Platform {
    Apple,
    Android { purchase_token: String },
}
impl Platform {
    pub fn api_path_component(&self) -> &'static str {
        match self {
            Platform::Apple => crate::routes::APPLE,
            Platform::Android { .. } => crate::routes::ANDROID,
        }
    }

    pub fn purchase_token(&self) -> Option<String> {
        match self {
            Platform::Apple => None,
            Platform::Android { purchase_token } => Some(purchase_token.clone()),
        }
    }
}
