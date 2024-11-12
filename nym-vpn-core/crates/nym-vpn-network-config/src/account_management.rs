// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use url::Url;

use crate::response::{AccountManagementPathsResponse, AccountManagementResponse};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AccountManagement {
    pub(crate) url: Url,
    pub(crate) paths: AccountManagementPaths,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct AccountManagementPaths {
    pub(crate) sign_up: String,
    pub(crate) sign_in: String,
    pub(crate) account: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedAccountLinks {
    pub sign_up: Url,
    pub sign_in: Url,
    pub account: Url,
}

impl AccountManagement {
    pub fn sign_up_url(&self, locale: &str) -> Option<Url> {
        self.url
            .join(&self.paths.sign_up.replace("{locale}", locale))
            .ok()
    }

    pub fn sign_in_url(&self, locale: &str) -> Option<Url> {
        self.url
            .join(&self.paths.sign_in.replace("{locale}", locale))
            .ok()
    }

    pub fn account_url(&self, locale: &str, account_id: &str) -> Option<Url> {
        self.url
            .join(
                &self
                    .paths
                    .account
                    .replace("{locale}", locale)
                    .replace("{account_id}", account_id),
            )
            .ok()
    }

    pub fn try_into_parsed_links(
        self,
        locale: &str,
        account_id: &str,
    ) -> Result<ParsedAccountLinks, anyhow::Error> {
        Ok(ParsedAccountLinks {
            sign_up: self
                .sign_up_url(locale)
                .ok_or_else(|| anyhow::anyhow!("Failed to parse sign up URL"))?,
            sign_in: self
                .sign_in_url(locale)
                .ok_or_else(|| anyhow::anyhow!("Failed to parse sign in URL"))?,
            account: self
                .account_url(locale, account_id)
                .ok_or_else(|| anyhow::anyhow!("Failed to parse account URL"))?,
        })
    }
}

impl TryFrom<AccountManagementResponse> for AccountManagement {
    type Error = anyhow::Error;

    fn try_from(response: AccountManagementResponse) -> Result<Self, Self::Error> {
        let url = response.url.parse()?;
        Ok(Self {
            url,
            paths: response.paths.into(),
        })
    }
}

impl From<AccountManagementPathsResponse> for AccountManagementPaths {
    fn from(response: AccountManagementPathsResponse) -> Self {
        Self {
            sign_up: response.sign_up,
            sign_in: response.sign_in,
            account: response.account,
        }
    }
}
