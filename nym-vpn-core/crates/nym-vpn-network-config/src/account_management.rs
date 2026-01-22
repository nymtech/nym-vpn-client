// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;

use url::Url;

use nym_vpn_api_client::response::{
    AccountManagementPathsResponse, AccountManagementPrivyPathsResponse, AccountManagementResponse,
};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AccountManagement {
    pub(crate) url: Url,
    pub(crate) paths: AccountManagementPaths,
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

    pub fn privy_mobile_url(&self, locale: &str) -> Option<Url> {
        self.url
            .join(&self.paths.privy.mobile.replace("{locale}", locale))
            .ok()
    }

    pub fn privy_desktop_url(&self, locale: &str) -> Option<Url> {
        self.url
            .join(&self.paths.privy.desktop.replace("{locale}", locale))
            .ok()
    }

    pub fn privy_web_url(&self, locale: &str) -> Option<Url> {
        self.url
            .join(&self.paths.privy.web.replace("{locale}", locale))
            .ok()
    }

    pub fn try_into_parsed_links(
        self,
        locale: &str,
        account_id: Option<&str>,
    ) -> Result<ParsedAccountLinks, AccountLinksConversionError> {
        Ok(ParsedAccountLinks {
            sign_up: self
                .sign_up_url(locale)
                .ok_or(AccountLinksConversionError::ParseSignupUrl)?,
            sign_in: self
                .sign_in_url(locale)
                .ok_or(AccountLinksConversionError::ParseSigninUrl)?,
            account: account_id.and_then(|account_id| self.account_url(locale, account_id)),
            privy: ParsedAccountPrivyLinks {
                mobile: self
                    .privy_mobile_url(locale)
                    .ok_or(AccountLinksConversionError::ParsePrivyWebUrl)?,
                desktop: self
                    .privy_desktop_url(locale)
                    .ok_or(AccountLinksConversionError::ParsePrivyDesktopUrl)?,
                web: self
                    .privy_web_url(locale)
                    .ok_or(AccountLinksConversionError::ParsePrivyWebUrl)?,
            },
        })
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, thiserror::Error)]
pub enum AccountLinksConversionError {
    #[error("Failed to parse sign in URL")]
    ParseSigninUrl,

    #[error("Failed to parse sign up URL")]
    ParseSignupUrl,

    #[error("Failed to parse privy mobile URL")]
    ParsePrivyMobileUrl,

    #[error("Failed to parse privy desktop URL")]
    ParsePrivyDesktopUrl,

    #[error("Failed to parse privy web URL")]
    ParsePrivyWebUrl,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct AccountManagementPaths {
    pub(crate) sign_up: String,
    pub(crate) sign_in: String,
    pub(crate) account: String,
    pub(crate) privy: AccountManagementPrivyPaths,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AccountManagementPrivyPaths {
    pub mobile: String,
    pub desktop: String,
    pub web: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedAccountLinks {
    pub sign_up: Url,
    pub sign_in: Url,
    pub account: Option<Url>,
    pub privy: ParsedAccountPrivyLinks,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedAccountPrivyLinks {
    pub mobile: Url,
    pub desktop: Url,
    pub web: Url,
}

impl fmt::Display for ParsedAccountLinks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "sign_up: {}", self.sign_up)?;
        write!(f, "sign_in: {}", self.sign_in)?;
        if let Some(account) = &self.account {
            write!(f, "\naccount: {account}")?;
        }

        Ok(())
    }
}

pub struct TryFromAccountManagementResponseError(url::ParseError);

impl std::fmt::Display for TryFromAccountManagementResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to parse URL: {}", self.0)
    }
}

impl TryFrom<AccountManagementResponse> for AccountManagement {
    type Error = TryFromAccountManagementResponseError;

    fn try_from(response: AccountManagementResponse) -> Result<Self, Self::Error> {
        let url = response
            .url
            .parse()
            .map_err(TryFromAccountManagementResponseError)?;
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
            privy: response.privy.into(),
        }
    }
}

impl From<AccountManagementPrivyPathsResponse> for AccountManagementPrivyPaths {
    fn from(response: AccountManagementPrivyPathsResponse) -> Self {
        Self {
            mobile: response.mobile,
            desktop: response.desktop,
            web: response.web,
        }
    }
}
