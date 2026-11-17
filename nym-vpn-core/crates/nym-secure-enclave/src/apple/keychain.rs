// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::{KeychainAccess, KeychainItem, LocalAuthentication};

/// Creates generic password items in the data protection keychain.
#[derive(Clone)]
pub struct Keychain {
    auth: LocalAuthentication,
    access_group: Option<String>,
}

impl Keychain {
    /// Items requiring [`KeychainAccess::UserPresence`] are accessed with the authentication
    /// obtained by [`LocalAuthentication::unlock`] on `auth`.
    ///
    /// Processes sharing the items, such as an app and its network extension, must use the same
    /// `access_group`. Without one, items go to the default access group of the process.
    pub fn new(auth: LocalAuthentication, access_group: Option<&str>) -> Self {
        Self {
            auth,
            access_group: access_group.map(ToOwned::to_owned),
        }
    }

    /// Item identified by keychain `service` and `account`.
    pub fn item(&self, service: &str, account: &str, access: KeychainAccess) -> KeychainItem {
        KeychainItem::new(
            service,
            account,
            self.access_group.as_deref(),
            access,
            self.auth.clone(),
        )
    }
}
